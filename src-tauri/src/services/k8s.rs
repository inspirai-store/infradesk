//! Kubernetes service discovery
//!
//! This service handles K8s cluster connections and service discovery.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{ConfigMap, Pod, Secret, Service};
use k8s_openapi::api::networking::v1::Ingress;
use kube::{
    api::{Api, ListParams},
    config::{KubeConfigOptions, Kubeconfig},
    Client, Config,
};
use std::collections::HashMap;

use crate::db::models::{
    DiscoveredService, K8sConfigMapInfo, K8sDeployment, K8sIngressInfo, K8sPod, K8sSecretInfo,
    K8sServiceInfo, ListClustersResponse,
};
use crate::error::{AppError, AppResult};

/// Known database service types and their detection patterns
const MYSQL_PATTERNS: &[&str] = &["mysql", "mariadb", "percona"];
const REDIS_PATTERNS: &[&str] = &["redis", "keydb", "dragonfly"];
const MONGODB_PATTERNS: &[&str] = &["mongo", "mongodb"];

/// Default ports for database services
const MYSQL_DEFAULT_PORT: i32 = 3306;
const REDIS_DEFAULT_PORT: i32 = 6379;
const MONGODB_DEFAULT_PORT: i32 = 27017;

/// Service for Kubernetes operations
pub struct K8sService {
    client: Client,
}

impl K8sService {
    /// Create a new K8sService from kubeconfig content
    pub async fn from_kubeconfig(kubeconfig: &str, context: Option<&str>) -> AppResult<Self> {
        let config = Kubeconfig::from_yaml(kubeconfig)
            .map_err(|e| AppError::K8s(format!("Failed to parse kubeconfig: {}", e)))?;

        let options = KubeConfigOptions {
            context: context.map(String::from),
            ..Default::default()
        };

        let client_config = Config::from_custom_kubeconfig(config, &options)
            .await
            .map_err(|e| AppError::K8s(format!("Failed to create config: {}", e)))?;

        let client = Client::try_from(client_config)
            .map_err(|e| AppError::K8s(format!("Failed to create client: {}", e)))?;

        Ok(Self { client })
    }

    /// Create a K8sService from in-cluster config (when running inside K8s)
    pub async fn in_cluster() -> AppResult<Self> {
        let client = Client::try_default()
            .await
            .map_err(|e| AppError::K8s(format!("Failed to create in-cluster client: {}", e)))?;

        Ok(Self { client })
    }

    /// List all contexts from a kubeconfig
    pub fn list_contexts_from_kubeconfig(kubeconfig: &str) -> AppResult<ListClustersResponse> {
        let config = Kubeconfig::from_yaml(kubeconfig)
            .map_err(|e| AppError::K8s(format!("Failed to parse kubeconfig: {}", e)))?;

        let clusters: Vec<String> = config
            .contexts
            .into_iter()
            .map(|c| c.name)
            .collect();

        Ok(ListClustersResponse { clusters })
    }

    /// Discover database services in all namespaces
    pub async fn discover_services(&self) -> AppResult<Vec<DiscoveredService>> {
        let mut discovered = Vec::new();

        // List all namespaces
        let namespaces = self.list_namespaces().await?;

        for namespace in namespaces {
            // Discover services in each namespace
            let services = self.discover_services_in_namespace(&namespace).await?;
            discovered.extend(services);
        }

        Ok(discovered)
    }

    /// List all namespaces
    async fn list_namespaces(&self) -> AppResult<Vec<String>> {
        let namespaces: Api<k8s_openapi::api::core::v1::Namespace> = Api::all(self.client.clone());
        let ns_list = namespaces
            .list(&ListParams::default())
            .await
            .map_err(|e| AppError::K8s(format!("Failed to list namespaces: {}", e)))?;

        Ok(ns_list.items.into_iter().filter_map(|ns| ns.metadata.name).collect())
    }

