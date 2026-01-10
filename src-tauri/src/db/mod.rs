//! Database module
//!
//! This module contains database-related functionality including:
//! - SQLite local storage
//! - Data models

pub mod models;
pub mod sqlite;

pub use sqlite::SqlitePool;
