//! Tauri IPC commands
//!
//! This module exports all Tauri commands for frontend communication.

pub mod cluster;
pub mod connection;
pub mod k8s;
pub mod mysql;
pub mod redis;

pub use cluster::*;
pub use connection::*;
pub use k8s::*;
pub use mysql::*;
pub use redis::*;
