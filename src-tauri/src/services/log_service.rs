//! Log aggregation service for collecting and streaming logs from multiple sources
//!
//! This service collects logs from:
//! - Backend (Rust): Internal log events
//! - Vite: Frontend dev server logs
//! - Browser: Console logs from the web app
//!
//! Logs are streamed to connected clients via Server-Sent Events (SSE).

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use serde::{Deserialize, Serialize};

/// Maximum number of logs to keep in memory
const MAX_LOG_BUFFER_SIZE: usize = 1000;

/// Log source identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogSource {
    Backend,
    Vite,
    Browser,
}

impl std::fmt::Display for LogSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogSource::Backend => write!(f, "backend"),
            LogSource::Vite => write!(f, "vite"),
            LogSource::Browser => write!(f, "browser"),
        }
    }
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Log, // For browser console.log
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Log => write!(f, "LOG"),
        }
    }
}

/// A single log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp in ISO 8601 format
    pub timestamp: String,
    /// Source of the log
    pub source: LogSource,
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
}

impl LogEntry {
    pub fn new(source: LogSource, level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            source,
            level,
            message: message.into(),
        }
    }

    pub fn backend_info(message: impl Into<String>) -> Self {
        Self::new(LogSource::Backend, LogLevel::Info, message)
    }

    pub fn backend_error(message: impl Into<String>) -> Self {
        Self::new(LogSource::Backend, LogLevel::Error, message)
    }

    pub fn backend_debug(message: impl Into<String>) -> Self {
        Self::new(LogSource::Backend, LogLevel::Debug, message)
    }

    pub fn backend_warn(message: impl Into<String>) -> Self {
        Self::new(LogSource::Backend, LogLevel::Warn, message)
    }
}

/// Request to add a log entry from external sources
#[derive(Debug, Deserialize)]
pub struct AddLogRequest {
    pub source: LogSource,
    pub level: LogLevel,
    pub message: String,
}

/// Log service for aggregating and streaming logs
#[derive(Clone)]
pub struct LogService {
    /// Circular buffer of log entries
    buffer: Arc<RwLock<VecDeque<LogEntry>>>,
    /// Broadcast channel for streaming logs to subscribers
    sender: broadcast::Sender<LogEntry>,
}

impl LogService {
    /// Create a new log service
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(256);
        Self {
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_LOG_BUFFER_SIZE))),
            sender,
        }
    }

    /// Add a log entry
    pub async fn add_log(&self, entry: LogEntry) {
        // Add to buffer
        {
            let mut buffer = self.buffer.write().await;
            if buffer.len() >= MAX_LOG_BUFFER_SIZE {
                buffer.pop_front();
            }
            buffer.push_back(entry.clone());
        }

        // Broadcast to subscribers (ignore errors if no subscribers)
        let _ = self.sender.send(entry);
    }

    /// Add a log from external source (e.g., Vite or Browser)
    pub async fn add_external_log(&self, request: AddLogRequest) {
        let entry = LogEntry::new(request.source, request.level, request.message);
        self.add_log(entry).await;
    }

    /// Get all logs in the buffer
    pub async fn get_logs(&self) -> Vec<LogEntry> {
        self.buffer.read().await.iter().cloned().collect()
    }

    /// Get logs filtered by source
    pub async fn get_logs_by_source(&self, source: LogSource) -> Vec<LogEntry> {
        self.buffer
            .read()
            .await
            .iter()
            .filter(|entry| entry.source == source)
            .cloned()
            .collect()
    }

    /// Clear all logs
    pub async fn clear_logs(&self) {
        self.buffer.write().await.clear();
    }

    /// Subscribe to log stream
    pub fn subscribe(&self) -> broadcast::Receiver<LogEntry> {
        self.sender.subscribe()
    }
}

impl Default for LogService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_get_logs() {
        let service = LogService::new();

        service.add_log(LogEntry::backend_info("Test message 1")).await;
        service.add_log(LogEntry::backend_error("Test message 2")).await;

        let logs = service.get_logs().await;
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].message, "Test message 1");
        assert_eq!(logs[1].message, "Test message 2");
    }

    #[tokio::test]
    async fn test_filter_by_source() {
        let service = LogService::new();

        service.add_log(LogEntry::backend_info("Backend log")).await;
        service.add_log(LogEntry::new(LogSource::Vite, LogLevel::Info, "Vite log")).await;
        service.add_log(LogEntry::new(LogSource::Browser, LogLevel::Log, "Browser log")).await;

        let backend_logs = service.get_logs_by_source(LogSource::Backend).await;
        assert_eq!(backend_logs.len(), 1);
        assert_eq!(backend_logs[0].message, "Backend log");
    }

    #[tokio::test]
    async fn test_buffer_limit() {
        let service = LogService::new();

        for i in 0..1100 {
            service.add_log(LogEntry::backend_info(format!("Message {}", i))).await;
        }

        let logs = service.get_logs().await;
        assert_eq!(logs.len(), MAX_LOG_BUFFER_SIZE);
        // First message should be #100 (0-99 were removed)
        assert_eq!(logs[0].message, "Message 100");
    }
}
