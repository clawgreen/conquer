// conquer-server/src/routes/state.rs — Game state endpoints (T296-T304)

use axum::extract::{Path, State};
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use conquer_core::*;
use crate::app::AppState;
use crate::errors::ApiError;
use crate::jwt::Claims;

// ============================================================
// Response types
// ============================================================

#[derive(Debug, Serialize)]
pub struct MapResponse {
    pub map_x: usize,
    pub map_y: usize,
    pub sectors: Vec<Vec<Option<Sector>>>,
}

#[derive(Debug, Serialize)]
pub struct ArmyInfo {
    pub index: u8,
    pub unit_type: u8,
    pub x: u8,
    pub y: u8,
    pub movement: u8,
    pub soldiers: i64,
    pub status: u8,
}

#[derive(Debug, Serialize)]
pub struct NavyInfo {
    pub index: u8,
    pub warships: u16,
    pub merchant: u16,
    pub galleys: u16,
    pub x: u8,
    pub y: u8,
    pub movement: u8,
    pub crew: u8,
    pub people: u8,
}

// ============================================================
// Helper: resolve nation_id from claims
// ============================================================

async fn resolve_nation(
    state: &AppState,
    claims: &Claims,
    game_id: Uuid,
) -> Result<u8, ApiError> {
    let user_id = crate::jwt::JwtManager::user_id_from_claims(claims)
        .map_err(|_| ApiError::Unauthorized("Invalid user ID".to_string()))?;
    let player = state.store.get_player(game_id, user_id).await?;
    Ok(player.nation_id)
}

// ============================================================
// Handlers
// ============================================================

/// GET /api/games/:id/map — Visible map with fog of war (T296)
pub async fn get_map(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<MapResponse>, ApiError> {
    let nation_id = resolve_nation(&state, &claims, game_id).await?;
    let visible = state.store.get_visible_map(game_id, nation_id).await?;
    let game_info = state.store.get_game_info(game_id).await?;

    Ok(Json(MapResponse {
        map_x: game_info.settings.map_x,
        map_y: game_info.settings.map_y,
        sectors: visible,
    }))
}

/// GET /api/games/:id/nation — Own nation data (T297)
pub async fn get_nation(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<Nation>, ApiError> {
    let nation_id = resolve_nation(&state, &claims, game_id).await?;
    let nation = state.store.get_nation(game_id, nation_id).await?;
    Ok(Json(nation))
}

/// GET /api/games/:id/nations — Public nation info (T298)
pub async fn get_nations(
    State(state): State<AppState>,
    _claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<Vec<conquer_db::store::PublicNationInfo>>, ApiError> {
    let nations = state.store.get_public_nations(game_id).await?;
    Ok(Json(nations))
}

/// GET /api/games/:id/armies — Own armies (T299)
pub async fn get_armies(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<Vec<ArmyInfo>>, ApiError> {
    let nation_id = resolve_nation(&state, &claims, game_id).await?;
    let nation = state.store.get_nation(game_id, nation_id).await?;

    let armies: Vec<ArmyInfo> = nation.armies.iter().enumerate()
        .filter(|(_, a)| a.soldiers > 0)
        .map(|(i, a)| ArmyInfo {
            index: i as u8,
            unit_type: a.unit_type,
            x: a.x,
            y: a.y,
            movement: a.movement,
            soldiers: a.soldiers,
            status: a.status,
        })
        .collect();

    Ok(Json(armies))
}

/// GET /api/games/:id/navies — Own navies (T300)
pub async fn get_navies(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<Vec<NavyInfo>>, ApiError> {
    let nation_id = resolve_nation(&state, &claims, game_id).await?;
    let nation = state.store.get_nation(game_id, nation_id).await?;

    let navies: Vec<NavyInfo> = nation.navies.iter().enumerate()
        .filter(|(_, n)| n.has_ships())
        .map(|(i, n)| NavyInfo {
            index: i as u8,
            warships: n.warships,
            merchant: n.merchant,
            galleys: n.galleys,
            x: n.x,
            y: n.y,
            movement: n.movement,
            crew: n.crew,
            people: n.people,
        })
        .collect();

    Ok(Json(navies))
}

/// GET /api/games/:id/sector/:x/:y — Sector details (T301)
pub async fn get_sector(
    State(state): State<AppState>,
    claims: Claims,
    Path((game_id, x, y)): Path<(Uuid, usize, usize)>,
) -> Result<Json<Option<Sector>>, ApiError> {
    let nation_id = resolve_nation(&state, &claims, game_id).await?;
    let visible = state.store.get_visible_map(game_id, nation_id).await?;

    if x < visible.len() && y < visible[0].len() {
        Ok(Json(visible[x][y].clone()))
    } else {
        Err(ApiError::BadRequest("Coordinates out of range".to_string()))
    }
}

/// GET /api/games/:id/news — Current turn news (T302)
pub async fn get_news(
    State(state): State<AppState>,
    _claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<Vec<conquer_db::models::NewsEntry>>, ApiError> {
    let game_info = state.store.get_game_info(game_id).await?;
    let news = state.store.get_news(game_id, Some(game_info.current_turn)).await?;
    Ok(Json(news))
}

/// GET /api/games/:id/scores — Scoreboard (T303)
pub async fn get_scores(
    State(state): State<AppState>,
    _claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<Vec<conquer_db::store::ScoreEntry>>, ApiError> {
    let scores = state.store.get_scores(game_id).await?;
    Ok(Json(scores))
}

/// GET /api/games/:id/budget — Budget/spreadsheet (T304)
pub async fn get_budget(
    State(state): State<AppState>,
    claims: Claims,
    Path(game_id): Path<Uuid>,
) -> Result<Json<Spreadsheet>, ApiError> {
    let nation_id = resolve_nation(&state, &claims, game_id).await?;
    let budget = state.store.get_budget(game_id, nation_id).await?;
    Ok(Json(budget))
}
