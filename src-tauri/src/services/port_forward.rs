//! Port Forwarding Service
//!
//! This service manages Kubernetes port forwarding for database connections.
//! It uses the kube crate's portforward functionality to create local TCP tunnels
//! to K8s services without requiring kubectl to be installed.

use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::RwLock;

use kube::{Api, Client, Config};
use kube::config::{KubeConfigOptions, Kubeconfig};
use k8s_openapi::api::core::v1::Pod;

use crate::db::models::{Connection, PortForward};
use crate::db::SqlitePool;
use crate::error::{AppError, AppResult};

/// Active port forward connection info
#[derive(Debug)]
struct ActiveForward {
    /// Forward ID
    id: String,
    /// Local port
    local_port: u16,
    /// Shutdown signal sender
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
}

/// Port forwarding service
pub struct PortForwardService {
    pool: SqlitePool,
    /// Map of forward ID to active forward
    active_forwards: Arc<RwLock<HashMap<String, ActiveForward>>>,
}

impl PortForwardService {
    /// Create a new port forward service
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            active_forwards: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Find an available local port
    fn find_available_port() -> AppResult<u16> {
        // Bind to port 0 to let the OS assign an available port
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| AppError::PortForward(format!("Failed to find available port: {}", e)))?;

        let port = listener.local_addr()
            .map_err(|e| AppError::PortForward(format!("Failed to get local address: {}", e)))?
            .port();

