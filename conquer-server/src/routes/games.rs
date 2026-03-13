// conquer-server/src/routes/games.rs — Game management endpoints (T288-T295)

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::errors::ApiError;
use crate::jwt::Claims;
use conquer_db::models::*;

// ============================================================
// Request/Response types
// ============================================================

#[derive(Debug, Deserialize)]
pub struct CreateGameRequest {
    pub name: String,
    #[serde(default)]
    pub settings: Option<GameSettings>,
}

#[derive(Debug, Deserialize)]
pub struct ListGamesQuery {
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct JoinGameRequest {
    pub nation_name: String,
    pub leader_name: String,
    pub race: char,
    #[serde(default = "default_class")]
    pub class: i16,
    #[serde(default = "default_mark")]
    pub mark: char,
}

fn default_class() -> i16 {
    1
}
fn default_mark() -> char {
    '*'
}

#[derive(Debug, Serialize)]
pub struct JoinGameResponse {
    pub nation_id: u8,
    pub game_id: String,
    pub nation_name: String,
}

// ============================================================
// Handlers
// ============================================================

/// POST /api/games — Create a new game (T288)
pub async fn create_game(
    State(state): State<AppState>,
    claims: Claims,
    Json(req): Json<CreateGameRequest>,
) -> Result<Json<GameInfo>, ApiError> {
    if req.name.is_empty() {
        return Err(ApiError::BadRequest("Game name required".to_string()));
    }

    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;

    let mut settings = req.settings.unwrap_or_default();
    settings.creator_id = Some(user_id);
    let game = state.store.create_game(&req.name, settings).await?;

    // Broadcast not needed yet — no one is connected

    Ok(Json(game))
}

/// Game list entry with optional user's nation info
#[derive(Serialize)]
pub struct GameListEntry {
    #[serde(flatten)]
    pub info: GameInfo,
    pub my_nation_id: Option<u8>,
}

/// GET /api/games — List games (T289)
pub async fn list_games(
    State(state): State<AppState>,
    claims: Claims,
    Query(query): Query<ListGamesQuery>,
) -> Result<Json<Vec<GameListEntry>>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims).ok();

    let status_filter = query.status.and_then(|s| match s.as_str() {
        "waiting" | "waiting_for_players" => Some(GameStatus::WaitingForPlayers),
        "active" => Some(GameStatus::Active),
        "paused" => Some(GameStatus::Paused),
        "completed" => Some(GameStatus::Completed),
        _ => None,
    });

    let games = state.store.list_games(status_filter).await;

    let mut entries = Vec::new();
    for info in games {
        let my_nation_id = if let Some(uid) = user_id {
            state.store.get_player_nation_id(info.id, uid).await.ok()
        } else {
            None
        };
        entries.push(GameListEntry { info, my_nation_id });
    }
    Ok(Json(entries))
}

/// GET /api/games/:id — Game details (T290)
pub async fn get_game(
    State(state): State<AppState>,
    _claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<GameInfo>, ApiError> {
    let game = state.store.get_game_info(game_id).await?;
    Ok(Json(game))
}

/// POST /api/games/:id/join — Join game as new nation (T291)
pub async fn join_game(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
    Json(req): Json<JoinGameRequest>,
) -> Result<Json<JoinGameResponse>, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(&claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID in token".to_string()))?;

    if req.nation_name.is_empty() {
        return Err(ApiError::BadRequest("Nation name required".to_string()));
    }

    let player = state
        .store
        .join_game(
            game_id,
            user_id,
            &req.nation_name,
            &req.leader_name,
            req.race,
            req.class,
            req.mark,
        )
        .await?;

    // Broadcast player joined
    state
        .ws_manager
        .broadcast(
            game_id,
            crate::ws::ServerMessage::PlayerJoined {
                nation_id: player.nation_id,
                nation_name: req.nation_name.clone(),
                race: req.race,
            },
        )
        .await;

    // Broadcast system chat message (T395)
    let race_name = match req.race {
        'H' => "Human",
        'E' => "Elf",
        'D' => "Dwarf",
        'O' => "Orc",
        'L' => "Lizard",
        'P' => "Pirate",
        'S' => "Savage",
        'N' => "Nomad",
        _ => "Unknown",
    };
    let class_name = match req.class {
        0 => "Monster",
        1 => "King",
        2 => "Emperor",
        3 => "Wizard",
        4 => "Priest",
        5 => "Pirate",
        6 => "Trader",
        7 => "Warlord",
        _ => "Adventurer",
    };
    state
        .ws_manager
        .broadcast(
            game_id,
            crate::ws::ServerMessage::ChatMessage {
                sender_nation_id: None,
                sender_name: "SYSTEM".to_string(),
                channel: "public".to_string(),
                content: format!(
                    "⚔ The nation of {} ({} {}) has entered the world!",
                    req.nation_name, race_name, class_name
                ),
                timestamp: chrono::Utc::now().to_rfc3339(),
                is_system: true,
            },
        )
        .await;

    Ok(Json(JoinGameResponse {
        nation_id: player.nation_id,
        game_id: game_id.to_string(),
        nation_name: req.nation_name,
    }))
}

/// GET /api/games/public — List public games for the browser (T422)
pub async fn list_public_games(
    State(state): State<AppState>,
    _claims: Claims,
) -> Result<Json<Vec<GameInfo>>, ApiError> {
    let games = state.store.list_public_games().await;
    Ok(Json(games))
}

/// DELETE /api/games/:id — Archive game (T293, admin only)
pub async fn delete_game(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !claims.is_admin {
        return Err(ApiError::Forbidden("Admin only".to_string()));
    }
    state.store.delete_game(game_id).await?;
    Ok(Json(serde_json::json!({"status": "archived"})))
}
