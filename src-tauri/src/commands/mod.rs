//! Tauri IPC commands
//!
//! This module exports all Tauri commands for frontend communication.

pub mod cluster;
pub mod connection;
pub mod history;
pub mod k8s;
pub mod k8s_favorite;
pub mod llm_config;
pub mod mysql;
pub mod port_forward;
pub mod redis;
pub mod saved_query;
pub mod settings;

pub use cluster::*;
pub use connection::*;
pub use history::*;
pub use k8s::*;
pub use k8s_favorite::*;
pub use llm_config::*;
pub use mysql::*;
pub use port_forward::*;
pub use redis::*;
pub use saved_query::*;
pub use settings::*;