        Ok(port)
    }

    /// Check if a specific port is available
    fn is_port_available(port: u16) -> bool {
        TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
    }

    /// Get or validate a local port
    /// If preferred_port is Some and > 0, validate it's available
    /// Otherwise, find a random available port
    fn get_local_port(preferred_port: Option<u16>) -> AppResult<u16> {
        match preferred_port {
            Some(port) if port > 0 => {
                // Validate the preferred port is available
                if Self::is_port_available(port) {
                    Ok(port)
                } else {
                    Err(AppError::PortForward(format!(
                        "Port {} is already in use. Please choose a different port or leave empty for auto-assignment.",
                        port
                    )))
                }
            }
            _ => {
                // Find an available port automatically
                Self::find_available_port()
            }
        }
    }

    /// Get K8s client from connection's cluster kubeconfig
    async fn get_k8s_client(&self, connection: &Connection) -> AppResult<Client> {
        // Get the cluster associated with this connection
        if let Some(cluster_id) = connection.cluster_id {
            let cluster = self.pool.get_cluster(cluster_id).await?;

            if let Some(kubeconfig_content) = &cluster.kubeconfig {
                let kubeconfig = Kubeconfig::from_yaml(kubeconfig_content)
                    .map_err(|e| AppError::K8s(format!("Failed to parse kubeconfig: {}", e)))?;

                let options = KubeConfigOptions {
                    context: cluster.context.clone(),
                    ..Default::default()
                };

                let config = Config::from_custom_kubeconfig(kubeconfig, &options)
                    .await
                    .map_err(|e| AppError::K8s(format!("Failed to create K8s config: {}", e)))?;

                let client = Client::try_from(config)
                    .map_err(|e| AppError::K8s(format!("Failed to create K8s client: {}", e)))?;

                return Ok(client);
            }
        }

        // Fallback to in-cluster config
        Client::try_default()
            .await
            .map_err(|e| AppError::K8s(format!("Failed to create default K8s client: {}", e)))
    }

    /// Find a pod for the given service
    async fn find_pod_for_service(
        &self,
        client: &Client,
        namespace: &str,
        service_name: &str,
    ) -> AppResult<String> {
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

    /// Start a port forward for a connection
    ///
    /// # Arguments
    /// * `connection_id` - The connection ID to forward
    /// * `preferred_local_port` - Optional preferred local port. If None or 0, auto-assign.
    pub async fn start(&self, connection_id: i64, preferred_local_port: Option<u16>) -> AppResult<PortForward> {
        // Get connection details
        let connection = self.pool.get_connection(connection_id).await?;

        // Validate K8s connection
        let namespace = connection.k8s_namespace.as_ref()
            .ok_or_else(|| AppError::PortForward("Connection has no K8s namespace".to_string()))?;
        let service_name = connection.k8s_service_name.as_ref()
            .ok_or_else(|| AppError::PortForward("Connection has no K8s service name".to_string()))?;
        let remote_port = connection.k8s_service_port
            .ok_or_else(|| AppError::PortForward("Connection has no K8s service port".to_string()))?;

        // Check if forward already exists
        if let Ok(existing) = self.pool.get_port_forward_by_connection(connection_id).await {
            // Check if it's active
            let forwards = self.active_forwards.read().await;
            if forwards.contains_key(&existing.id.clone().unwrap_or_default()) {
                return Ok(existing);
            }
            // If not active, delete the old record
            if let Some(id) = &existing.id {
                let _ = self.pool.delete_port_forward(id).await;
            }
        }

        // Get local port (use preferred if specified and available, otherwise auto-assign)
        let local_port = Self::get_local_port(preferred_local_port)?;

        // Create forward record
        let forward_id = uuid::Uuid::new_v4().to_string();
        let forward = PortForward {
            id: Some(forward_id.clone()),
            connection_id,
            namespace: namespace.clone(),
            service_name: service_name.clone(),
            remote_port,
            local_port: local_port as i32,
            status: "starting".to_string(),
            error: None,
            last_used: None,
            created_at: None,
        };

        let created = self.pool.create_port_forward(&forward).await?;

        // Start the actual port forward in background
        let pool = self.pool.clone();
        let active_forwards = self.active_forwards.clone();
        let connection = connection.clone();
        let fwd_id = forward_id.clone();
        let ns = namespace.clone();
        let svc = service_name.clone();

        tokio::spawn(async move {
            match Self::run_port_forward(
                pool.clone(),
                active_forwards,
                connection,
                fwd_id.clone(),
                ns,
                svc,
                remote_port as u16,
                local_port,
            ).await {
                Ok(_) => {
                    log::info!("Port forward {} stopped gracefully", fwd_id);
                }
                Err(e) => {
                    log::error!("Port forward {} failed: {}", fwd_id, e);
                    let _ = pool.update_port_forward_status(&fwd_id, "error", Some(&e.to_string())).await;
                }
            }
        });

        Ok(created)
    }

    /// Run the actual port forward
    async fn run_port_forward(
        pool: SqlitePool,
        active_forwards: Arc<RwLock<HashMap<String, ActiveForward>>>,
        connection: Connection,
        forward_id: String,
        namespace: String,
        service_name: String,
        remote_port: u16,
        local_port: u16,
    ) -> AppResult<()> {
        // Create shutdown channel
        let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);

        // Register active forward
        {
            let mut forwards = active_forwards.write().await;
            forwards.insert(forward_id.clone(), ActiveForward {
                id: forward_id.clone(),
                local_port,
                shutdown_tx: shutdown_tx.clone(),
            });
        }

        // Create K8s client
        let client = {
            let service = PortForwardService::new(pool.clone());
            service.get_k8s_client(&connection).await?
        };

        // Find pod for service
        let pod_name = {
            let service = PortForwardService::new(pool.clone());
            service.find_pod_for_service(&client, &namespace, &service_name).await?
        };

        log::info!("Starting port forward {} -> {}:{} via pod {}",
            local_port, service_name, remote_port, pod_name);

        // Update status to active
        pool.update_port_forward_status(&forward_id, "active", None).await?;

        // Bind local listener
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", local_port))
            .await
            .map_err(|e| AppError::PortForward(format!("Failed to bind to port {}: {}", local_port, e)))?;

        let pods: Api<Pod> = Api::namespaced(client.clone(), &namespace);
        let mut shutdown_rx = shutdown_tx.subscribe();

        loop {
            tokio::select! {
                // Handle shutdown signal
                _ = shutdown_rx.recv() => {
                    log::info!("Port forward {} received shutdown signal", forward_id);
                    break;
                }
                // Accept new connections
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            log::debug!("Accepted connection from {} for forward {}", addr, forward_id);

                            // Update last used time
                            let _ = pool.touch_port_forward(&forward_id).await;

                            // Handle the connection in a separate task
                            let pods = pods.clone();
                            let pod_name = pod_name.clone();
                            let forward_id = forward_id.clone();

                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_connection(
                                    stream,
                                    pods,
                                    &pod_name,
                                    remote_port,
                                ).await {
                                    log::error!("Forward {} connection error: {}", forward_id, e);
                                }
                            });
                        }
                        Err(e) => {
                            log::error!("Failed to accept connection for forward {}: {}", forward_id, e);
                        }
                    }
                }
            }
        }

        // Cleanup
        {
            let mut forwards = active_forwards.write().await;
            forwards.remove(&forward_id);
        }
        pool.update_port_forward_status(&forward_id, "stopped", None).await?;

        Ok(())
    }

    /// Handle a single forwarded connection
    async fn handle_connection(
        mut local_stream: tokio::net::TcpStream,
        pods: Api<Pod>,
        pod_name: &str,
        remote_port: u16,
    ) -> AppResult<()> {
        // Create port forward to pod
        let mut pf = pods.portforward(pod_name, &[remote_port])
            .await
            .map_err(|e| AppError::PortForward(format!("Failed to create port forward: {}", e)))?;

        // Get the port stream
        let upstream = pf.take_stream(remote_port)
            .ok_or_else(|| AppError::PortForward("Failed to get port stream".to_string()))?;

        // Bidirectional copy
        let (mut local_read, mut local_write) = local_stream.split();
        let (mut upstream_read, mut upstream_write) = tokio::io::split(upstream);

        let client_to_server = tokio::io::copy(&mut local_read, &mut upstream_write);
        let server_to_client = tokio::io::copy(&mut upstream_read, &mut local_write);

        tokio::select! {
            result = client_to_server => {
                if let Err(e) = result {
                    log::debug!("Client to server copy ended: {}", e);
                }
            }
            result = server_to_client => {
                if let Err(e) = result {
                    log::debug!("Server to client copy ended: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Stop a port forward
    pub async fn stop(&self, id: &str) -> AppResult<()> {
        // Send shutdown signal
        {
            let forwards = self.active_forwards.read().await;
            if let Some(forward) = forwards.get(id) {
                let _ = forward.shutdown_tx.send(());
            }
        }

        // Update status
        self.pool.update_port_forward_status(id, "stopped", None).await?;

        // Remove from active forwards
        {
            let mut forwards = self.active_forwards.write().await;
            forwards.remove(id);
        }

        Ok(())
    }

    /// Reconnect a port forward
    ///
    /// # Arguments
    /// * `id` - The port forward ID to reconnect
    /// * `preferred_local_port` - Optional preferred local port. If None, reuse the old port.
    pub async fn reconnect(&self, id: &str, preferred_local_port: Option<u16>) -> AppResult<PortForward> {
        // Get existing forward
        let forward = self.pool.get_port_forward(id).await?;

        // Use the old port if no preferred port is specified
        let port_to_use = preferred_local_port.or(Some(forward.local_port as u16));

        // Stop if active
        let _ = self.stop(id).await;

        // Start new forward with preferred port
        self.start(forward.connection_id, port_to_use).await
    }

    /// Get all port forwards
    pub async fn list(&self) -> AppResult<Vec<PortForward>> {
        let mut forwards = self.pool.get_all_port_forwards().await?;

        // Update status based on active forwards
        let active = self.active_forwards.read().await;
        for forward in &mut forwards {
            if let Some(id) = &forward.id {
                if active.contains_key(id) {
                    forward.status = "active".to_string();
                } else if forward.status == "active" || forward.status == "starting" {
                    forward.status = "stopped".to_string();
                }
            }
        }

        Ok(forwards)
    }

    /// Get a port forward by ID
    pub async fn get(&self, id: &str) -> AppResult<PortForward> {
        let mut forward = self.pool.get_port_forward(id).await?;

        // Update status based on active forwards
        let active = self.active_forwards.read().await;
        if active.contains_key(id) {
            forward.status = "active".to_string();
        } else if forward.status == "active" || forward.status == "starting" {
            forward.status = "stopped".to_string();
        }

        Ok(forward)
    }

    /// Get port forward by connection ID
    pub async fn get_by_connection(&self, connection_id: i64) -> AppResult<PortForward> {
        let mut forward = self.pool.get_port_forward_by_connection(connection_id).await?;

        // Update status based on active forwards
        if let Some(id) = &forward.id {
            let active = self.active_forwards.read().await;
            if active.contains_key(id) {
                forward.status = "active".to_string();
            } else if forward.status == "active" || forward.status == "starting" {
                forward.status = "stopped".to_string();
            }
        }

        Ok(forward)
    }

    /// Touch a port forward (update last used time)
    pub async fn touch(&self, id: &str) -> AppResult<()> {
        self.pool.touch_port_forward(id).await
    }
}
