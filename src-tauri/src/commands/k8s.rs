//! Tauri commands for Kubernetes operations
//!
//! These commands are exposed to the frontend via IPC.

use tauri::State;

use std::collections::HashMap;

use crate::db::models::{
    Cluster, Connection, DiscoveredService, ImportConnectionsRequest, ImportConnectionsResponse,
    ImportConnectionResult, K8sConfigMapInfo, K8sDeployment, K8sIngressInfo, K8sPod,
    K8sSecretInfo, K8sServiceInfo, ListClustersResponse,
};
use crate::db::SqlitePool;
use crate::error::AppError;
use crate::services::{ClusterService, ConnectionService, K8sService};

/// Discover database services in a K8s cluster
#[tauri::command]
pub async fn k8s_discover(
    pool: State<'_, SqlitePool>,
    kubeconfig: Option<String>,
    context: Option<String>,
) -> Result<Vec<DiscoveredService>, AppError> {
    // Get kubeconfig from parameter or try to use cluster config
    let k8s = if let Some(kc) = kubeconfig {
        K8sService::from_kubeconfig(&kc, context.as_deref()).await?
    } else {
        // Try in-cluster config
        K8sService::in_cluster().await.map_err(|_| {
            AppError::K8s(
                "No kubeconfig provided and not running in a K8s cluster".to_string(),
            )
        })?
    };

    let discovered = k8s.discover_services().await?;

    // Filter out already existing connections
    let existing_conns = pool.inner().get_all_connections().await.unwrap_or_default();
    let existing_k8s_services: std::collections::HashSet<String> = existing_conns
        .iter()
        .filter(|c| c.k8s_namespace.is_some() && c.k8s_service_name.is_some())
        .map(|c| {
            format!(
                "{}/{}",
                c.k8s_namespace.as_deref().unwrap_or(""),
                c.k8s_service_name.as_deref().unwrap_or("")
            )
        })
        .collect();

    let filtered: Vec<DiscoveredService> = discovered
        .into_iter()
        .filter(|s| !existing_k8s_services.contains(&format!("{}/{}", s.namespace, s.service_name)))
        .collect();

    Ok(filtered)
}

/// List clusters/contexts from a kubeconfig
#[tauri::command]
pub async fn k8s_list_clusters(kubeconfig: String) -> Result<ListClustersResponse, AppError> {
    K8sService::list_contexts_from_kubeconfig(&kubeconfig)
}

/// Import discovered services as connections
#[tauri::command]
pub async fn k8s_import_connections(
    pool: State<'_, SqlitePool>,
    request: ImportConnectionsRequest,
) -> Result<ImportConnectionsResponse, AppError> {
    let mut response = ImportConnectionsResponse {
        success: 0,
        failed: 0,
        updated: 0,
        skipped: 0,
        results: Vec::new(),
    };

    // If cluster name is provided, get or create cluster
    let cluster_id = if let Some(cluster_name) = &request.cluster_name {
        let cluster_service = ClusterService::new(pool.inner().clone());
        match cluster_service.get_by_name(cluster_name).await {
            Ok(cluster) => cluster.id,
            Err(_) => {
                // Create new cluster
                let new_cluster = Cluster {
                    id: None,
                    name: cluster_name.clone(),
                    context: request.context.clone(),
                    environment: Some("unknown".to_string()),
                    is_active: true,
                    kubeconfig: request.kubeconfig.clone(),
                    created_at: None,
                    updated_at: None,
                };

                match pool.inner().create_cluster(&new_cluster).await {
                    Ok(created) => created.id,
                    Err(e) => {
                        log::warn!("Failed to create cluster: {}", e);
                        None
                    }
                }
            }
        }
    } else {
        None
    };

    // Import each service
    for svc in &request.services {
        let mut result = ImportConnectionResult {
            name: svc.name.clone(),
            success: false,
            updated: None,
            skipped: None,
            error: None,
            id: None,
        };

        // Build connection name
        let conn_name = if !svc.namespace.is_empty() {
            format!("{}/{}", svc.namespace, svc.name)
        } else {
            svc.name.clone()
        };

        let service_name = svc.service_name.clone().unwrap_or_else(|| svc.name.clone());

        // Create connection
        let conn = Connection {
            id: None,
            name: conn_name.clone(),
            conn_type: svc.service_type.clone(),
            host: "localhost".to_string(), // Will be updated when port forward starts
            port: 0,                         // Will be assigned by port forward
            username: svc.username.clone(),
            password: svc.password.clone(),
            database_name: svc.database.clone(),
            is_default: false,
            source: Some("k8s".to_string()),
            k8s_namespace: Some(svc.namespace.clone()),
            k8s_service_name: Some(service_name),
            k8s_service_port: Some(svc.port),
            cluster_id,
            forward_local_port: None, // User can set preferred port later
            created_at: None,
            updated_at: None,
        };

        // Check for existing connection
        let existing = pool
            .inner()
            .get_connections_by_type(&svc.service_type)
            .await
            .unwrap_or_default()
            .into_iter()
            .find(|c| {
                c.k8s_namespace == conn.k8s_namespace
                    && c.k8s_service_name == conn.k8s_service_name
            });

        // Use ConnectionService to properly handle password storage in keyring
        let conn_service = ConnectionService::new(pool.inner().clone());

        if let Some(existing_conn) = existing {
            if request.force_override {
                // Update existing connection
                let mut updated_conn = conn.clone();
                updated_conn.id = existing_conn.id;
                updated_conn.is_default = existing_conn.is_default;
                // Restore password for update
                updated_conn.password = svc.password.clone();

                match conn_service
                    .update(existing_conn.id.unwrap(), updated_conn)
                    .await
                {
                    Ok(updated) => {
                        result.success = true;
                        result.updated = Some(true);
                        result.id = updated.id;
                        response.success += 1;
                        response.updated += 1;
                    }
                    Err(e) => {
                        result.error = Some(e.to_string());
                        response.failed += 1;
                    }
                }
            } else {
                // Skip existing
                result.skipped = Some(true);
                result.error = Some("Connection already exists".to_string());
                response.skipped += 1;
            }
        } else {
            // Create new connection using ConnectionService (handles keyring properly)
            let mut new_conn = conn.clone();
            new_conn.password = svc.password.clone();

            match conn_service.create(new_conn).await {
                Ok(created) => {
                    result.success = true;
                    result.id = created.id;
                    response.success += 1;
                }
                Err(e) => {
                    result.error = Some(e.to_string());
                    response.failed += 1;
                }
            }
        }

        response.results.push(result);
    }

    Ok(response)
}

