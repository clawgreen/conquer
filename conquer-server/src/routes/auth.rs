// conquer-server/src/routes/auth.rs — Authentication endpoints (T283-T286)

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::errors::ApiError;

// ============================================================
// Request/Response types
// ============================================================

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub is_admin: bool,
}

// ============================================================
// Handlers
// ============================================================

/// POST /api/auth/register (T283)
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Validate
    if req.username.len() < 3 {
        return Err(ApiError::BadRequest("Username must be at least 3 characters".to_string()));
    }
    if req.password.len() < 6 {
        return Err(ApiError::BadRequest("Password must be at least 6 characters".to_string()));
    }
    if !req.email.contains('@') {
        return Err(ApiError::BadRequest("Invalid email".to_string()));
    }

    let user = state.store.create_user(
        &req.username,
        &req.email,
        &req.password,
        req.display_name.as_deref(),
    ).await?;

    let token = state.jwt.create_token(user.id, &user.username, user.is_admin)
        .map_err(|e| ApiError::Internal(format!("Token creation failed: {}", e)))?;

    Ok(Json(AuthResponse {
        token,
        user_id: user.id.to_string(),
        username: user.username,
        display_name: user.display_name,
        is_admin: user.is_admin,
    }))
}

/// POST /api/auth/login (T284)
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let user = state.store.authenticate_user(&req.username, &req.password).await?;

    let token = state.jwt.create_token(user.id, &user.username, user.is_admin)
        .map_err(|e| ApiError::Internal(format!("Token creation failed: {}", e)))?;

    Ok(Json(AuthResponse {
        token,
        user_id: user.id.to_string(),
        username: user.username,
        display_name: user.display_name,
        is_admin: user.is_admin,
    }))
}
