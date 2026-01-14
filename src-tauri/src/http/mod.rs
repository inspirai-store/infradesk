//! HTTP server module for standalone web mode
//!
//! This module provides HTTP API endpoints that mirror the Tauri IPC commands,
//! allowing the frontend to work in pure web mode without Tauri.

use std::net::TcpListener;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use k8s_openapi::api::core::v1::Pod;
use kube::config::{KubeConfigOptions, Kubeconfig};
use kube::{Api, Client, Config};
use serde::Deserialize;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use crate::db::models::{
    AddQueryHistoryRequest, AlterTableRequest, AlterUserPasswordRequest, Cluster, Connection,
    CopyTableRequest, CreateDatabaseRequest, CreateForeignKeyRequest, CreateIndexRequest,
    CreateSavedQueryRequest, CreateTableRequest, CreateUserRequest, CreateViewRequest,
    DiscoveredService, DropUserRequest, ExplainResult, ExportTableRequest, ExportTableResponse,
    ForeignKeyInfo, GrantPrivilegesRequest, ImportConnectionResult, ImportConnectionsRequest,
    ImportConnectionsResponse, ImportDataRequest, ImportResult, IndexInfo, ListClustersResponse,
    MysqlDatabase, MysqlQueryResult, MysqlServerInfo, MysqlTable, MysqlTableData, MysqlTableSchema,
    MysqlUserInfo, PortForward, ProcedureDefinition, ProcedureInfo, ProcessInfo, QueryHistory,
    QueryHistoryListResponse, RedisKeyListResponse, RedisKeyValue, RedisServerInfo, RenameTableRequest,
    RevokePrivilegesRequest, SavedQuery, ServerVariable, SetKeyRequest, TableMaintenanceResult,
    TestConnectionRequest, TestConnectionResult, TestK8sConnectionRequest, TriggerDefinition,
    TriggerInfo, UpdateConnectionRequest, UpdateSavedQueryRequest, UserGrantsResponse,
    ViewDefinition, ViewInfo,
};
use crate::db::SqlitePool;
use crate::error::AppError;
use crate::services::{
    ClusterService, ConnectionService, K8sService, MysqlService, PortForwardService, RedisService,
};

/// Application state shared across all routes
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub port_forward_service: Arc<RwLock<PortForwardService>>,
}

