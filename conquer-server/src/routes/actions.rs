// conquer-server/src/routes/actions.rs — Action endpoints (T305-T311)

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use conquer_core::actions::Action;
use crate::app::AppState;
use crate::errors::ApiError;
use crate::jwt::Claims;
use crate::ws::ServerMessage;

// ============================================================
// Request/Response types
// ============================================================

#[derive(Debug, Deserialize)]
pub struct SubmitActionsRequest {
    pub actions: Vec<Action>,
}

#[derive(Debug, Serialize)]
pub struct SubmittedActionResponse {
    pub id: String,
    pub action: Action,
    pub order: u32,
}

#[derive(Debug, Serialize)]
pub struct TurnAdvanceResponse {
    pub new_turn: i16,
    pub message: String,
}

// ============================================================
// Handlers
// ============================================================

/// POST /api/games/:id/actions — Submit actions (T305)
pub async fn submit_actions(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
    Json(req): Json<SubmitActionsRequest>,
) -> Result<Json<Vec<SubmittedActionResponse>>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;
    let player = state.store.get_player(game_id, user_id).await?;

    // Validate: reject engine-only actions from player API
    let mut results = Vec::new();
    for action in req.actions {
        if !action.is_player_action() {
            return Err(ApiError::BadRequest(format!(
                "Action {:?} is engine-internal and cannot be submitted by players",
                std::mem::discriminant(&action)
            )));
        }
        let submitted = state.store.submit_action(game_id, player.nation_id, action).await?;
        results.push(SubmittedActionResponse {
            id: submitted.id.to_string(),
            action: submitted.action,
            order: submitted.order,
        });
    }

    Ok(Json(results))
}

/// GET /api/games/:id/actions — Get own submitted actions (T306)
pub async fn get_actions(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<Vec<SubmittedActionResponse>>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;
    let player = state.store.get_player(game_id, user_id).await?;

    let actions = state.store.get_actions(game_id, player.nation_id).await?;
    let results: Vec<SubmittedActionResponse> = actions.into_iter()
        .map(|a| SubmittedActionResponse {
            id: a.id.to_string(),
            action: a.action,
            order: a.order,
        })
        .collect();

    Ok(Json(results))
}

/// DELETE /api/games/:id/actions/:action_id — Retract action (T307)
pub async fn retract_action(
    State(state): State<AppState>,
    claims: Claims,
    Path((game_id, action_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;
    let player = state.store.get_player(game_id, user_id).await?;

    state.store.retract_action(game_id, action_id, player.nation_id).await?;
    Ok(Json(serde_json::json!({"status": "retracted"})))
}

/// POST /api/games/:id/end-turn — Mark nation done (T308)
pub async fn end_turn(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;
    let player = state.store.get_player(game_id, user_id).await?;

    state.store.set_player_done(game_id, user_id, true).await?;

    // Get nation name for broadcast
    let nation = state.store.get_nation(game_id, player.nation_id).await?;

    // Broadcast player done
    state.ws_manager.broadcast(game_id, ServerMessage::PlayerDone {
        nation_id: player.nation_id,
        nation_name: nation.name.clone(),
    }).await;

    // Check if all players done → auto advance
    if state.store.all_players_done(game_id).await? {
        let new_turn = state.store.run_turn(game_id).await?;
        // Broadcast turn end + system chat message (T394)
        state.ws_manager.broadcast(game_id, ServerMessage::TurnEnd {
            old_turn: new_turn - 1,
            new_turn,
        }).await;
        let season = ["Winter", "Spring", "Summer", "Fall"][(new_turn % 4) as usize];
        let year = (new_turn as i32 + 3) / 4;
        state.ws_manager.broadcast(game_id, ServerMessage::ChatMessage {
            sender_nation_id: None,
            sender_name: "SYSTEM".to_string(),
            channel: "public".to_string(),
            content: format!("━━━ Turn {} ({}, Year {}) has begun ━━━", new_turn, season, year),
            timestamp: chrono::Utc::now().to_rfc3339(),
            is_system: true,
        }).await;
        return Ok(Json(serde_json::json!({
            "status": "turn_advanced",
            "new_turn": new_turn,
        })));
    }

    Ok(Json(serde_json::json!({
        "status": "done",
        "nation_id": player.nation_id,
    })))
}

/// POST /api/games/:id/run-turn — Admin turn advance (T309)
pub async fn run_turn(
    State(state): State<AppState>,
    _claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<TurnAdvanceResponse>, ApiError> {
    // For now allow any authenticated user; in production check admin
    // or implement turn timer auto-advance
    let new_turn = state.store.run_turn(game_id).await?;

    state.ws_manager.broadcast(game_id, ServerMessage::TurnEnd {
        old_turn: new_turn - 1,
        new_turn,
    }).await;

    // Broadcast system chat message (T394)
    let season = ["Winter", "Spring", "Summer", "Fall"][(new_turn % 4) as usize];
    let year = (new_turn as i32 + 3) / 4;
    state.ws_manager.broadcast(game_id, ServerMessage::ChatMessage {
        sender_nation_id: None,
        sender_name: "SYSTEM".to_string(),
        channel: "public".to_string(),
        content: format!("━━━ Turn {} ({}, Year {}) has begun ━━━", new_turn, season, year),
        timestamp: chrono::Utc::now().to_rfc3339(),
        is_system: true,
    }).await;

    Ok(Json(TurnAdvanceResponse {
        new_turn,
        message: format!("Turn {} has begun", new_turn),
    }))
}
