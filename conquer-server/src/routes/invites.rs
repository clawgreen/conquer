// conquer-server/src/routes/invites.rs — Invite system (T321-T323)

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::errors::ApiError;
use crate::jwt::Claims;

#[derive(Debug, Deserialize)]
pub struct CreateInviteRequest {
    pub max_uses: Option<u32>,
    pub expires_hours: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct InviteResponse {
    pub invite_code: String,
    pub game_id: String,
    pub game_name: String,
    pub max_uses: Option<u32>,
    pub uses: u32,
}

/// POST /api/games/:id/invites — Create invite (T321)
pub async fn create_invite(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
    Json(req): Json<CreateInviteRequest>,
) -> Result<Json<InviteResponse>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;

    let invite = state.store.create_invite(
        game_id, user_id, req.max_uses, req.expires_hours,
    ).await?;

    let game = state.store.get_game_info(game_id).await?;

    Ok(Json(InviteResponse {
        invite_code: invite.invite_code,
        game_id: game_id.to_string(),
        game_name: game.name,
        max_uses: invite.max_uses,
        uses: invite.uses,
    }))
}

/// GET /api/invites/:code — Validate invite (T322)
pub async fn get_invite(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<InviteResponse>, ApiError> {
    let (invite, game) = state.store.get_invite(&code).await?;

    Ok(Json(InviteResponse {
        invite_code: invite.invite_code,
        game_id: game.id.to_string(),
        game_name: game.name,
        max_uses: invite.max_uses,
        uses: invite.uses,
    }))
}

/// POST /api/invites/:code/accept — Accept invite and join game (T323)
pub async fn accept_invite(
    State(state): State<AppState>,
    claims: Claims,
    Path(code): Path<String>,
    Json(req): Json<super::games::JoinGameRequest>,
) -> Result<Json<super::games::JoinGameResponse>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;

    let (invite, _game) = state.store.get_invite(&code).await?;

    let player = state.store.join_game(
        invite.game_id,
        user_id,
        &req.nation_name,
        &req.leader_name,
        req.race,
        req.class,
        req.mark,
    ).await?;

    state.store.use_invite(&code).await?;

    // Broadcast
    state.ws_manager.broadcast(invite.game_id, crate::ws::ServerMessage::PlayerJoined {
        nation_id: player.nation_id,
        nation_name: req.nation_name.clone(),
        race: req.race,
    }).await;

    Ok(Json(super::games::JoinGameResponse {
        nation_id: player.nation_id,
        game_id: invite.game_id.to_string(),
        nation_name: req.nation_name,
    }))
}
