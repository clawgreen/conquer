// conquer-server/src/routes/notifications.rs — In-app notifications (T432-T434)

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::errors::ApiError;
use crate::jwt::Claims;
use conquer_db::models::*;

// ============================================================
// Request types
// ============================================================

#[derive(Debug, Deserialize)]
pub struct NotificationQuery {
    #[serde(default)]
    pub unread_only: bool,
}

// ============================================================
// Handlers
// ============================================================

/// GET /api/notifications — Get user notifications (T432)
pub async fn get_notifications(
    State(state): State<AppState>,
    claims: Claims,
    Query(query): Query<NotificationQuery>,
) -> Result<Json<Vec<Notification>>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;
    let notifs = state
        .store
        .get_notifications(user_id, query.unread_only)
        .await;
    Ok(Json(notifs))
}

/// POST /api/notifications/:id/read — Mark notification as read (T432)
pub async fn mark_read(
    State(state): State<AppState>,
    claims: Claims,
    Path(notif_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;
    state
        .store
        .mark_notification_read(user_id, notif_id)
        .await?;
    Ok(Json(serde_json::json!({"status": "read"})))
}

/// POST /api/notifications/read-all — Mark all as read (T432)
pub async fn mark_all_read(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;
    state.store.mark_all_read(user_id).await;
    Ok(Json(serde_json::json!({"status": "all_read"})))
}

/// GET /api/notifications/preferences — Get notification preferences (T434)
pub async fn get_preferences(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<NotificationPreferences>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;
    let prefs = state.store.get_notification_prefs(user_id).await;
    Ok(Json(prefs))
}

/// PUT /api/notifications/preferences — Update notification preferences (T434)
pub async fn set_preferences(
    State(state): State<AppState>,
    claims: Claims,
    Json(prefs): Json<NotificationPreferences>,
) -> Result<Json<NotificationPreferences>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;
    state
        .store
        .set_notification_prefs(user_id, prefs.clone())
        .await;
    Ok(Json(prefs))
}