/// Create the HTTP router with all API routes
pub fn create_router(pool: SqlitePool, pf_service: PortForwardService) -> Router {
    let state = Arc::new(AppState {
        pool,
        port_forward_service: Arc::new(RwLock::new(pf_service)),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Connection routes
        .route("/api/connections", get(get_all_connections))
        .route("/api/connections", post(create_connection))
        .route("/api/connections/:id", get(get_connection))
        .route("/api/connections/:id", put(update_connection))
        .route("/api/connections/:id", delete(delete_connection))
        .route("/api/connections/test", post(test_connection))
        .route("/api/connections/test-k8s", post(test_k8s_connection))
        .route("/api/connections/type/:conn_type", get(get_connections_by_type))
        // Cluster routes
        .route("/api/clusters", get(get_all_clusters))
        .route("/api/clusters", post(create_cluster))
        .route("/api/clusters/:id", get(get_cluster))
        .route("/api/clusters/:id", put(update_cluster))
        .route("/api/clusters/:id", delete(delete_cluster))
        // K8s routes
        .route("/api/k8s/clusters", post(k8s_list_clusters))
        .route("/api/k8s/local-kubeconfig", get(k8s_read_local_kubeconfig_http))
        .route("/api/k8s/discover", post(k8s_discover))
        .route("/api/k8s/import", post(k8s_import_connections))
        // K8s resource listing routes
        .route("/api/k8s/clusters/:cluster_id/namespaces", get(k8s_list_namespaces_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/deployments", get(k8s_list_deployments_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/pods", get(k8s_list_pods_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/pods/:name", get(k8s_get_pod_detail_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/pods/:name/logs", get(k8s_get_pod_logs_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/configmaps", get(k8s_list_configmaps_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/configmaps/:name", get(k8s_get_configmap_data_http).put(k8s_update_configmap_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/secrets", get(k8s_list_secrets_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/secrets/:name", get(k8s_get_secret_data_http).put(k8s_update_secret_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/services", get(k8s_list_services_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/ingresses", get(k8s_list_ingresses_http))
        // Extended workload types
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/jobs", get(k8s_list_jobs_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/cronjobs", get(k8s_list_cronjobs_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/statefulsets", get(k8s_list_statefulsets_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/daemonsets", get(k8s_list_daemonsets_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/replicasets", get(k8s_list_replicasets_http))
        // Deployment operations
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/deployments/:name/yaml", get(k8s_get_deployment_yaml_http).put(k8s_update_deployment_yaml_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/deployments/:name/scale", post(k8s_scale_deployment_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/deployments/:name/restart", post(k8s_restart_deployment_http))
        // Proxy operations
        .route("/api/k8s/clusters/:cluster_id/proxies", get(k8s_list_all_proxies_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/proxies", get(k8s_list_proxies_http).post(k8s_create_proxy_http))
        .route("/api/k8s/clusters/:cluster_id/namespaces/:namespace/proxies/:name", delete(k8s_delete_proxy_http))
        // Port forward routes
        .route("/api/port-forward", get(list_port_forwards))
        .route("/api/port-forward/start", post(start_port_forward))
        .route("/api/port-forward/:id", get(get_port_forward))
        .route("/api/port-forward/:id/stop", post(stop_port_forward))
        .route("/api/port-forward/:id/reconnect", post(reconnect_port_forward))
        .route("/api/port-forward/:id/touch", post(touch_port_forward))
        .route("/api/port-forward/connection/:connection_id", get(get_port_forward_by_connection))
        // MySQL routes
        .route("/api/mysql/info", get(mysql_get_info))
        .route("/api/mysql/databases", get(mysql_list_databases))
        .route("/api/mysql/databases", post(mysql_create_database))
        .route("/api/mysql/databases/:name", delete(mysql_drop_database))
        .route("/api/mysql/databases/:db/tables", get(mysql_list_tables))
        .route("/api/mysql/databases/:db/tables/:table", delete(mysql_drop_table))
        .route("/api/mysql/databases/:db/tables/:table/schema", get(mysql_get_table_schema))
        .route("/api/mysql/databases/:db/tables/:table/rows", get(mysql_get_rows))
        .route("/api/mysql/databases/:db/tables/:table/rows", post(mysql_insert_row))
        .route("/api/mysql/databases/:db/tables/:table/rows", put(mysql_update_record))
        .route("/api/mysql/databases/:db/tables/:table/rows", delete(mysql_delete_row))
        .route("/api/mysql/query", post(mysql_execute_query))
        // MySQL table management routes
        .route("/api/mysql/databases/:db/tables", post(mysql_create_table))
        .route("/api/mysql/databases/:db/tables/:table", put(mysql_alter_table))
        .route("/api/mysql/databases/:db/tables/:table/rename", post(mysql_rename_table))
        .route("/api/mysql/databases/:db/tables/:table/truncate", post(mysql_truncate_table))
        .route("/api/mysql/databases/:db/tables/:table/copy", post(mysql_copy_table))
        // MySQL index management routes
        .route("/api/mysql/databases/:db/tables/:table/indexes", get(mysql_list_indexes))
        .route("/api/mysql/databases/:db/tables/:table/indexes", post(mysql_create_index))
        .route("/api/mysql/databases/:db/tables/:table/indexes/:index", delete(mysql_drop_index))
        // MySQL foreign key management routes
        .route("/api/mysql/databases/:db/tables/:table/foreign-keys", get(mysql_list_foreign_keys))
        .route("/api/mysql/databases/:db/tables/:table/foreign-keys", post(mysql_create_foreign_key))
        .route("/api/mysql/databases/:db/tables/:table/foreign-keys/:fk", delete(mysql_drop_foreign_key))
        // MySQL data export/import routes
        .route("/api/mysql/databases/:db/tables/:table/export", post(mysql_export_table))
        .route("/api/mysql/databases/:db/tables/:table/import", post(mysql_import_data))
        // MySQL user management routes
        .route("/api/mysql/users", get(mysql_list_users))
        .route("/api/mysql/users", post(mysql_create_user))
        .route("/api/mysql/users/password", put(mysql_alter_user_password))
        .route("/api/mysql/users/drop", post(mysql_drop_user))
        .route("/api/mysql/users/grants", get(mysql_show_grants))
        .route("/api/mysql/users/grant", post(mysql_grant_privileges))
        .route("/api/mysql/users/revoke", post(mysql_revoke_privileges))
        // MySQL view management routes
        .route("/api/mysql/databases/:db/views", get(mysql_list_views))
        .route("/api/mysql/databases/:db/views", post(mysql_create_view))
        .route("/api/mysql/databases/:db/views/:view", get(mysql_get_view_definition))
        .route("/api/mysql/databases/:db/views/:view", delete(mysql_drop_view))
        // MySQL stored procedure management routes
        .route("/api/mysql/databases/:db/procedures", get(mysql_list_procedures))
        .route("/api/mysql/databases/:db/procedures/:name", get(mysql_get_procedure_definition))
        .route("/api/mysql/databases/:db/procedures/:name", delete(mysql_drop_procedure))
        .route("/api/mysql/databases/:db/functions/:name", delete(mysql_drop_function))
        // MySQL trigger management routes
        .route("/api/mysql/databases/:db/triggers", get(mysql_list_triggers))
        .route("/api/mysql/databases/:db/triggers/:name", get(mysql_get_trigger_definition))
        .route("/api/mysql/databases/:db/triggers/:name", delete(mysql_drop_trigger))
        // MySQL server monitoring routes
        .route("/api/mysql/server/variables", get(mysql_get_server_variables))
        .route("/api/mysql/server/processes", get(mysql_get_process_list))
        .route("/api/mysql/server/processes/:id", delete(mysql_kill_process))
        // MySQL query analysis routes
        .route("/api/mysql/explain", post(mysql_explain_query))
        // MySQL table maintenance routes
        .route("/api/mysql/databases/:db/tables/:table/optimize", post(mysql_optimize_table))
        .route("/api/mysql/databases/:db/tables/:table/analyze", post(mysql_analyze_table))
        .route("/api/mysql/databases/:db/tables/:table/check", post(mysql_check_table))
        // Redis routes
        .route("/api/redis/info", get(redis_get_info))
        .route("/api/redis/keys", get(redis_list_keys))
        .route("/api/redis/keys/:key", get(redis_get_key))
        .route("/api/redis/keys", post(redis_set_key))
        .route("/api/redis/keys/:key", delete(redis_delete_key))
        .route("/api/redis/keys/:key/ttl", put(redis_set_ttl))
        // History routes
        .route("/api/history", get(get_history))
        .route("/api/history", post(add_history))
        .route("/api/history/:id", delete(delete_history))
        .route("/api/history/cleanup", post(cleanup_history))
        // Saved query routes
        .route("/api/saved-queries", get(get_saved_queries))
        .route("/api/saved-queries", post(create_saved_query))
        .route("/api/saved-queries/:id", get(get_saved_query))
        .route("/api/saved-queries/:id", put(update_saved_query))
        .route("/api/saved-queries/:id", delete(delete_saved_query))
        // Settings routes
        .route("/api/settings", get(get_all_settings_http))
        .route("/api/settings/batch", post(get_settings_batch_http))
        .route("/api/settings/:key", get(get_setting_http))
        .route("/api/settings/:key", put(set_setting_http))
        .route("/api/settings/:key", delete(delete_setting_http))
        // LLM config routes
        .route("/api/llm-configs", get(get_all_llm_configs_http))
        .route("/api/llm-configs", post(create_llm_config_http))
        .route("/api/llm-configs/default", get(get_default_llm_config_http))
        .route("/api/llm-configs/:id", get(get_llm_config_http))
        .route("/api/llm-configs/:id", put(update_llm_config_http))
        .route("/api/llm-configs/:id", delete(delete_llm_config_http))
        .route("/api/llm-configs/:id/default", put(set_default_llm_config_http))
        .route("/api/llm-configs/:id/api-key", get(get_llm_api_key_http))
        .layer(cors)
        .with_state(state)
}

// ==================== Error handling ====================

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(serde_json::json!({
            "error": message
        }));

        (status, body).into_response()
    }
}

// ==================== Helper functions ====================

/// Extract connection_id from query parameter or X-Connection-ID header
fn extract_connection_id(
    query_id: Option<i64>,
    headers: &HeaderMap,
) -> Result<i64, AppError> {
    // Prefer query parameter
    if let Some(id) = query_id {
        return Ok(id);
    }

    // Fall back to header
    headers
        .get("X-Connection-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| AppError::Validation(
            "connection_id is required (via query param or X-Connection-ID header)".to_string()
        ))
}

/// Ensure K8s connection has an active port forward
/// For non-K8s connections, return the original connection unchanged
async fn ensure_port_forward_for_http(
    state: &AppState,
    mut conn: Connection,
) -> Result<Connection, AppError> {
    // Only handle K8s connections
    if conn.source.as_deref() != Some("k8s") {
        return Ok(conn);
    }

    let connection_id = conn.id.ok_or_else(||
        AppError::Validation("Connection ID is required".to_string())
    )?;

    let pf_service = state.port_forward_service.read().await;

    // Try to get existing forward
    let pf = match pf_service.get_by_connection(connection_id).await {
        Ok(existing) if existing.status == "active" => {
            // Verify port is actually listening
            if is_port_listening(existing.local_port as u16).await {
                log::info!("Using existing port forward on port {}", existing.local_port);
                existing
            } else {
                // Port not available, restart
                log::info!("Port {} not listening, restarting forward", existing.local_port);
                drop(pf_service);
                let pf_service = state.port_forward_service.write().await;
                let local_port = conn.forward_local_port.filter(|&p| p > 0).map(|p| p as u16);
                pf_service.start(connection_id, local_port).await?
            }
        }
        Ok(existing) => {
            // Exists but not active, try reconnect
            log::info!("Port forward exists but not active (status: {}), reconnecting", existing.status);
            drop(pf_service);
            let pf_service = state.port_forward_service.write().await;
            let local_port = conn.forward_local_port.filter(|&p| p > 0).map(|p| p as u16);
            pf_service.reconnect(&existing.id.unwrap_or_default(), local_port).await?
        }
        Err(e) => {
            // Does not exist, create new forward
            log::info!("No port forward found for connection {} (err: {}), starting new one", connection_id, e);
            drop(pf_service);
            let pf_service = state.port_forward_service.write().await;
            let local_port = conn.forward_local_port.filter(|&p| p > 0).map(|p| p as u16);
            pf_service.start(connection_id, local_port).await?
        }
    };

    let port = pf.local_port as u16;

    // Wait for port to become available (max 10 seconds)
    let max_wait = Duration::from_secs(10);
    let check_interval = Duration::from_millis(200);
    let start = std::time::Instant::now();

    log::info!("Waiting for port {} to become available...", port);

    while start.elapsed() < max_wait {
        if is_port_listening(port).await {
            log::info!("Port {} is now listening after {:?}", port, start.elapsed());
            break;
        }
        tokio::time::sleep(check_interval).await;
    }

    if !is_port_listening(port).await {
        return Err(AppError::PortForward(format!(
            "Port forward on {} did not become available within {:?}",
            port, max_wait
        )));
    }

    // Update forward_local_port in database if changed
    if conn.forward_local_port != Some(pf.local_port) {
        let conn_service = ConnectionService::new(state.pool.clone());
        conn_service.update_forward_port(connection_id, pf.local_port).await?;
        conn.forward_local_port = Some(pf.local_port);
        log::info!("Updated forward_local_port to {} for connection {}", pf.local_port, connection_id);
    }

    Ok(conn)
}

/// Check if a port is listening
async fn is_port_listening(port: u16) -> bool {
    tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
        .await
        .is_ok()
}

// ==================== Connection handlers ====================

async fn get_all_connections(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Connection>>, AppError> {
    let service = ConnectionService::new(state.pool.clone());
    let connections = service.get_all().await?;
    Ok(Json(connections))
}

async fn get_connection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Connection>, AppError> {
    let service = ConnectionService::new(state.pool.clone());
    let connection = service.get_by_id(id).await?;
    Ok(Json(connection))
}

async fn get_connections_by_type(
    State(state): State<Arc<AppState>>,
    Path(conn_type): Path<String>,
) -> Result<Json<Vec<Connection>>, AppError> {
    let service = ConnectionService::new(state.pool.clone());
    let connections = service.get_by_type(&conn_type).await?;
    Ok(Json(connections))
}

async fn create_connection(
    State(state): State<Arc<AppState>>,
    Json(data): Json<Connection>,
) -> Result<Json<Connection>, AppError> {
    let service = ConnectionService::new(state.pool.clone());
    let connection = service.create(data).await?;
    Ok(Json(connection))
}

async fn update_connection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(data): Json<UpdateConnectionRequest>,
) -> Result<Json<Connection>, AppError> {
    let service = ConnectionService::new(state.pool.clone());
    // Use partial_update to only update provided fields
    let connection = service.partial_update(id, data).await?;
    Ok(Json(connection))
}

async fn delete_connection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let service = ConnectionService::new(state.pool.clone());
    service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn test_connection(
    State(state): State<Arc<AppState>>,
    Json(data): Json<TestConnectionRequest>,
) -> Result<Json<TestConnectionResult>, AppError> {
    let service = ConnectionService::new(state.pool.clone());
    // Convert TestConnectionRequest to Connection for testing
    let conn = data.to_connection();
    let result = service.test(&conn).await?;
    Ok(Json(result))
}

async fn test_k8s_connection(
    State(state): State<Arc<AppState>>,
    Json(data): Json<TestK8sConnectionRequest>,
) -> Result<Json<TestConnectionResult>, AppError> {
    // Get kubeconfig: from request, or look up from cluster if cluster_id is provided
    let (kubeconfig_content, context) = if let Some(kc) = &data.kubeconfig {
        (kc.clone(), data.context.clone())
    } else if let Some(cluster_id) = data.cluster_id {
        log::info!("Looking up kubeconfig for cluster ID: {}", cluster_id);
        let cluster = state.pool.get_cluster(cluster_id).await?;
        let kc = cluster.kubeconfig.ok_or_else(|| {
            AppError::K8s(format!(
                "No kubeconfig found for cluster '{}'. Please re-upload the kubeconfig file.",
                cluster.name
            ))
        })?;
        let ctx = data.context.clone().or(cluster.context);
        (kc, ctx)
    } else {
        return Err(AppError::K8s(
            "No kubeconfig provided and no cluster_id to look up. Please upload a kubeconfig file.".to_string()
        ));
    };

    // Create K8s client from kubeconfig
    let kubeconfig = Kubeconfig::from_yaml(&kubeconfig_content)
        .map_err(|e| AppError::K8s(format!("Failed to parse kubeconfig: {}", e)))?;

    let options = KubeConfigOptions {
        context: context.clone(),
        ..Default::default()
    };

    let config = Config::from_custom_kubeconfig(kubeconfig, &options)
        .await
        .map_err(|e| AppError::K8s(format!("Failed to create K8s config: {}", e)))?;

    let client = Client::try_from(config)
        .map_err(|e| AppError::K8s(format!("Failed to create K8s client: {}", e)))?;

    // Find a pod for the service
    let pod_name = find_pod_for_service(&client, &data.k8s_namespace, &data.k8s_service_name).await?;

    // Find an available local port
    let local_port = find_available_port()?;

    log::info!("Testing K8s connection via port forward: localhost:{} -> {}:{}",
        local_port, data.k8s_service_name, data.k8s_service_port);

    // Create port forward
    let pods: Api<Pod> = Api::namespaced(client.clone(), &data.k8s_namespace);
    let mut pf = pods.portforward(&pod_name, &[data.k8s_service_port as u16])
        .await
        .map_err(|e| AppError::K8s(format!("Failed to create port forward: {}", e)))?;

    // Get the port stream
    let upstream = pf.take_stream(data.k8s_service_port as u16)
        .ok_or_else(|| AppError::K8s("Failed to get port stream".to_string()))?;

    // Spawn a task to handle the port forward connection
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", local_port))
        .await
        .map_err(|e| AppError::PortForward(format!("Failed to bind to port {}: {}", local_port, e)))?;

    // Use a channel to signal when to stop the port forward
    let (stop_tx, mut stop_rx) = tokio::sync::oneshot::channel::<()>();

    // Spawn the port forward handler
    let pf_handle = tokio::spawn(async move {
        tokio::select! {
            _ = async {
                if let Ok((mut local_stream, _)) = listener.accept().await {
                    let (mut upstream_read, mut upstream_write) = tokio::io::split(upstream);
                    let (mut local_read, mut local_write) = local_stream.split();

                    tokio::select! {
                        _ = tokio::io::copy(&mut local_read, &mut upstream_write) => {}
                        _ = tokio::io::copy(&mut upstream_read, &mut local_write) => {}
                    }
                }
            } => {}
            _ = &mut stop_rx => {}
        }
    });

    // Give the port forward a moment to initialize
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test the connection through the forwarded port
    let result = test_connection_through_port(&data, local_port).await;

    // Stop the port forward
    let _ = stop_tx.send(());
    let _ = pf_handle.await;

    result.map(Json)
}

// ==================== Cluster handlers ====================

async fn get_all_clusters(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Cluster>>, AppError> {
    let service = ClusterService::new(state.pool.clone());
    let clusters = service.get_all().await?;
    Ok(Json(clusters))
}

async fn get_cluster(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Cluster>, AppError> {
    let service = ClusterService::new(state.pool.clone());
    let cluster = service.get_by_id(id).await?;
    Ok(Json(cluster))
}

async fn create_cluster(
    State(state): State<Arc<AppState>>,
    Json(data): Json<Cluster>,
) -> Result<Json<Cluster>, AppError> {
    let service = ClusterService::new(state.pool.clone());
    let cluster = service.create(&data).await?;
    Ok(Json(cluster))
}

async fn update_cluster(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(data): Json<Cluster>,
) -> Result<Json<Cluster>, AppError> {
    let service = ClusterService::new(state.pool.clone());
    let cluster = service.update(id, &data).await?;
    Ok(Json(cluster))
}

async fn delete_cluster(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let service = ClusterService::new(state.pool.clone());
    service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ==================== K8s handlers ====================

#[derive(Deserialize)]
struct K8sListClustersRequest {
    kubeconfig: String,
}

#[derive(Deserialize)]
struct K8sDiscoverRequest {
    kubeconfig: Option<String>,
    context: Option<String>,
}

async fn k8s_list_clusters(
    Json(req): Json<K8sListClustersRequest>,
) -> Result<Json<ListClustersResponse>, AppError> {
    let response = K8sService::list_contexts_from_kubeconfig(&req.kubeconfig)?;
    Ok(Json(response))
}

/// Read local kubeconfig file (~/.kube/config)
async fn k8s_read_local_kubeconfig_http() -> Result<Json<String>, AppError> {
    use std::path::PathBuf;

    // Get kubeconfig path from KUBECONFIG env or default to ~/.kube/config
    let kubeconfig_path = std::env::var("KUBECONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".kube")
                .join("config")
        });

    if !kubeconfig_path.exists() {
        return Err(AppError::K8s(format!(
            "Kubeconfig file not found at: {}",
            kubeconfig_path.display()
        )));
    }

    let content = std::fs::read_to_string(&kubeconfig_path).map_err(|e| {
        AppError::K8s(format!(
            "Failed to read kubeconfig from {}: {}",
            kubeconfig_path.display(),
            e
        ))
    })?;

    Ok(Json(content))
}

async fn k8s_discover(
    State(state): State<Arc<AppState>>,
    Json(req): Json<K8sDiscoverRequest>,
) -> Result<Json<Vec<DiscoveredService>>, AppError> {
    let k8s = if let Some(kc) = req.kubeconfig {
        K8sService::from_kubeconfig(&kc, req.context.as_deref()).await?
    } else {
        K8sService::in_cluster().await.map_err(|_| {
            AppError::K8s(
                "No kubeconfig provided and not running in a K8s cluster".to_string(),
            )
        })?
    };

    let discovered = k8s.discover_services().await?;

    // Filter out already existing connections
    let existing_conns = state.pool.get_all_connections().await.unwrap_or_default();
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

    Ok(Json(filtered))
}

async fn k8s_import_connections(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ImportConnectionsRequest>,
) -> Result<Json<ImportConnectionsResponse>, AppError> {
    let mut response = ImportConnectionsResponse {
        success: 0,
        failed: 0,
        updated: 0,
        skipped: 0,
        results: Vec::new(),
    };

    // If cluster name is provided, get or create cluster
    let cluster_id = if let Some(cluster_name) = &request.cluster_name {
        let cluster_service = ClusterService::new(state.pool.clone());
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

                match state.pool.create_cluster(&new_cluster).await {
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
            host: "localhost".to_string(),
            port: 0,
            username: svc.username.clone(),
            password: svc.password.clone(),
            database_name: svc.database.clone(),
            is_default: false,
            source: Some("k8s".to_string()),
            k8s_namespace: Some(svc.namespace.clone()),
            k8s_service_name: Some(service_name),
            k8s_service_port: Some(svc.port),
            cluster_id,
            forward_local_port: None,
            created_at: None,
            updated_at: None,
        };

        // Check for existing connection
        let existing = state.pool
            .get_connections_by_type(&svc.service_type)
            .await
            .unwrap_or_default()
            .into_iter()
            .find(|c| {
                c.k8s_namespace == conn.k8s_namespace
                    && c.k8s_service_name == conn.k8s_service_name
            });

        let conn_service = ConnectionService::new(state.pool.clone());

        if let Some(existing_conn) = existing {
            if request.force_override {
                let mut updated_conn = conn.clone();
                updated_conn.id = existing_conn.id;
                updated_conn.is_default = existing_conn.is_default;
                updated_conn.password = svc.password.clone();

                match conn_service.update(existing_conn.id.unwrap(), updated_conn).await {
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
                result.skipped = Some(true);
                result.error = Some("Connection already exists".to_string());
                response.skipped += 1;
            }
        } else {
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

    Ok(Json(response))
}

// ==================== K8s Resource Listing handlers ====================

use crate::db::models::{
    K8sDeployment, K8sPod, K8sConfigMapInfo, K8sSecretInfo, K8sServiceInfo, K8sIngressInfo,
    K8sJob, K8sCronJob, K8sStatefulSet, K8sDaemonSet, K8sReplicaSet,
};

/// Helper to get K8s service from cluster ID
async fn get_k8s_service_from_cluster(pool: &SqlitePool, cluster_id: i64) -> Result<K8sService, AppError> {
    let cluster_service = ClusterService::new(pool.clone());
    // Use get_with_kubeconfig to get the full cluster including kubeconfig
    // (get_by_id clears kubeconfig for security when exposed via API)
    let cluster = cluster_service.get_with_kubeconfig(cluster_id).await?;

    let kubeconfig = cluster.kubeconfig.ok_or_else(|| {
        AppError::K8s("Cluster has no kubeconfig".to_string())
    })?;

    K8sService::from_kubeconfig(&kubeconfig, cluster.context.as_deref()).await
}

async fn k8s_list_namespaces_http(
    State(state): State<Arc<AppState>>,
    Path(cluster_id): Path<i64>,
) -> Result<Json<Vec<String>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let namespaces = k8s.get_namespaces().await?;
    Ok(Json(namespaces))
}

async fn k8s_list_deployments_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace)): Path<(i64, String)>,
) -> Result<Json<Vec<K8sDeployment>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let deployments = k8s.list_deployments(&namespace).await?;
    Ok(Json(deployments))
}

async fn k8s_list_pods_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace)): Path<(i64, String)>,
) -> Result<Json<Vec<K8sPod>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let pods = k8s.list_pods(&namespace).await?;
    Ok(Json(pods))
}

async fn k8s_get_pod_detail_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace, name)): Path<(i64, String, String)>,
) -> Result<Json<crate::db::models::K8sPodDetail>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let detail = k8s.get_pod_detail(&namespace, &name).await?;
    Ok(Json(detail))
}

#[derive(Deserialize)]
struct PodLogsQuery {
    container: Option<String>,
    tail: Option<i64>,
}

async fn k8s_get_pod_logs_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace, name)): Path<(i64, String, String)>,
    Query(query): Query<PodLogsQuery>,
) -> Result<String, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let logs = k8s.get_pod_logs(&namespace, &name, query.container.as_deref(), query.tail).await?;
    Ok(logs)
}

