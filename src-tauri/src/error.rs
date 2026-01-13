//! Unified error handling for the Tauri backend
//!
//! This module defines a common error type that can be used across all modules
//! and properly serialized for IPC communication.

use serde::Serialize;
use thiserror::Error;

/// Application-wide error type
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Kubernetes error: {0}")]
    K8s(String),

    #[error("Port forward error: {0}")]
    PortForward(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

// Implement Serialize for AppError to enable IPC transmission
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// Convert from sqlx errors
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Record not found".to_string()),
            _ => AppError::Database(err.to_string()),
        }
    }
}

// Convert from IO errors
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}


// Convert from anyhow errors
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

// Convert from serde_json errors
impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Validation(err.to_string())
    }
}

/// Result type alias for AppError
pub type AppResult<T> = Result<T, AppError>;
