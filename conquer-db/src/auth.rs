use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};
use crate::error::DbError;

/// Authentication helper for password hashing and verification
pub struct AuthManager;

impl AuthManager {
    /// Hash a password with argon2
    pub fn hash_password(password: &str) -> Result<String, DbError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| DbError::Internal(format!("Password hash error: {}", e)))?;
        Ok(hash.to_string())
    }

    /// Verify a password against a hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool, DbError> {
        let parsed_hash = argon2::PasswordHash::new(hash)
            .map_err(|e| DbError::Internal(format!("Invalid hash format: {}", e)))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "test_password_123";
        let hash = AuthManager::hash_password(password).unwrap();
        assert!(AuthManager::verify_password(password, &hash).unwrap());
        assert!(!AuthManager::verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_different_hashes_for_same_password() {
        let password = "same_password";
        let hash1 = AuthManager::hash_password(password).unwrap();
        let hash2 = AuthManager::hash_password(password).unwrap();
        // Different salts = different hashes
        assert_ne!(hash1, hash2);
        // But both verify
        assert!(AuthManager::verify_password(password, &hash1).unwrap());
        assert!(AuthManager::verify_password(password, &hash2).unwrap());
    }
}
