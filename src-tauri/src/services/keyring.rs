//! Keyring service for secure password storage
//!
//! This module uses the system keyring (Keychain on macOS, Credential Manager on Windows,
//! Secret Service on Linux) to securely store database connection passwords.

use keyring::Entry;

use crate::error::{AppError, AppResult};

const SERVICE_NAME: &str = "zeni-x";

/// Keyring service for password management
pub struct KeyringService;

impl KeyringService {
    /// Generate keyring key for a connection
    fn get_key(connection_id: i64) -> String {
        format!("connection_{}", connection_id)
    }

    /// Save password for a connection
    pub fn save_password(connection_id: i64, password: &str) -> AppResult<()> {
        let key = Self::get_key(connection_id);
        let entry = Entry::new(SERVICE_NAME, &key)?;
        entry.set_password(password)?;
        Ok(())
    }

    /// Get password for a connection
    pub fn get_password(connection_id: i64) -> AppResult<Option<String>> {
        let key = Self::get_key(connection_id);
        let entry = Entry::new(SERVICE_NAME, &key)?;

        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Keyring(e.to_string())),
        }
    }

    /// Delete password for a connection
    pub fn delete_password(connection_id: i64) -> AppResult<()> {
        let key = Self::get_key(connection_id);
        let entry = Entry::new(SERVICE_NAME, &key)?;

        match entry.delete_credential() {
            Ok(_) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()), // Already deleted or never existed
            Err(e) => Err(AppError::Keyring(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests interact with the system keyring
    // They may require permissions on some systems

    #[test]
    #[ignore] // Ignore by default as it requires system keyring access
    fn test_password_lifecycle() {
        let test_id = 99999i64; // Use a high ID to avoid conflicts
        let password = "test_password_123";

        // Clean up any existing entry
        let _ = KeyringService::delete_password(test_id);

        // Save password
        KeyringService::save_password(test_id, password).unwrap();

        // Retrieve password
        let retrieved = KeyringService::get_password(test_id).unwrap();
        assert_eq!(retrieved, Some(password.to_string()));

        // Delete password
        KeyringService::delete_password(test_id).unwrap();

        // Verify deletion
        let deleted = KeyringService::get_password(test_id).unwrap();
        assert_eq!(deleted, None);
    }
}
