// conquer-server/src/routes/users.rs — User profile & settings (T409-T411)

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};

use conquer_db::models::*;
use crate::app::AppState;
use crate::errors::ApiError;
use crate::jwt::Claims;

// ============================================================
// Request/Response types
// ============================================================

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

// ============================================================
// Handlers
// ============================================================

/// GET /api/users/me — Get user profile (T409)
pub async fn get_profile(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<UserProfile>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;
    let profile = state.store.get_user_profile(user_id).await?;
    Ok(Json(profile))
}

/// PUT /api/users/me — Update profile (T410)
pub async fn update_profile(
    State(state): State<AppState>,
    claims: Claims,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<User>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;
    let user = state.store.update_user_profile(
        user_id,
        req.display_name.as_deref(),
        req.email.as_deref(),
    ).await?;
    Ok(Json(user))
}

/// PUT /api/users/me/password — Change password (T410)
pub async fn change_password(
    State(state): State<AppState>,
    claims: Claims,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;

    if req.new_password.len() < 6 {
        return Err(ApiError::BadRequest("New password must be at least 6 characters".to_string()));
    }

    state.store.change_password(user_id, &req.old_password, &req.new_password).await?;
    Ok(Json(serde_json::json!({"status": "password_changed"})))
}

/// GET /api/users/me/history — Game history (T411)
pub async fn get_history(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<Vec<GameHistoryEntry>>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;
    let profile = state.store.get_user_profile(user_id).await?;
    Ok(Json(profile.game_history))
}
