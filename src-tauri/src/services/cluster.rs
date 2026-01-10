//! Cluster management service
//!
//! This service handles Kubernetes cluster configuration management.

use crate::db::models::{Cluster, Connection};
use crate::db::SqlitePool;
use crate::error::{AppError, AppResult};

/// Service for managing Kubernetes cluster configurations
pub struct ClusterService {
    pool: SqlitePool,
}

impl ClusterService {
    /// Create a new ClusterService instance
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get all clusters
    pub async fn get_all(&self) -> AppResult<Vec<Cluster>> {
        let mut clusters = self.pool.get_all_clusters().await?;
        // Clear kubeconfig for security (don't expose in responses)
        for cluster in &mut clusters {
            cluster.kubeconfig = None;
        }
        Ok(clusters)
    }

    /// Get cluster by ID
    pub async fn get_by_id(&self, id: i64) -> AppResult<Cluster> {
        let mut cluster = self.pool.get_cluster(id).await?;
        // Clear kubeconfig for security
        cluster.kubeconfig = None;
        Ok(cluster)
    }

    /// Get cluster by name
    pub async fn get_by_name(&self, name: &str) -> AppResult<Cluster> {
        let mut cluster = self.pool.get_cluster_by_name(name).await?;
        // Clear kubeconfig for security
        cluster.kubeconfig = None;
        Ok(cluster)
    }

    /// Create a new cluster
    pub async fn create(&self, cluster: &Cluster) -> AppResult<Cluster> {
        // Check if cluster name already exists
        if self.pool.get_cluster_by_name(&cluster.name).await.is_ok() {
            return Err(AppError::Validation(format!(
                "Cluster with name '{}' already exists",
                cluster.name
            )));
        }

        let mut created = self.pool.create_cluster(cluster).await?;
        // Clear kubeconfig for security
        created.kubeconfig = None;
        Ok(created)
    }

    /// Update an existing cluster
    pub async fn update(&self, id: i64, cluster: &Cluster) -> AppResult<Cluster> {
        // Verify cluster exists
        self.pool.get_cluster(id).await?;

        // Check if new name conflicts with another cluster
        if let Ok(existing) = self.pool.get_cluster_by_name(&cluster.name).await {
            if existing.id != Some(id) {
                return Err(AppError::Validation(format!(
                    "Cluster with name '{}' already exists",
                    cluster.name
                )));
            }
        }

        let mut updated = self.pool.update_cluster(id, cluster).await?;
        // Clear kubeconfig for security
        updated.kubeconfig = None;
        Ok(updated)
    }

    /// Delete a cluster
    pub async fn delete(&self, id: i64) -> AppResult<()> {
        // Check if there are associated connections
        let connections = self.pool.get_connections_by_cluster(id).await?;
        if !connections.is_empty() {
            return Err(AppError::Validation(format!(
                "Cannot delete cluster: {} connections are associated with it",
                connections.len()
            )));
        }

        self.pool.delete_cluster(id).await
    }

    /// Get all connections associated with a cluster
    pub async fn get_connections(&self, cluster_id: i64) -> AppResult<Vec<Connection>> {
        // Verify cluster exists
        self.pool.get_cluster(cluster_id).await?;

        let mut connections = self.pool.get_connections_by_cluster(cluster_id).await?;
        // Clear passwords for security
        for conn in &mut connections {
            conn.password = None;
        }
        Ok(connections)
    }

    /// Get cluster with kubeconfig (for internal use only, not exposed via IPC)
    pub(crate) async fn get_with_kubeconfig(&self, id: i64) -> AppResult<Cluster> {
        self.pool.get_cluster(id).await
    }
}