    /// Discover database services in a specific namespace
    async fn discover_services_in_namespace(&self, namespace: &str) -> AppResult<Vec<DiscoveredService>> {
        let mut discovered = Vec::new();

        // Get all pods in the namespace
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let pod_list = pods
            .list(&ListParams::default())
            .await
            .map_err(|e| AppError::K8s(format!("Failed to list pods: {}", e)))?;

        // Get all services in the namespace
        let services: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        let service_list = services
            .list(&ListParams::default())
            .await
            .map_err(|e| AppError::K8s(format!("Failed to list services: {}", e)))?;

        // Get all secrets in the namespace (for credentials)
        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), namespace);
        let secret_items: Vec<Secret> = match secrets.list(&ListParams::default()).await {
            Ok(list) => list.items,
            Err(_) => {
                // If we can't list secrets (permission denied), continue without credentials
                vec![]
            }
        };

        // Build a map of secrets for quick lookup
        let secrets_map: HashMap<String, &Secret> = secret_items
            .iter()
            .filter_map(|s| s.metadata.name.as_ref().map(|n| (n.clone(), s)))
            .collect();

        // Check each pod for database containers
        for pod in &pod_list.items {
            let pod_name = pod.metadata.name.as_deref().unwrap_or("");
            let labels = pod.metadata.labels.as_ref();

            // Skip if no containers
            let containers = match &pod.spec {
                Some(spec) => &spec.containers,
                None => continue,
            };

            for container in containers {
                let image = container.image.as_deref().unwrap_or("");
                let container_name = &container.name;

                // Detect service type from image name
                let (service_type, default_port) = if MYSQL_PATTERNS.iter().any(|p| image.contains(p)) {
                    ("mysql", MYSQL_DEFAULT_PORT)
                } else if REDIS_PATTERNS.iter().any(|p| image.contains(p)) {
                    ("redis", REDIS_DEFAULT_PORT)
                } else if MONGODB_PATTERNS.iter().any(|p| image.contains(p)) {
                    ("mongodb", MONGODB_DEFAULT_PORT)
                } else {
                    continue;
                };

                // Find the service that selects this pod
                let matching_service = service_list.items.iter().find(|svc| {
                    if let Some(selector) = svc.spec.as_ref().and_then(|s| s.selector.as_ref()) {
                        if let Some(pod_labels) = labels {
                            selector.iter().all(|(k, v)| pod_labels.get(k) == Some(v))
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                });

                let service_name = matching_service
                    .and_then(|s| s.metadata.name.clone())
                    .unwrap_or_else(|| pod_name.to_string());

                // Get the port from service or container
                let port = matching_service
                    .and_then(|s| s.spec.as_ref())
                    .and_then(|spec| spec.ports.as_ref())
                    .and_then(|ports| ports.first())
                    .map(|p| p.port)
                    .unwrap_or(default_port);

                // Try to extract credentials from environment variables
                let (username, password, database) =
                    self.extract_credentials(container, &secrets_map, service_type);

                // Build service host (cluster DNS name)
                let host = format!("{}.{}.svc.cluster.local", service_name, namespace);

                discovered.push(DiscoveredService {
                    name: format!("{}/{}", namespace, service_name),
                    namespace: namespace.to_string(),
                    service_type: service_type.to_string(),
                    host,
                    port,
                    has_credentials: username.is_some() || password.is_some(),
                    username,
                    password,
                    database,
                    service_name,
                });
            }
        }

        // Deduplicate by namespace/service_name
        let mut seen = std::collections::HashSet::new();
        discovered.retain(|s| seen.insert(format!("{}/{}", s.namespace, s.service_name)));

        Ok(discovered)
    }

    /// Extract credentials from container environment variables
    fn extract_credentials(
        &self,
        container: &k8s_openapi::api::core::v1::Container,
        secrets_map: &HashMap<String, &Secret>,
        service_type: &str,
    ) -> (Option<String>, Option<String>, Option<String>) {
        let mut username = None;
        let mut password = None;
        let mut database = None;

        let env_vars = match &container.env {
            Some(env) => env,
            None => return (None, None, None),
        };

        // Common environment variable names for different databases
        let (user_vars, pass_vars, db_vars) = match service_type {
            "mysql" => (
                vec!["MYSQL_USER", "MYSQL_ROOT_USER", "DB_USER"],
                vec!["MYSQL_PASSWORD", "MYSQL_ROOT_PASSWORD", "DB_PASSWORD"],
                vec!["MYSQL_DATABASE", "DB_NAME"],
            ),
            "redis" => (
                vec![] as Vec<&str>,
                vec!["REDIS_PASSWORD", "REDIS_AUTH"],
                vec![],
            ),
            "mongodb" => (
                vec!["MONGO_INITDB_ROOT_USERNAME", "MONGODB_ROOT_USER"],
                vec!["MONGO_INITDB_ROOT_PASSWORD", "MONGODB_ROOT_PASSWORD"],
                vec!["MONGO_INITDB_DATABASE"],
            ),
            _ => return (None, None, None),
        };

        for env_var in env_vars {
            let name = &env_var.name;
            let value = self.resolve_env_value(env_var, secrets_map);

            if user_vars.contains(&name.as_str()) && username.is_none() {
                username = value.clone();
            }
            if pass_vars.contains(&name.as_str()) && password.is_none() {
                password = value.clone();
            }
            if db_vars.contains(&name.as_str()) && database.is_none() {
                database = value;
            }
        }

        // For MySQL, default to root user if no user specified
        if service_type == "mysql" && username.is_none() && password.is_some() {
            username = Some("root".to_string());
        }

        (username, password, database)
    }

    /// Resolve environment variable value (including secret references)
    fn resolve_env_value(
        &self,
        env_var: &k8s_openapi::api::core::v1::EnvVar,
        secrets_map: &HashMap<String, &Secret>,
    ) -> Option<String> {
        // Direct value
        if let Some(value) = &env_var.value {
            return Some(value.clone());
        }

        // Secret reference
        if let Some(value_from) = &env_var.value_from {
            if let Some(secret_ref) = &value_from.secret_key_ref {
                let secret_name = &secret_ref.name;
                let secret_key = &secret_ref.key;

                if let Some(secret) = secret_name.as_ref().and_then(|n| secrets_map.get(n)) {
                    if let Some(data) = &secret.data {
                        if let Some(value_bytes) = data.get(secret_key) {
                            // Decode base64
                            if let Ok(decoded) = BASE64.decode(&value_bytes.0) {
                                return String::from_utf8(decoded).ok();
                            }
                        }
                    }
                }
            }
        }

        None
    }

    // ==================== K8s Resource List Methods ====================

    /// List all namespace names (public)
    pub async fn get_namespaces(&self) -> AppResult<Vec<String>> {
        self.list_namespaces().await
    }

    /// List deployments in a namespace
    pub async fn list_deployments(&self, namespace: &str) -> AppResult<Vec<K8sDeployment>> {
        let deployments: Api<Deployment> = Api::namespaced(self.client.clone(), namespace);
        let list = deployments
            .list(&ListParams::default())
            .await
            .map_err(|e| AppError::K8s(format!("Failed to list deployments: {}", e)))?;

        Ok(list
            .items
            .into_iter()
            .map(|d| {
                let spec = d.spec.as_ref();
                let status = d.status.as_ref();
                K8sDeployment {
                    name: d.metadata.name.unwrap_or_default(),
                    namespace: d.metadata.namespace.unwrap_or_else(|| namespace.to_string()),
                    replicas: spec.and_then(|s| s.replicas).unwrap_or(0),
                    ready_replicas: status.and_then(|s| s.ready_replicas).unwrap_or(0),
                    available_replicas: status.and_then(|s| s.available_replicas).unwrap_or(0),
                    labels: d.metadata.labels.unwrap_or_default().into_iter().collect(),
                    created_at: d
                        .metadata
                        .creation_timestamp
                        .map(|t| t.0.to_rfc3339()),
                }
            })
            .collect())
    }

    /// List pods in a namespace
    pub async fn list_pods(&self, namespace: &str) -> AppResult<Vec<K8sPod>> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let list = pods
            .list(&ListParams::default())
            .await
            .map_err(|e| AppError::K8s(format!("Failed to list pods: {}", e)))?;

        Ok(list
            .items
            .into_iter()
            .map(|p| {
                let status = p.status.as_ref();
                let container_statuses = status.and_then(|s| s.container_statuses.as_ref());

                // Calculate ready count
                let (ready_count, total_count) = container_statuses
                    .map(|cs| {
                        let ready = cs.iter().filter(|c| c.ready).count();
                        (ready, cs.len())
                    })
                    .unwrap_or((0, 0));

                // Calculate total restarts
                let restarts: i32 = container_statuses
                    .map(|cs| cs.iter().map(|c| c.restart_count).sum())
                    .unwrap_or(0);

                K8sPod {
                    name: p.metadata.name.unwrap_or_default(),
                    namespace: p.metadata.namespace.unwrap_or_else(|| namespace.to_string()),
                    status: status
                        .and_then(|s| s.phase.clone())
                        .unwrap_or_else(|| "Unknown".to_string()),
                    ready: format!("{}/{}", ready_count, total_count),
                    restarts,
                    node: p.spec.as_ref().and_then(|s| s.node_name.clone()),
                    ip: status.and_then(|s| s.pod_ip.clone()),
                    created_at: p.metadata.creation_timestamp.map(|t| t.0.to_rfc3339()),
                }
            })
            .collect())
    }

    /// List ConfigMaps in a namespace (metadata only)
    pub async fn list_configmaps(&self, namespace: &str) -> AppResult<Vec<K8sConfigMapInfo>> {
        let configmaps: Api<ConfigMap> = Api::namespaced(self.client.clone(), namespace);
        let list = configmaps
            .list(&ListParams::default())
            .await
            .map_err(|e| AppError::K8s(format!("Failed to list configmaps: {}", e)))?;

        Ok(list
            .items
            .into_iter()
            .map(|cm| K8sConfigMapInfo {
                name: cm.metadata.name.unwrap_or_default(),
                namespace: cm.metadata.namespace.unwrap_or_else(|| namespace.to_string()),
                data_keys: cm.data.map(|d| d.keys().cloned().collect()).unwrap_or_default(),
                created_at: cm.metadata.creation_timestamp.map(|t| t.0.to_rfc3339()),
            })
            .collect())
    }

    /// Get ConfigMap data by name
    pub async fn get_configmap_data(
        &self,
        namespace: &str,
        name: &str,
    ) -> AppResult<HashMap<String, String>> {
        let configmaps: Api<ConfigMap> = Api::namespaced(self.client.clone(), namespace);
        let cm = configmaps
            .get(name)
            .await
            .map_err(|e| AppError::K8s(format!("Failed to get configmap: {}", e)))?;

        Ok(cm.data.unwrap_or_default().into_iter().collect())
    }

    /// List Secrets in a namespace (metadata only, no values)
    pub async fn list_secrets(&self, namespace: &str) -> AppResult<Vec<K8sSecretInfo>> {
        let secrets: Api<Secret> = Api::namespaced(self.client.clone(), namespace);
        let list = secrets
            .list(&ListParams::default())
            .await
            .map_err(|e| AppError::K8s(format!("Failed to list secrets: {}", e)))?;

        Ok(list
            .items
            .into_iter()
            .map(|s| K8sSecretInfo {
                name: s.metadata.name.unwrap_or_default(),
                namespace: s.metadata.namespace.unwrap_or_else(|| namespace.to_string()),
                secret_type: s.type_.unwrap_or_else(|| "Opaque".to_string()),
                data_keys: s.data.map(|d| d.keys().cloned().collect()).unwrap_or_default(),
                created_at: s.metadata.creation_timestamp.map(|t| t.0.to_rfc3339()),
            })
            .collect())
    }

    /// List Services in a namespace
    pub async fn list_services_info(&self, namespace: &str) -> AppResult<Vec<K8sServiceInfo>> {
        let services: Api<Service> = Api::namespaced(self.client.clone(), namespace);
        let list = services
            .list(&ListParams::default())
            .await
            .map_err(|e| AppError::K8s(format!("Failed to list services: {}", e)))?;

        Ok(list
            .items
            .into_iter()
            .map(|svc| {
                let spec = svc.spec.as_ref();
                let status = svc.status.as_ref();

                // Format ports as "port:nodePort/protocol"
                let ports: Vec<String> = spec
                    .and_then(|s| s.ports.as_ref())
                    .map(|ports| {
                        ports
                            .iter()
                            .map(|p| {
                                let protocol = p.protocol.as_deref().unwrap_or("TCP");
                                if let Some(node_port) = p.node_port {
                                    format!("{}:{}/{}", p.port, node_port, protocol)
                                } else {
                                    format!("{}/{}", p.port, protocol)
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                // Get external IP from LoadBalancer status
                let external_ip = status
                    .and_then(|s| s.load_balancer.as_ref())
                    .and_then(|lb| lb.ingress.as_ref())
                    .and_then(|ingress| ingress.first())
                    .and_then(|ing| ing.ip.clone().or_else(|| ing.hostname.clone()));

                K8sServiceInfo {
                    name: svc.metadata.name.unwrap_or_default(),
                    namespace: svc.metadata.namespace.unwrap_or_else(|| namespace.to_string()),
                    service_type: spec
                        .and_then(|s| s.type_.clone())
                        .unwrap_or_else(|| "ClusterIP".to_string()),
                    cluster_ip: spec.and_then(|s| s.cluster_ip.clone()),
                    external_ip,
                    ports,
                    created_at: svc.metadata.creation_timestamp.map(|t| t.0.to_rfc3339()),
                }
            })
            .collect())
    }

    /// List Ingresses in a namespace
    pub async fn list_ingresses(&self, namespace: &str) -> AppResult<Vec<K8sIngressInfo>> {
        let ingresses: Api<Ingress> = Api::namespaced(self.client.clone(), namespace);
        let list = ingresses
            .list(&ListParams::default())
            .await
            .map_err(|e| AppError::K8s(format!("Failed to list ingresses: {}", e)))?;

        Ok(list
            .items
            .into_iter()
            .map(|ing| {
                let spec = ing.spec.as_ref();
                let status = ing.status.as_ref();

                // Extract hosts from rules
                let hosts: Vec<String> = spec
                    .and_then(|s| s.rules.as_ref())
                    .map(|rules| {
                        rules
                            .iter()
                            .filter_map(|r| r.host.clone())
                            .collect()
                    })
                    .unwrap_or_default();

                // Get address from status
                let address = status
                    .and_then(|s| s.load_balancer.as_ref())
                    .and_then(|lb| lb.ingress.as_ref())
                    .and_then(|ingress| ingress.first())
                    .and_then(|ing| ing.ip.clone().or_else(|| ing.hostname.clone()));

                K8sIngressInfo {
                    name: ing.metadata.name.unwrap_or_default(),
                    namespace: ing.metadata.namespace.unwrap_or_else(|| namespace.to_string()),
                    hosts,
                    address,
                    created_at: ing.metadata.creation_timestamp.map(|t| t.0.to_rfc3339()),
                }
            })
            .collect())
    }
}