async fn k8s_list_configmaps_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace)): Path<(i64, String)>,
) -> Result<Json<Vec<K8sConfigMapInfo>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let configmaps = k8s.list_configmaps(&namespace).await?;
    Ok(Json(configmaps))
}

async fn k8s_get_configmap_data_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace, name)): Path<(i64, String, String)>,
) -> Result<Json<std::collections::HashMap<String, String>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let data = k8s.get_configmap_data(&namespace, &name).await?;
    Ok(Json(data))
}

async fn k8s_list_secrets_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace)): Path<(i64, String)>,
) -> Result<Json<Vec<K8sSecretInfo>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let secrets = k8s.list_secrets(&namespace).await?;
    Ok(Json(secrets))
}

async fn k8s_list_services_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace)): Path<(i64, String)>,
) -> Result<Json<Vec<K8sServiceInfo>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let services = k8s.list_services_info(&namespace).await?;
    Ok(Json(services))
}

async fn k8s_list_ingresses_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace)): Path<(i64, String)>,
) -> Result<Json<Vec<K8sIngressInfo>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let ingresses = k8s.list_ingresses(&namespace).await?;
    Ok(Json(ingresses))
}

async fn k8s_get_secret_data_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace, name)): Path<(i64, String, String)>,
) -> Result<Json<std::collections::HashMap<String, String>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let data = k8s.get_secret_data(&namespace, &name).await?;
    Ok(Json(data))
}

