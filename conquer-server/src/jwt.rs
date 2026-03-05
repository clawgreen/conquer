use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// User ID
    pub sub: String,
    /// Username
    pub username: String,
    /// Is admin
    pub is_admin: bool,
    /// Expiry timestamp
    pub exp: i64,
    /// Issued at
    pub iat: i64,
}

/// JWT manager
#[derive(Clone)]
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiry_hours: u64,
}

impl JwtManager {
    pub fn new(secret: &str, expiry_hours: u64) -> Self {
        JwtManager {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            expiry_hours,
        }
    }

    /// Create a new JWT token for a user
    pub fn create_token(
        &self,
        user_id: Uuid,
        username: &str,
        is_admin: bool,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            username: username.to_string(),
            is_admin,
            exp: (now + Duration::hours(self.expiry_hours as i64)).timestamp(),
            iat: now.timestamp(),
        };
        encode(&Header::default(), &claims, &self.encoding_key)
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &Validation::default())?;
        Ok(token_data.claims)
    }

    /// Extract user_id from claims
    pub fn user_id_from_claims(claims: &Claims) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(&claims.sub)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_create_and_validate() {
        let jwt = JwtManager::new("test-secret", 24);
        let user_id = Uuid::new_v4();
        let token = jwt.create_token(user_id, "testuser", false).unwrap();

        let claims = jwt.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.username, "testuser");
        assert!(!claims.is_admin);
    }

    #[test]
    fn test_jwt_invalid_token() {
        let jwt = JwtManager::new("test-secret", 24);
        let result = jwt.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_wrong_secret() {
        let jwt1 = JwtManager::new("secret-1", 24);
        let jwt2 = JwtManager::new("secret-2", 24);
        let user_id = Uuid::new_v4();

        let token = jwt1.create_token(user_id, "testuser", false).unwrap();
        let result = jwt2.validate_token(&token);
        assert!(result.is_err());
    }
}
