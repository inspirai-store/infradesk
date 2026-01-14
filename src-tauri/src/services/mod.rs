//! Business logic services
//!
//! This module contains service layer implementations for:
//! - Connection management
//! - Cluster management
//! - Crypto (password encryption)
//! - MySQL operations
//! - Redis operations
//! - Kubernetes operations
//! - Port forwarding
//! - User settings
//! - LLM configuration
//! - Log aggregation (for web debug mode)

pub mod cluster;
pub mod connection;
pub mod crypto;
pub mod k8s;
pub mod llm_config;
pub mod log_service;
pub mod mysql;
pub mod port_forward;
pub mod redis;
pub mod settings;

pub use cluster::ClusterService;
pub use connection::ConnectionService;
pub use crypto::CryptoService;
pub use k8s::K8sService;
pub use llm_config::LLMConfigService;
pub use log_service::{AddLogRequest, LogEntry, LogLevel, LogService, LogSource};
pub use mysql::MysqlService;
pub use port_forward::PortForwardService;
pub use redis::RedisService;
pub use settings::SettingsService;
