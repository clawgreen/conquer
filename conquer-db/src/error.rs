use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Game full: no available nation slots")]
    GameFull,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