#[derive(Deserialize)]
struct UpdateSecretRequest {
    data: std::collections::HashMap<String, String>,
}

async fn k8s_update_secret_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace, name)): Path<(i64, String, String)>,
    Json(req): Json<UpdateSecretRequest>,
) -> Result<StatusCode, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    k8s.update_secret(&namespace, &name, req.data).await?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct UpdateConfigMapRequest {
    data: std::collections::HashMap<String, String>,
}

async fn k8s_update_configmap_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace, name)): Path<(i64, String, String)>,
    Json(req): Json<UpdateConfigMapRequest>,
) -> Result<StatusCode, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    k8s.update_configmap(&namespace, &name, req.data).await?;
    Ok(StatusCode::OK)
}

// ==================== MySQL handlers ====================

#[derive(Deserialize)]
struct ConnectionIdQuery {
    connection_id: Option<i64>,
}

#[derive(Deserialize)]
struct MysqlQueryRequest {
    connection_id: i64,
    database: String,
    query: String,
}

#[derive(Deserialize)]
struct MysqlRowsQuery {
    connection_id: Option<i64>,
    page: Option<i32>,
    page_size: Option<i32>,
}

#[derive(Deserialize)]
struct MysqlInsertRequest {
    connection_id: i64,
    data: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct MysqlUpdateRequest {
    connection_id: i64,
    primary_key: String,
    primary_value: serde_json::Value,
    updates: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct MysqlDeleteRequest {
    connection_id: i64,
    where_clause: std::collections::HashMap<String, serde_json::Value>,
}

async fn mysql_get_info(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<Json<MysqlServerInfo>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let info = mysql_service.get_info().await?;
    Ok(Json(info))
}

async fn mysql_list_databases(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<MysqlDatabase>>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let databases = mysql_service.list_databases().await?;
    Ok(Json(databases))
}

async fn mysql_create_database(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Json(data): Json<CreateDatabaseRequest>,
) -> Result<StatusCode, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.create_database(&data).await?;
    Ok(StatusCode::CREATED)
}

async fn mysql_drop_database(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.drop_database(&name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn mysql_list_tables(
    State(state): State<Arc<AppState>>,
    Path(db): Path<String>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<MysqlTable>>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let tables = mysql_service.list_tables(&db).await?;
    Ok(Json(tables))
}

async fn mysql_drop_table(
    State(state): State<Arc<AppState>>,
    Path((db, table)): Path<(String, String)>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.drop_table(&db, &table).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn mysql_get_table_schema(
    State(state): State<Arc<AppState>>,
    Path((db, table)): Path<(String, String)>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<Json<MysqlTableSchema>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let schema = mysql_service.get_table_schema(&db, &table).await?;
    Ok(Json(schema))
}

async fn mysql_get_rows(
    State(state): State<Arc<AppState>>,
    Path((db, table)): Path<(String, String)>,
    Query(params): Query<MysqlRowsQuery>,
    headers: HeaderMap,
) -> Result<Json<MysqlTableData>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(100);
    let data = mysql_service.get_rows(&db, &table, page, page_size).await?;
    Ok(Json(data))
}

async fn mysql_insert_row(
    State(state): State<Arc<AppState>>,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<MysqlInsertRequest>,
) -> Result<Json<u64>, AppError> {
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(req.connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let id = mysql_service.insert_row(&db, &table, &req.data).await?;
    Ok(Json(id))
}

async fn mysql_update_record(
    State(state): State<Arc<AppState>>,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<MysqlUpdateRequest>,
) -> Result<Json<u64>, AppError> {
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(req.connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let affected = mysql_service
        .update_record(&db, &table, &req.primary_key, &req.primary_value, &req.updates)
        .await?;
    Ok(Json(affected))
}

async fn mysql_delete_row(
    State(state): State<Arc<AppState>>,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<MysqlDeleteRequest>,
) -> Result<Json<u64>, AppError> {
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(req.connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let affected = mysql_service.delete_row(&db, &table, &req.where_clause).await?;
    Ok(Json(affected))
}

async fn mysql_execute_query(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MysqlQueryRequest>,
) -> Result<Json<MysqlQueryResult>, AppError> {
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(req.connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let result = mysql_service.execute_query(&req.database, &req.query).await?;
    Ok(Json(result))
}

// ==================== MySQL Table Management Handlers ====================

async fn mysql_create_table(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Path(db): Path<String>,
    Json(req): Json<CreateTableRequest>,
) -> Result<Json<()>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.create_table(&db, &req).await?;
    Ok(Json(()))
}

async fn mysql_alter_table(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<AlterTableRequest>,
) -> Result<Json<()>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.alter_table(&db, &table, &req).await?;
    Ok(Json(()))
}

async fn mysql_rename_table(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<RenameTableRequest>,
) -> Result<Json<()>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.rename_table(&db, &table, &req.new_name).await?;
    Ok(Json(()))
}

async fn mysql_truncate_table(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
) -> Result<Json<()>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.truncate_table(&db, &table).await?;
    Ok(Json(()))
}

async fn mysql_copy_table(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<CopyTableRequest>,
) -> Result<Json<()>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.copy_table(&db, &table, &req.target_name, req.with_data).await?;
    Ok(Json(()))
}

// ==================== MySQL Index handlers ====================

async fn mysql_list_indexes(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
) -> Result<Json<Vec<IndexInfo>>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let indexes = mysql_service.list_indexes(&db, &table).await?;
    Ok(Json(indexes))
}

async fn mysql_create_index(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<CreateIndexRequest>,
) -> Result<Json<()>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.create_index(&db, &table, &req).await?;
    Ok(Json(()))
}

