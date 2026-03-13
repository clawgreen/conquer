// conquer-server/src/routes/spectators.rs — Spectator mode (T428-T431)

use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::errors::ApiError;
use crate::jwt::Claims;
use crate::routes::state::MapResponse;

// ============================================================
// Handlers
// ============================================================

/// POST /api/games/:id/spectate — Join as spectator (T428)
pub async fn join_spectator(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;

    state.store.join_as_spectator(game_id, user_id).await?;

    Ok(Json(serde_json::json!({
        "status": "spectating",
        "game_id": game_id.to_string(),
    })))
}

/// DELETE /api/games/:id/spectate — Leave spectator mode (T428)
pub async fn leave_spectator(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;

    state.store.leave_spectator(game_id, user_id).await?;
    Ok(Json(serde_json::json!({"status": "left"})))
}

/// GET /api/games/:id/spectate/map — Spectator map view (T430)
pub async fn spectator_map(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<MapResponse>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;

    if !state.store.is_spectator(game_id, user_id).await {
        return Err(ApiError::Forbidden(
            "Not a spectator of this game".to_string(),
        ));
    }

    let visible = state.store.get_spectator_map(game_id).await?;
    let game_info = state.store.get_game_info(game_id).await?;

    Ok(Json(MapResponse {
        map_x: game_info.settings.map_x,
        map_y: game_info.settings.map_y,
        sectors: visible,
    }))
}
