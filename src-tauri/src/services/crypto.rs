//! Cryptography service for password encryption/decryption
//!
//! Uses AES-256-GCM for symmetric encryption of passwords stored in SQLite.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::Rng;

use crate::error::{AppError, AppResult};

/// Fixed encryption key (in production, this should come from secure storage or be derived)
/// 32 bytes for AES-256
const ENCRYPTION_KEY: &[u8; 32] = b"zeni-x-secret-key-2024-secure!@#";

/// Nonce size for AES-GCM (12 bytes)
const NONCE_SIZE: usize = 12;

/// Crypto service for password encryption
pub struct CryptoService;

impl CryptoService {
    /// Encrypt a password
    ///
    /// Returns base64-encoded string containing: nonce (12 bytes) + ciphertext
    pub fn encrypt(plaintext: &str) -> AppResult<String> {
        if plaintext.is_empty() {
            return Ok(String::new());
        }

        let cipher = Aes256Gcm::new_from_slice(ENCRYPTION_KEY)
            .map_err(|e| AppError::Crypto(format!("Failed to create cipher: {}", e)))?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| AppError::Crypto(format!("Encryption failed: {}", e)))?;

        // Combine nonce + ciphertext and encode as base64
        let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(&combined))
    }

    /// Decrypt an encrypted password
    ///
    /// Input should be base64-encoded string containing: nonce (12 bytes) + ciphertext
    pub fn decrypt(encrypted: &str) -> AppResult<String> {
        if encrypted.is_empty() {
            return Ok(String::new());
        }

        let cipher = Aes256Gcm::new_from_slice(ENCRYPTION_KEY)
            .map_err(|e| AppError::Crypto(format!("Failed to create cipher: {}", e)))?;

        // Decode base64
        let combined = BASE64
            .decode(encrypted)
            .map_err(|e| AppError::Crypto(format!("Failed to decode base64: {}", e)))?;

        if combined.len() < NONCE_SIZE {
            return Err(AppError::Crypto("Invalid encrypted data: too short".to_string()));
        }

        // Extract nonce and ciphertext
        let nonce = Nonce::from_slice(&combined[..NONCE_SIZE]);
        let ciphertext = &combined[NONCE_SIZE..];

        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| AppError::Crypto(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| AppError::Crypto(format!("Invalid UTF-8 in decrypted data: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let password = "my_secret_password_123!@#";

        let encrypted = CryptoService::encrypt(password).unwrap();
        assert!(!encrypted.is_empty());
        assert_ne!(encrypted, password);

        let decrypted = CryptoService::decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, password);
    }

    #[test]
    fn test_empty_password() {
        let encrypted = CryptoService::encrypt("").unwrap();
        assert!(encrypted.is_empty());

        let decrypted = CryptoService::decrypt("").unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_special_characters() {
        let password = "p@$$w0rd/with+special=chars&more!";

        let encrypted = CryptoService::encrypt(password).unwrap();
        let decrypted = CryptoService::decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, password);
    }

    #[test]
    fn test_unicode_password() {
        let password = "密码测试🔐";

        let encrypted = CryptoService::encrypt(password).unwrap();
        let decrypted = CryptoService::decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, password);
    }
}