async fn mysql_drop_index(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Path((db, table, index)): Path<(String, String, String)>,
) -> Result<Json<()>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.drop_index(&db, &table, &index).await?;
    Ok(Json(()))
}

// ==================== MySQL Foreign Key handlers ====================

async fn mysql_list_foreign_keys(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
) -> Result<Json<Vec<ForeignKeyInfo>>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let fks = mysql_service.list_foreign_keys(&db, &table).await?;
    Ok(Json(fks))
}

async fn mysql_create_foreign_key(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<CreateForeignKeyRequest>,
) -> Result<Json<()>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.create_foreign_key(&db, &table, &req).await?;
    Ok(Json(()))
}

async fn mysql_drop_foreign_key(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Path((db, table, fk)): Path<(String, String, String)>,
) -> Result<Json<()>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.drop_foreign_key(&db, &table, &fk).await?;
    Ok(Json(()))
}

// ==================== MySQL Data Export/Import handlers ====================

async fn mysql_export_table(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<ExportTableRequest>,
) -> Result<Json<ExportTableResponse>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let response = mysql_service.export_table(&db, &table, &req).await?;
    Ok(Json(response))
}

async fn mysql_import_data(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<ImportDataRequest>,
) -> Result<Json<ImportResult>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let result = mysql_service.import_data(&db, &table, &req).await?;
    Ok(Json(result))
}

// ==================== MySQL User Management handlers ====================

async fn mysql_list_users(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<MysqlUserInfo>>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let users = mysql_service.list_users().await?;
    Ok(Json(users))
}

async fn mysql_create_user(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Json(req): Json<CreateUserRequest>,
) -> Result<StatusCode, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.create_user(&req).await?;
    Ok(StatusCode::CREATED)
}

async fn mysql_alter_user_password(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Json(req): Json<AlterUserPasswordRequest>,
) -> Result<StatusCode, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.alter_user_password(&req).await?;
    Ok(StatusCode::OK)
}

async fn mysql_drop_user(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Json(req): Json<DropUserRequest>,
) -> Result<StatusCode, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.drop_user(&req).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct ShowGrantsQuery {
    connection_id: Option<i64>,
    username: String,
    host: String,
}

async fn mysql_show_grants(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ShowGrantsQuery>,
    headers: HeaderMap,
) -> Result<Json<UserGrantsResponse>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let grants = mysql_service.show_grants(&params.username, &params.host).await?;
    Ok(Json(grants))
}

#[derive(Deserialize)]
struct GrantPrivilegesHttpRequest {
    database: String,
    #[serde(flatten)]
    grant: GrantPrivilegesRequest,
}

async fn mysql_grant_privileges(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Json(req): Json<GrantPrivilegesHttpRequest>,
) -> Result<StatusCode, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.grant_privileges(&req.database, &req.grant).await?;
    Ok(StatusCode::OK)
}

async fn mysql_revoke_privileges(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Json(req): Json<RevokePrivilegesRequest>,
) -> Result<StatusCode, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.revoke_privileges(&req).await?;
    Ok(StatusCode::OK)
}

// ==================== MySQL View handlers ====================

async fn mysql_list_views(
    State(state): State<Arc<AppState>>,
    Path(db): Path<String>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<ViewInfo>>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let views = mysql_service.list_views(&db).await?;
    Ok(Json(views))
}

async fn mysql_get_view_definition(
    State(state): State<Arc<AppState>>,
    Path((db, view)): Path<(String, String)>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<Json<ViewDefinition>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let definition = mysql_service.get_view_definition(&db, &view).await?;
    Ok(Json(definition))
}

async fn mysql_create_view(
    State(state): State<Arc<AppState>>,
    Path(db): Path<String>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
    Json(req): Json<CreateViewRequest>,
) -> Result<StatusCode, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.create_view(&db, &req).await?;
    Ok(StatusCode::CREATED)
}

