// conquer-server/src/routes/admin.rs — Admin dashboard & game management (T423-T427)

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use conquer_db::models::*;
use conquer_db::ServerStats;
use crate::app::AppState;
use crate::errors::ApiError;
use crate::jwt::Claims;

// ============================================================
// Request types
// ============================================================

#[derive(Debug, Deserialize)]
pub struct SetGameStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct KickPlayerRequest {
    pub nation_id: u8,
}

#[derive(Debug, Deserialize)]
pub struct RollbackRequest {
    pub target_turn: i16,
}

// ============================================================
// Response types
// ============================================================

#[derive(Debug, Serialize)]
pub struct AdminPlayerInfo {
    pub user_id: String,
    pub nation_id: u8,
    pub nation_name: String,
    pub race: char,
    pub class: i16,
    pub is_done: bool,
    pub score: i64,
    pub joined_at: String,
}

#[derive(Debug, Serialize)]
pub struct TurnSnapshotInfo {
    pub turn: i16,
    pub created_at: String,
}

// ============================================================
// Helpers
// ============================================================

/// Verify user is game creator (admin of that game)
async fn require_game_admin(
    state: &AppState,
    claims: &Claims,
    game_id: Uuid,
) -> Result<Uuid, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;
    let is_admin = state.store.is_game_admin(game_id, user_id).await
        .map_err(|e| ApiError::from(e))?;
    // Also allow global admins
    let is_global = state.store.is_admin(user_id).await;
    if !is_admin && !is_global {
        return Err(ApiError::Forbidden("Only game creator or site admin can perform this action".to_string()));
    }
    Ok(user_id)
}

// ============================================================
// Handlers
// ============================================================

/// GET /api/games/:id/admin/players — List players with admin info (T423)
pub async fn admin_list_players(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<Vec<AdminPlayerInfo>>, ApiError> {
    require_game_admin(&state, &claims, game_id).await?;

    let players = state.store.list_players(game_id).await?;
    let game_state = state.store.get_game_state(game_id).await?;

    let result: Vec<AdminPlayerInfo> = players.iter().map(|p| {
        let nation = &game_state.nations[p.nation_id as usize];
        AdminPlayerInfo {
            user_id: p.user_id.to_string(),
            nation_id: p.nation_id,
            nation_name: nation.name.clone(),
            race: nation.race,
            class: nation.class,
            is_done: p.is_done_this_turn,
            score: nation.score,
            joined_at: p.joined_at.to_rfc3339(),
        }
    }).collect();

    Ok(Json(result))
}

/// POST /api/games/:id/admin/kick — Kick a player (T423)
pub async fn admin_kick_player(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
    Json(req): Json<KickPlayerRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_game_admin(&state, &claims, game_id).await?;
    state.store.kick_player(game_id, req.nation_id).await?;

    // Broadcast system message
    state.ws_manager.broadcast(game_id, crate::ws::ServerMessage::ChatMessage {
        sender_nation_id: None,
        sender_name: "SYSTEM".to_string(),
        channel: "public".to_string(),
        content: format!("⚠ Nation {} has been removed from the game by an admin.", req.nation_id),
        timestamp: chrono::Utc::now().to_rfc3339(),
        is_system: true,
    }).await;

    Ok(Json(serde_json::json!({"status": "kicked", "nation_id": req.nation_id})))
}

/// POST /api/games/:id/admin/status — Pause/resume/complete game (T423)
pub async fn admin_set_status(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
    Json(req): Json<SetGameStatusRequest>,
) -> Result<Json<GameInfo>, ApiError> {
    require_game_admin(&state, &claims, game_id).await?;

    let status = match req.status.as_str() {
        "active" => GameStatus::Active,
        "paused" => GameStatus::Paused,
        "completed" => GameStatus::Completed,
        "waiting_for_players" => GameStatus::WaitingForPlayers,
        _ => return Err(ApiError::BadRequest(format!("Invalid status: {}", req.status))),
    };

    let info = state.store.set_game_status(game_id, status).await?;

    // Broadcast status change
    state.ws_manager.broadcast(game_id, crate::ws::ServerMessage::SystemMessage {
        content: format!("Game status changed to: {}", req.status),
    }).await;

    Ok(Json(info))
}

/// POST /api/games/:id/admin/advance-turn — Force turn advance (T423)
pub async fn admin_advance_turn(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_game_admin(&state, &claims, game_id).await?;

    let new_turn = state.store.run_turn(game_id).await?;

    state.ws_manager.broadcast(game_id, crate::ws::ServerMessage::TurnEnd {
        old_turn: new_turn - 1,
        new_turn,
    }).await;

    let season = ["Winter", "Spring", "Summer", "Fall"][(new_turn % 4) as usize];
    let year = (new_turn as i32 + 3) / 4;
    state.ws_manager.broadcast(game_id, crate::ws::ServerMessage::ChatMessage {
        sender_nation_id: None,
        sender_name: "SYSTEM".to_string(),
        channel: "public".to_string(),
        content: format!("━━━ Turn {} ({}, Year {}) advanced by admin ━━━", new_turn, season, year),
        timestamp: chrono::Utc::now().to_rfc3339(),
        is_system: true,
    }).await;

    Ok(Json(serde_json::json!({"status": "turn_advanced", "new_turn": new_turn})))
}

/// GET /api/games/:id/admin/snapshots — List turn snapshots (T426)
pub async fn admin_list_snapshots(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<Vec<TurnSnapshotInfo>>, ApiError> {
    require_game_admin(&state, &claims, game_id).await?;

    let snapshots = state.store.list_turn_snapshots(game_id).await?;
    let result: Vec<TurnSnapshotInfo> = snapshots.into_iter().map(|(turn, created_at)| {
        TurnSnapshotInfo { turn, created_at: created_at.to_rfc3339() }
    }).collect();

    Ok(Json(result))
}

/// POST /api/games/:id/admin/rollback — Rollback to previous turn (T426)
pub async fn admin_rollback(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
    Json(req): Json<RollbackRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    require_game_admin(&state, &claims, game_id).await?;

    let turn = state.store.rollback_turn(game_id, req.target_turn).await?;

    state.ws_manager.broadcast(game_id, crate::ws::ServerMessage::SystemMessage {
        content: format!("⚠ Game rolled back to turn {} by admin", turn),
    }).await;

    Ok(Json(serde_json::json!({"status": "rolled_back", "turn": turn})))
}

/// PUT /api/games/:id/settings — Update game settings (T415-T418)
pub async fn update_game_settings(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
    Json(settings): Json<GameSettings>,
) -> Result<Json<GameInfo>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;
    let info = state.store.update_game_settings(game_id, user_id, settings).await?;
    Ok(Json(info))
}

/// GET /api/admin/stats — Server status dashboard (T427)
pub async fn server_stats(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<ServerStats>, ApiError> {
    // Allow any authenticated user (could restrict to admins)
    let stats = state.store.server_stats().await;
    Ok(Json(stats))
}