// ==================== K8s Resource Commands ====================

/// Helper to get K8sService from cluster_id
async fn get_k8s_service(pool: &SqlitePool, cluster_id: i64) -> Result<K8sService, AppError> {
    let cluster_service = ClusterService::new(pool.clone());
    let cluster = cluster_service.get_by_id(cluster_id).await?;

    let kubeconfig = cluster.kubeconfig.ok_or_else(|| {
        AppError::K8s("Cluster has no kubeconfig".to_string())
    })?;

    K8sService::from_kubeconfig(&kubeconfig, cluster.context.as_deref()).await
}

/// List all namespaces in a cluster
#[tauri::command]
pub async fn k8s_list_namespaces(
    pool: State<'_, SqlitePool>,
    cluster_id: i64,
) -> Result<Vec<String>, AppError> {
    let k8s = get_k8s_service(pool.inner(), cluster_id).await?;
    k8s.get_namespaces().await
}

/// List deployments in a namespace
#[tauri::command]
pub async fn k8s_list_deployments(
    pool: State<'_, SqlitePool>,
    cluster_id: i64,
    namespace: String,
) -> Result<Vec<K8sDeployment>, AppError> {
    let k8s = get_k8s_service(pool.inner(), cluster_id).await?;
    k8s.list_deployments(&namespace).await
}

/// List pods in a namespace
#[tauri::command]
pub async fn k8s_list_pods(
    pool: State<'_, SqlitePool>,
    cluster_id: i64,
    namespace: String,
) -> Result<Vec<K8sPod>, AppError> {
    let k8s = get_k8s_service(pool.inner(), cluster_id).await?;
    k8s.list_pods(&namespace).await
}

/// List ConfigMaps in a namespace
#[tauri::command]
pub async fn k8s_list_configmaps(
    pool: State<'_, SqlitePool>,
    cluster_id: i64,
    namespace: String,
) -> Result<Vec<K8sConfigMapInfo>, AppError> {
    let k8s = get_k8s_service(pool.inner(), cluster_id).await?;
    k8s.list_configmaps(&namespace).await
}

/// Get ConfigMap data
#[tauri::command]
pub async fn k8s_get_configmap_data(
    pool: State<'_, SqlitePool>,
    cluster_id: i64,
    namespace: String,
    name: String,
) -> Result<HashMap<String, String>, AppError> {
    let k8s = get_k8s_service(pool.inner(), cluster_id).await?;
    k8s.get_configmap_data(&namespace, &name).await
}

/// List Secrets in a namespace (metadata only)
#[tauri::command]
pub async fn k8s_list_secrets(
    pool: State<'_, SqlitePool>,
    cluster_id: i64,
    namespace: String,
) -> Result<Vec<K8sSecretInfo>, AppError> {
    let k8s = get_k8s_service(pool.inner(), cluster_id).await?;
    k8s.list_secrets(&namespace).await
}

/// List Services in a namespace
#[tauri::command]
pub async fn k8s_list_services(
    pool: State<'_, SqlitePool>,
    cluster_id: i64,
    namespace: String,
) -> Result<Vec<K8sServiceInfo>, AppError> {
    let k8s = get_k8s_service(pool.inner(), cluster_id).await?;
    k8s.list_services_info(&namespace).await
}

/// List Ingresses in a namespace
#[tauri::command]
pub async fn k8s_list_ingresses(
    pool: State<'_, SqlitePool>,
    cluster_id: i64,
    namespace: String,
) -> Result<Vec<K8sIngressInfo>, AppError> {
    let k8s = get_k8s_service(pool.inner(), cluster_id).await?;
    k8s.list_ingresses(&namespace).await
}