async fn mysql_drop_view(
    State(state): State<Arc<AppState>>,
    Path((db, view)): Path<(String, String)>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.drop_view(&db, &view).await?;
    Ok(StatusCode::OK)
}

// ==================== MySQL Procedure handlers ====================

#[derive(Deserialize)]
struct ProcedureQuery {
    connection_id: Option<i64>,
    routine_type: Option<String>,
}

async fn mysql_list_procedures(
    State(state): State<Arc<AppState>>,
    Path(db): Path<String>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<ProcedureInfo>>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let procedures = mysql_service.list_procedures(&db).await?;
    Ok(Json(procedures))
}

async fn mysql_get_procedure_definition(
    State(state): State<Arc<AppState>>,
    Path((db, name)): Path<(String, String)>,
    Query(params): Query<ProcedureQuery>,
    headers: HeaderMap,
) -> Result<Json<ProcedureDefinition>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let routine_type = params.routine_type.unwrap_or_else(|| "PROCEDURE".to_string());
    let definition = mysql_service.get_procedure_definition(&db, &name, &routine_type).await?;
    Ok(Json(definition))
}

async fn mysql_drop_procedure(
    State(state): State<Arc<AppState>>,
    Path((db, name)): Path<(String, String)>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.drop_procedure(&db, &name).await?;
    Ok(StatusCode::OK)
}

async fn mysql_drop_function(
    State(state): State<Arc<AppState>>,
    Path((db, name)): Path<(String, String)>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.drop_function(&db, &name).await?;
    Ok(StatusCode::OK)
}

// ==================== MySQL Trigger handlers ====================

async fn mysql_list_triggers(
    State(state): State<Arc<AppState>>,
    Path(db): Path<String>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<TriggerInfo>>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let triggers = mysql_service.list_triggers(&db).await?;
    Ok(Json(triggers))
}

async fn mysql_get_trigger_definition(
    State(state): State<Arc<AppState>>,
    Path((db, name)): Path<(String, String)>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<Json<TriggerDefinition>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let definition = mysql_service.get_trigger_definition(&db, &name).await?;
    Ok(Json(definition))
}

async fn mysql_drop_trigger(
    State(state): State<Arc<AppState>>,
    Path((db, name)): Path<(String, String)>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.drop_trigger(&db, &name).await?;
    Ok(StatusCode::OK)
}

// ==================== MySQL Server Monitoring handlers ====================

#[derive(Deserialize)]
struct ServerVariablesQuery {
    connection_id: Option<i64>,
    filter: Option<String>,
}

async fn mysql_get_server_variables(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ServerVariablesQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<ServerVariable>>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let variables = mysql_service.get_server_variables(params.filter.as_deref()).await?;
    Ok(Json(variables))
}

async fn mysql_get_process_list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<ProcessInfo>>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let processes = mysql_service.get_process_list().await?;
    Ok(Json(processes))
}

async fn mysql_kill_process(
    State(state): State<Arc<AppState>>,
    Path(process_id): Path<u64>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    mysql_service.kill_process(process_id).await?;
    Ok(StatusCode::OK)
}

// ==================== MySQL Query Analysis handlers ====================

#[derive(Deserialize)]
struct ExplainRequest {
    connection_id: i64,
    database: String,
    query: String,
}

async fn mysql_explain_query(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExplainRequest>,
) -> Result<Json<ExplainResult>, AppError> {
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(req.connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let result = mysql_service.explain_query(&req.database, &req.query).await?;
    Ok(Json(result))
}

// ==================== MySQL Table Maintenance handlers ====================

async fn mysql_optimize_table(
    State(state): State<Arc<AppState>>,
    Path((db, table)): Path<(String, String)>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<Json<TableMaintenanceResult>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let result = mysql_service.optimize_table(&db, &table).await?;
    Ok(Json(result))
}

async fn mysql_analyze_table(
    State(state): State<Arc<AppState>>,
    Path((db, table)): Path<(String, String)>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<Json<TableMaintenanceResult>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let result = mysql_service.analyze_table(&db, &table).await?;
    Ok(Json(result))
}

async fn mysql_check_table(
    State(state): State<Arc<AppState>>,
    Path((db, table)): Path<(String, String)>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<Json<TableMaintenanceResult>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mysql_service = MysqlService::connect(&connection).await?;
    let result = mysql_service.check_table(&db, &table).await?;
    Ok(Json(result))
}

// ==================== Redis handlers ====================

#[derive(Deserialize)]
struct RedisKeysQuery {
    connection_id: Option<i64>,
    pattern: Option<String>,
    cursor: Option<u64>,
    count: Option<u64>,
}

#[derive(Deserialize)]
struct RedisKeyQuery {
    connection_id: Option<i64>,
}

#[derive(Deserialize)]
struct RedisSetKeyRequest {
    connection_id: i64,
    key: String,
    #[serde(rename = "type")]
    key_type: String,
    value: serde_json::Value,
    ttl: Option<i64>,
}

#[derive(Deserialize)]
struct RedisTtlRequest {
    connection_id: i64,
    ttl: i64,
}

async fn redis_get_info(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ConnectionIdQuery>,
    headers: HeaderMap,
) -> Result<Json<RedisServerInfo>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mut redis_service = RedisService::connect(&connection).await?;
    let info = redis_service.get_info().await?;
    Ok(Json(info))
}

async fn redis_list_keys(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RedisKeysQuery>,
    headers: HeaderMap,
) -> Result<Json<RedisKeyListResponse>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mut redis_service = RedisService::connect(&connection).await?;
    let pattern = params.pattern.as_deref().unwrap_or("*");
    let cursor = params.cursor.unwrap_or(0);
    let count = params.count.unwrap_or(100);
    let result = redis_service.list_keys(pattern, cursor, count).await?;
    Ok(Json(result))
}

async fn redis_get_key(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    Query(params): Query<RedisKeyQuery>,
    headers: HeaderMap,
) -> Result<Json<RedisKeyValue>, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mut redis_service = RedisService::connect(&connection).await?;
    let value = redis_service.get_key(&key).await?;
    Ok(Json(value))
}

