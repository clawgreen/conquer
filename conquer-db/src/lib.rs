// conquer-db: Game persistence layer (Phase 3)
//
// Provides:
// - In-memory game store for testing (no Postgres required)
// - Repository traits for future Postgres implementation
// - User management with argon2 password hashing
// - Game lifecycle management

pub mod error;
pub mod models;
pub mod store;
pub mod auth;

#[cfg(feature = "postgres")]
pub mod pg;

pub use error::DbError;
pub use models::*;
pub use store::{GameStore, PublicNationInfo, ScoreEntry, ServerStats};
pub use auth::AuthManager;
