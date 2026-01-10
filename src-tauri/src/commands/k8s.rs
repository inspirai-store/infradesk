//! Tauri commands for Kubernetes operations
//!
//! These commands are exposed to the frontend via IPC.

use tauri::State;

use crate::db::models::{
    Cluster, Connection, DiscoveredService, ImportConnectionsRequest, ImportConnectionsResponse,
    ImportConnectionResult, ListClustersResponse,
};
use crate::db::SqlitePool;
use crate::error::AppError;
use crate::services::{ClusterService, K8sService};

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

        if let Some(existing_conn) = existing {
            if request.force_override {
                // Update existing connection
                let mut updated_conn = conn.clone();
                updated_conn.id = existing_conn.id;
                updated_conn.is_default = existing_conn.is_default;

                match pool
                    .inner()
                    .update_connection(existing_conn.id.unwrap(), &updated_conn)
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
            // Create new connection
            match pool.inner().create_connection(&conn).await {
                Ok(created) => {
                    // Store password in keyring if provided
                    if let (Some(id), Some(password)) = (created.id, &svc.password) {
                        let _ = crate::services::keyring::KeyringService::save_password(id, password);
                    }

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