async fn redis_set_key(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RedisSetKeyRequest>,
) -> Result<StatusCode, AppError> {
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(req.connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mut redis_service = RedisService::connect(&connection).await?;
    let set_req = SetKeyRequest {
        key: req.key,
        key_type: req.key_type,
        value: req.value,
        ttl: req.ttl,
    };
    redis_service.set_key(&set_req).await?;
    Ok(StatusCode::OK)
}

async fn redis_delete_key(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    Query(params): Query<RedisKeyQuery>,
    headers: HeaderMap,
) -> Result<StatusCode, AppError> {
    let connection_id = extract_connection_id(params.connection_id, &headers)?;
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mut redis_service = RedisService::connect(&connection).await?;
    redis_service.delete_key(&key).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn redis_set_ttl(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    Json(req): Json<RedisTtlRequest>,
) -> Result<StatusCode, AppError> {
    let conn_service = ConnectionService::new(state.pool.clone());
    let connection = conn_service.get_by_id(req.connection_id).await?;
    let connection = ensure_port_forward_for_http(&state, connection).await?;
    let mut redis_service = RedisService::connect(&connection).await?;
    redis_service.set_ttl(&key, req.ttl).await?;
    Ok(StatusCode::OK)
}

// ==================== History handlers ====================

#[derive(Deserialize)]
struct HistoryQuery {
    conn_type: Option<String>,
    database: Option<String>,
    status: Option<String>,
    keyword: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn get_history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HistoryQuery>,
) -> Result<Json<QueryHistoryListResponse>, AppError> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    let (history, total) = state.pool.get_query_history(
        params.conn_type.as_deref(),
        params.database.as_deref(),
        params.status.as_deref(),
        params.keyword.as_deref(),
        limit,
        offset,
    ).await?;
    Ok(Json(QueryHistoryListResponse { history, total }))
}

async fn add_history(
    State(state): State<Arc<AppState>>,
    Json(data): Json<AddQueryHistoryRequest>,
) -> Result<Json<QueryHistory>, AppError> {
    let history = state.pool.add_query_history(&data).await?;
    Ok(Json(history))
}

async fn delete_history(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    state.pool.delete_query_history(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct CleanupRequest {
    days: i64,
}

async fn cleanup_history(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CleanupRequest>,
) -> Result<Json<i64>, AppError> {
    let deleted = state.pool.cleanup_query_history(req.days).await?;
    Ok(Json(deleted))
}

// ==================== Saved query handlers ====================

#[derive(Deserialize)]
struct SavedQueryQuery {
    category: Option<String>,
}

async fn get_saved_queries(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SavedQueryQuery>,
) -> Result<Json<Vec<SavedQuery>>, AppError> {
    let queries = state.pool.get_saved_queries(params.category.as_deref()).await?;
    Ok(Json(queries))
}

async fn get_saved_query(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<SavedQuery>, AppError> {
    let query = state.pool.get_saved_query(id).await?;
    Ok(Json(query))
}

async fn create_saved_query(
    State(state): State<Arc<AppState>>,
    Json(data): Json<CreateSavedQueryRequest>,
) -> Result<Json<SavedQuery>, AppError> {
    let query = state.pool.create_saved_query(&data).await?;
    Ok(Json(query))
}

async fn update_saved_query(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(data): Json<UpdateSavedQueryRequest>,
) -> Result<Json<SavedQuery>, AppError> {
    let query = state.pool.update_saved_query(id, &data).await?;
    Ok(Json(query))
}

async fn delete_saved_query(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    state.pool.delete_saved_query(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ==================== Helper functions for K8s connection testing ====================

/// Find an available local port
fn find_available_port() -> Result<u16, AppError> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| AppError::PortForward(format!("Failed to find available port: {}", e)))?;

    let port = listener.local_addr()
        .map_err(|e| AppError::PortForward(format!("Failed to get local address: {}", e)))?
        .port();

    Ok(port)
}

/// Find a pod for the given service
async fn find_pod_for_service(
    client: &Client,
    namespace: &str,
    service_name: &str,
) -> Result<String, AppError> {
    // Get the service to find its selector
    let services: Api<k8s_openapi::api::core::v1::Service> = Api::namespaced(client.clone(), namespace);
    let service = services.get(service_name).await
        .map_err(|e| AppError::K8s(format!("Failed to get service {}: {}", service_name, e)))?;

    // Get selector labels from the service
    let selector = service.spec
        .and_then(|s| s.selector)
        .ok_or_else(|| AppError::K8s(format!("Service {} has no selector", service_name)))?;

    // Build label selector string
    let label_selector: String = selector.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",");

    // Find pods matching the selector
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let pod_list = pods.list(&kube::api::ListParams::default().labels(&label_selector))
        .await
        .map_err(|e| AppError::K8s(format!("Failed to list pods: {}", e)))?;

    // Return the first running pod
    for pod in pod_list.items {
        if let Some(status) = &pod.status {
            if let Some(phase) = &status.phase {
                if phase == "Running" {
                    if let Some(name) = pod.metadata.name {
                        return Ok(name);
                    }
                }
            }
        }
    }

    Err(AppError::K8s(format!("No running pods found for service {}", service_name)))
}

/// Test connection through the forwarded port
async fn test_connection_through_port(
    data: &TestK8sConnectionRequest,
    local_port: u16,
) -> Result<TestConnectionResult, AppError> {
    match data.conn_type.as_str() {
        "mysql" => test_mysql_connection(data, local_port).await,
        "redis" => test_redis_connection(data, local_port).await,
        _ => Ok(TestConnectionResult::failure(format!(
            "Unsupported connection type: {}",
            data.conn_type
        ))),
    }
}

/// Test MySQL connection through forwarded port
async fn test_mysql_connection(
    data: &TestK8sConnectionRequest,
    local_port: u16,
) -> Result<TestConnectionResult, AppError> {
    use sqlx::mysql::MySqlPoolOptions;
    use urlencoding::encode;

    let password = data.password.as_deref().unwrap_or("");
    let username = data.username.as_deref().unwrap_or("root");
    let database = data.database_name.as_deref().unwrap_or("");

    // URL-encode username and password to handle special characters like / @ :
    let encoded_username = encode(username);
    let encoded_password = encode(password);

    let url = if database.is_empty() {
        format!(
            "mysql://{}:{}@127.0.0.1:{}",
            encoded_username, encoded_password, local_port
        )
    } else {
        format!(
            "mysql://{}:{}@127.0.0.1:{}/{}",
            encoded_username, encoded_password, local_port, database
        )
    };

    let result = MySqlPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&url)
        .await;

    match result {
        Ok(pool) => {
            let version: Result<(String,), _> =
                sqlx::query_as("SELECT VERSION()").fetch_one(&pool).await;

            match version {
                Ok((ver,)) => Ok(TestConnectionResult::success(format!(
                    "Connected to MySQL {} via K8s port forward",
                    ver
                ))),
                Err(e) => Ok(TestConnectionResult::failure(e.to_string())),
            }
        }
        Err(e) => Ok(TestConnectionResult::failure(e.to_string())),
    }
}

/// Test Redis connection through forwarded port
async fn test_redis_connection(
    data: &TestK8sConnectionRequest,
    local_port: u16,
) -> Result<TestConnectionResult, AppError> {
    use urlencoding::encode;

    let password = data.password.as_deref();

    // URL-encode password to handle special characters like / @ :
    let url = if let Some(pwd) = password {
        if pwd.is_empty() {
            format!("redis://127.0.0.1:{}", local_port)
        } else {
            let encoded_pwd = encode(pwd);
            format!("redis://:{}@127.0.0.1:{}", encoded_pwd, local_port)
        }
    } else {
        format!("redis://127.0.0.1:{}", local_port)
    };

    let result = redis::Client::open(url.as_str());

    match result {
        Ok(client) => {
            let con_result = client.get_multiplexed_tokio_connection().await;

            match con_result {
                Ok(mut con) => {
                    let pong: Result<String, _> = redis::cmd("PING")
                        .query_async(&mut con)
                        .await;

                    match pong {
                        Ok(_) => Ok(TestConnectionResult::success("Connected to Redis via K8s port forward")),
                        Err(e) => Ok(TestConnectionResult::failure(e.to_string())),
                    }
                }
                Err(e) => Ok(TestConnectionResult::failure(e.to_string())),
            }
        }
        Err(e) => Ok(TestConnectionResult::failure(e.to_string())),
    }
}

// ==================== Port forward handlers ====================

async fn list_port_forwards(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PortForward>>, AppError> {
    let service = state.port_forward_service.read().await;
    let forwards = service.list().await?;
    Ok(Json(forwards))
}

#[derive(Deserialize)]
struct StartPortForwardRequestBody {
    connection_id: i64,
    #[serde(default)]
    local_port: Option<u16>,
}

async fn start_port_forward(
    State(state): State<Arc<AppState>>,
    Json(req): Json<StartPortForwardRequestBody>,
) -> Result<Json<PortForward>, AppError> {
    let service = state.port_forward_service.read().await;
    let forward = service.start(req.connection_id, req.local_port).await?;
    Ok(Json(forward))
}

async fn get_port_forward(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<PortForward>, AppError> {
    let service = state.port_forward_service.read().await;
    let forward = service.get(&id).await?;
    Ok(Json(forward))
}

async fn stop_port_forward(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let service = state.port_forward_service.read().await;
    service.stop(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn reconnect_port_forward(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<PortForward>, AppError> {
    let service = state.port_forward_service.read().await;
    let forward = service.reconnect(&id, None).await?;
    Ok(Json(forward))
}

async fn touch_port_forward(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let service = state.port_forward_service.read().await;
    service.touch(&id).await?;
    Ok(StatusCode::OK)
}

async fn get_port_forward_by_connection(
    State(state): State<Arc<AppState>>,
    Path(connection_id): Path<i64>,
) -> Result<Json<PortForward>, AppError> {
    let service = state.port_forward_service.read().await;
    let forward = service.get_by_connection(connection_id).await?;
    Ok(Json(forward))
}

// ==================== Settings handlers ====================

use crate::db::models::{UserSetting, UpsertSettingRequest, BatchGetSettingsRequest, BatchSettingsResponse};
use crate::db::models::{CreateLLMConfigRequest, UpdateLLMConfigRequest, LLMConfigResponse};
use crate::services::{SettingsService, LLMConfigService};

async fn get_all_settings_http(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserSetting>>, AppError> {
    let service = SettingsService::new(state.pool.clone());
    let settings = service.get_all().await?;
    Ok(Json(settings))
}

async fn get_setting_http(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> Result<Json<Option<serde_json::Value>>, AppError> {
    let service = SettingsService::new(state.pool.clone());
    let value = service.get(&key).await?;
    Ok(Json(value))
}

async fn get_settings_batch_http(
    State(state): State<Arc<AppState>>,
    Json(request): Json<BatchGetSettingsRequest>,
) -> Result<Json<BatchSettingsResponse>, AppError> {
    let service = SettingsService::new(state.pool.clone());
    let settings = service.get_batch(&request.keys).await?;
    Ok(Json(settings))
}

#[derive(Deserialize)]
struct SetSettingBody {
    value: serde_json::Value,
}

async fn set_setting_http(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    Json(body): Json<SetSettingBody>,
) -> Result<Json<UserSetting>, AppError> {
    let service = SettingsService::new(state.pool.clone());
    let request = UpsertSettingRequest {
        key,
        value: body.value,
    };
    let setting = service.set(&request).await?;
    Ok(Json(setting))
}

async fn delete_setting_http(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> Result<StatusCode, AppError> {
    let service = SettingsService::new(state.pool.clone());
    service.delete(&key).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ==================== LLM Config handlers ====================

async fn get_all_llm_configs_http(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<LLMConfigResponse>>, AppError> {
    let service = LLMConfigService::new(state.pool.clone());
    let configs = service.get_all().await?;
    Ok(Json(configs))
}

async fn get_llm_config_http(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<LLMConfigResponse>, AppError> {
    let service = LLMConfigService::new(state.pool.clone());
    let config = service.get(id).await?;
    Ok(Json(config))
}

async fn get_default_llm_config_http(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Option<LLMConfigResponse>>, AppError> {
    let service = LLMConfigService::new(state.pool.clone());
    let config = service.get_default().await?;
    Ok(Json(config))
}

async fn create_llm_config_http(
    State(state): State<Arc<AppState>>,
    Json(data): Json<CreateLLMConfigRequest>,
) -> Result<Json<LLMConfigResponse>, AppError> {
    let service = LLMConfigService::new(state.pool.clone());
    let config = service.create(data).await?;
    Ok(Json(config))
}

async fn update_llm_config_http(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(data): Json<UpdateLLMConfigRequest>,
) -> Result<Json<LLMConfigResponse>, AppError> {
    let service = LLMConfigService::new(state.pool.clone());
    let config = service.update(id, data).await?;
    Ok(Json(config))
}

async fn delete_llm_config_http(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let service = LLMConfigService::new(state.pool.clone());
    service.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn set_default_llm_config_http(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<LLMConfigResponse>, AppError> {
    let service = LLMConfigService::new(state.pool.clone());
    let config = service.set_default(id).await?;
    Ok(Json(config))
}

async fn get_llm_api_key_http(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<Option<String>>, AppError> {
    let service = LLMConfigService::new(state.pool.clone());
    let api_key = service.get_api_key(id).await?;
    Ok(Json(api_key))
}

// ==================== Extended K8s Workload handlers ====================

async fn k8s_list_jobs_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace)): Path<(i64, String)>,
) -> Result<Json<Vec<K8sJob>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let jobs = k8s.list_jobs(&namespace).await?;
    Ok(Json(jobs))
}

async fn k8s_list_cronjobs_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace)): Path<(i64, String)>,
) -> Result<Json<Vec<K8sCronJob>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let cronjobs = k8s.list_cronjobs(&namespace).await?;
    Ok(Json(cronjobs))
}

async fn k8s_list_statefulsets_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace)): Path<(i64, String)>,
) -> Result<Json<Vec<K8sStatefulSet>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let statefulsets = k8s.list_statefulsets(&namespace).await?;
    Ok(Json(statefulsets))
}

async fn k8s_list_daemonsets_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace)): Path<(i64, String)>,
) -> Result<Json<Vec<K8sDaemonSet>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let daemonsets = k8s.list_daemonsets(&namespace).await?;
    Ok(Json(daemonsets))
}

async fn k8s_list_replicasets_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace)): Path<(i64, String)>,
) -> Result<Json<Vec<K8sReplicaSet>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let replicasets = k8s.list_replicasets(&namespace).await?;
    Ok(Json(replicasets))
}

