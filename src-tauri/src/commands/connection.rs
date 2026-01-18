//! Tauri commands for connection management
//!
//! These commands are exposed to the frontend via IPC.

use std::net::TcpListener;
use std::time::Duration;
use tauri::State;

use kube::{Api, Client, Config};
use kube::config::{KubeConfigOptions, Kubeconfig};
use k8s_openapi::api::core::v1::Pod;

use crate::db::models::{Connection, TestConnectionResult, TestK8sConnectionRequest};
use crate::db::SqlitePool;
use crate::error::AppError;
use crate::services::ConnectionService;

/// Get all connections
#[tauri::command]
pub async fn get_all_connections(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<Connection>, AppError> {
    let service = ConnectionService::new(pool.inner().clone());
    service.get_all().await
}

/// Get a single connection by ID
#[tauri::command]
pub async fn get_connection(pool: State<'_, SqlitePool>, id: i64) -> Result<Connection, AppError> {
    let service = ConnectionService::new(pool.inner().clone());
    service.get_by_id(id).await
}

/// Get connections by type
#[tauri::command]
pub async fn get_connections_by_type(
    pool: State<'_, SqlitePool>,
    conn_type: String,
) -> Result<Vec<Connection>, AppError> {
    let service = ConnectionService::new(pool.inner().clone());
    service.get_by_type(&conn_type).await
}

/// Create a new connection
#[tauri::command]
pub async fn create_connection(
    pool: State<'_, SqlitePool>,
    data: Connection,
) -> Result<Connection, AppError> {
    let service = ConnectionService::new(pool.inner().clone());
    service.create(data).await
}

/// Update an existing connection
#[tauri::command]
pub async fn update_connection(
    pool: State<'_, SqlitePool>,
    id: i64,
    data: Connection,
) -> Result<Connection, AppError> {
    let service = ConnectionService::new(pool.inner().clone());
    service.update(id, data).await
}

/// Delete a connection
#[tauri::command]
pub async fn delete_connection(pool: State<'_, SqlitePool>, id: i64) -> Result<(), AppError> {
    let service = ConnectionService::new(pool.inner().clone());
    service.delete(id).await
}

/// Test a connection without saving
#[tauri::command]
pub async fn test_connection(
    pool: State<'_, SqlitePool>,
    data: Connection,
) -> Result<TestConnectionResult, AppError> {
    let service = ConnectionService::new(pool.inner().clone());
    service.test(&data).await
}

/// Test a K8s connection by creating a temporary port forward
#[tauri::command]
pub async fn test_k8s_connection(
    pool: State<'_, SqlitePool>,
    data: TestK8sConnectionRequest,
) -> Result<TestConnectionResult, AppError> {
    // 30-second overall timeout for the entire test
    log::info!("[test-k8s] Starting connection test for {} service {}/{}",
        data.conn_type, data.k8s_namespace, data.k8s_service_name);

    match tokio::time::timeout(
        Duration::from_secs(30),
        test_k8s_connection_inner(pool.inner().clone(), &data)
    ).await {
        Ok(result) => {
            log::info!("[test-k8s] Connection test completed");
            result
        },
        Err(_) => {
            log::error!("[test-k8s] Connection test timed out after 30 seconds");
            Ok(TestConnectionResult::failure(
                "Connection test timed out after 30 seconds. Please check: 1) K8s cluster connectivity, 2) Service exists in namespace, 3) Pod is running"
            ))
        }
    }
}

async fn test_k8s_connection_inner(
    pool: SqlitePool,
    data: &TestK8sConnectionRequest,
) -> Result<TestConnectionResult, AppError> {
    // Step 1: Get kubeconfig
    log::info!("[test-k8s] Step 1: Getting kubeconfig...");
    let (kubeconfig_content, context) = if let Some(kc) = &data.kubeconfig {
        (kc.clone(), data.context.clone())
    } else if let Some(cluster_id) = data.cluster_id {
        log::info!("[test-k8s] Looking up kubeconfig for cluster ID: {}", cluster_id);
        let cluster = pool.get_cluster(cluster_id).await?;
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
    log::info!("[test-k8s] Step 1: Kubeconfig obtained successfully");

    // Step 2: Create K8s client from kubeconfig
    log::info!("[test-k8s] Step 2: Creating K8s client...");
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
    log::info!("[test-k8s] Step 2: K8s client created successfully");

    // Step 3: Find a pod for the service
    log::info!("[test-k8s] Step 3: Finding pod for service {}/{}...",
        data.k8s_namespace, data.k8s_service_name);
    let pod_name = find_pod_for_service(&client, &data.k8s_namespace, &data.k8s_service_name).await?;
    log::info!("[test-k8s] Step 3: Found pod: {}", pod_name);

    // Step 4: Find an available local port
    let local_port = find_available_port()?;
    log::info!("[test-k8s] Step 4: Allocated local port: {}", local_port);

    // Step 5: Create port forward
    log::info!("[test-k8s] Step 5: Creating port forward localhost:{} -> {}:{}",
        local_port, pod_name, data.k8s_service_port);
    let pods: Api<Pod> = Api::namespaced(client.clone(), &data.k8s_namespace);
    let mut pf = pods.portforward(&pod_name, &[data.k8s_service_port as u16])
        .await
        .map_err(|e| AppError::K8s(format!("Failed to create port forward: {}", e)))?;

    // Get the port stream
    let upstream = pf.take_stream(data.k8s_service_port as u16)
        .ok_or_else(|| AppError::K8s("Failed to get port stream".to_string()))?;
    log::info!("[test-k8s] Step 5: Port forward established");

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

    // Step 6: Test the connection through the forwarded port
    log::info!("[test-k8s] Step 6: Testing {} connection on local port {}...",
        data.conn_type, local_port);
    let result = test_connection_through_port(data, local_port).await;
    log::info!("[test-k8s] Step 6: Connection test result: {:?}",
        result.as_ref().map(|r| r.success));

    // Stop the port forward
    let _ = stop_tx.send(());
    let _ = pf_handle.await;

    result
}

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

    log::info!("[test-k8s] Redis: Connecting to {}", url.replace(password.unwrap_or(""), "***"));

    let result = redis::Client::open(url.as_str());

    match result {
        Ok(client) => {
            // Add 10-second timeout for Redis connection
            let con_result = tokio::time::timeout(
                Duration::from_secs(10),
                client.get_multiplexed_tokio_connection()
            ).await;

            match con_result {
                Ok(Ok(mut con)) => {
                    // Add 5-second timeout for PING command
                    let pong: Result<Result<String, _>, _> = tokio::time::timeout(
                        Duration::from_secs(5),
                        redis::cmd("PING").query_async(&mut con)
                    ).await;

                    match pong {
                        Ok(Ok(_)) => {
                            log::info!("[test-k8s] Redis: PING successful");
                            Ok(TestConnectionResult::success("Connected to Redis via K8s port forward"))
                        },
                        Ok(Err(e)) => {
                            log::error!("[test-k8s] Redis: PING failed: {}", e);
                            Ok(TestConnectionResult::failure(e.to_string()))
                        },
                        Err(_) => {
                            log::error!("[test-k8s] Redis: PING timed out");
                            Ok(TestConnectionResult::failure("Redis PING command timed out after 5 seconds"))
                        }
                    }
                }
                Ok(Err(e)) => {
                    log::error!("[test-k8s] Redis: Connection failed: {}", e);
                    Ok(TestConnectionResult::failure(e.to_string()))
                },
                Err(_) => {
                    log::error!("[test-k8s] Redis: Connection timed out");
                    Ok(TestConnectionResult::failure("Redis connection timed out after 10 seconds"))
                }
            }
        }
        Err(e) => {
            log::error!("[test-k8s] Redis: Failed to create client: {}", e);
            Ok(TestConnectionResult::failure(e.to_string()))
        }
    }
}