// ==================== Deployment Operations handlers ====================

async fn k8s_get_deployment_yaml_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace, name)): Path<(i64, String, String)>,
) -> Result<String, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let yaml = k8s.get_deployment_yaml(&namespace, &name).await?;
    Ok(yaml)
}

#[derive(Deserialize)]
struct UpdateDeploymentYamlRequest {
    yaml: String,
}

async fn k8s_update_deployment_yaml_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace, name)): Path<(i64, String, String)>,
    Json(req): Json<UpdateDeploymentYamlRequest>,
) -> Result<StatusCode, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    k8s.update_deployment_yaml(&namespace, &name, &req.yaml).await?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct ScaleDeploymentRequest {
    replicas: i32,
}

async fn k8s_scale_deployment_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace, name)): Path<(i64, String, String)>,
    Json(req): Json<ScaleDeploymentRequest>,
) -> Result<StatusCode, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    k8s.scale_deployment(&namespace, &name, req.replicas).await?;
    Ok(StatusCode::OK)
}

async fn k8s_restart_deployment_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace, name)): Path<(i64, String, String)>,
) -> Result<StatusCode, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    k8s.restart_deployment(&namespace, &name).await?;
    Ok(StatusCode::OK)
}

// ==================== Proxy Pod Operations ====================

async fn k8s_list_proxies_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace)): Path<(i64, String)>,
) -> Result<Json<Vec<crate::db::models::ProxyPodInfo>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let proxies = k8s.list_proxy_pods(&namespace).await?;
    Ok(Json(proxies))
}

async fn k8s_list_all_proxies_http(
    State(state): State<Arc<AppState>>,
    Path(cluster_id): Path<i64>,
) -> Result<Json<Vec<crate::db::models::ProxyPodInfo>>, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    let proxies = k8s.list_all_proxy_pods().await?;
    Ok(Json(proxies))
}

#[derive(Debug, Deserialize)]
struct CreateProxyRequest {
    proxy_name: String,
    target_host: String,
    target_port: u16,
    target_type: String,
    /// Optional custom image for the proxy container (defaults to "alpine/socat")
    image: Option<String>,
}

async fn k8s_create_proxy_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace)): Path<(i64, String)>,
    Json(req): Json<CreateProxyRequest>,
) -> Result<StatusCode, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    k8s.create_tcp_proxy(
        &namespace,
        &req.proxy_name,
        &req.target_host,
        req.target_port,
        &req.target_type,
        req.image.as_deref(),
    )
    .await?;
    Ok(StatusCode::CREATED)
}

async fn k8s_delete_proxy_http(
    State(state): State<Arc<AppState>>,
    Path((cluster_id, namespace, name)): Path<(i64, String, String)>,
) -> Result<StatusCode, AppError> {
    let k8s = get_k8s_service_from_cluster(&state.pool, cluster_id).await?;
    k8s.delete_tcp_proxy(&namespace, &name).await?;
    Ok(StatusCode::OK)
}
