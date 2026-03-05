// conquer-db/src/pg.rs — Postgres persistence layer (write-through cache)
//
// All functions here persist in-memory state to Postgres.
// On startup, `hydrate()` loads everything back from Postgres into memory.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use conquer_core::GameState;

use crate::error::DbError;
use crate::models::*;
use crate::store::ManagedGame;

// ============================================================
// Migration
// ============================================================

pub async fn run_migrations(pool: &PgPool) -> Result<(), DbError> {
    // Check if tables exist already (idempotent)
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'users' AND table_schema = 'public'"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Migration check failed: {}", e)))?;

    if row.0 == 0 {
        sqlx::raw_sql(include_str!("../migrations/001_initial_schema.sql"))
            .execute(pool)
            .await
            .map_err(|e| DbError::Internal(format!("Migration failed: {}", e)))?;
        tracing::info!("Applied initial schema migration");
    } else {
        tracing::info!("Schema already exists, skipping migration");
    }
    Ok(())
}

// ============================================================
// User persistence
// ============================================================

pub async fn save_user(pool: &PgPool, user: &User) -> Result<(), DbError> {
    sqlx::query(
        r#"INSERT INTO users (id, username, email, password_hash, display_name, created_at, is_admin)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           ON CONFLICT (id) DO UPDATE SET
             email = EXCLUDED.email,
             password_hash = EXCLUDED.password_hash,
             display_name = EXCLUDED.display_name,
             is_admin = EXCLUDED.is_admin"#,
    )
    .bind(user.id)
    .bind(&user.username)
    .bind(&user.email)
    .bind(&user.password_hash)
    .bind(&user.display_name)
    .bind(user.created_at)
    .bind(user.is_admin)
    .execute(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Failed to save user: {}", e)))?;
    Ok(())
}

pub async fn load_all_users(pool: &PgPool) -> Result<Vec<User>, DbError> {
    let rows = sqlx::query_as::<_, UserRow>(
        "SELECT id, username, email, password_hash, display_name, created_at, is_admin FROM users"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Failed to load users: {}", e)))?;

    Ok(rows.into_iter().map(|r| User {
        id: r.id,
        username: r.username,
        email: r.email,
        password_hash: r.password_hash,
        display_name: r.display_name,
        created_at: r.created_at,
        is_admin: r.is_admin,
    }).collect())
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    username: String,
    email: String,
    password_hash: String,
    display_name: String,
    created_at: DateTime<Utc>,
    is_admin: bool,
}

// ============================================================
// Game metadata persistence
// ============================================================

pub async fn save_game_info(pool: &PgPool, info: &GameInfo) -> Result<(), DbError> {
    let settings_json = serde_json::to_value(&info.settings)
        .map_err(|e| DbError::SerializationError(e.to_string()))?;
    let status_str = info.status.to_string();

    sqlx::query(
        r#"INSERT INTO games (id, name, seed, status, settings, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           ON CONFLICT (id) DO UPDATE SET
             name = EXCLUDED.name,
             status = EXCLUDED.status,
             settings = EXCLUDED.settings,
             updated_at = EXCLUDED.updated_at"#,
    )
    .bind(info.id)
    .bind(&info.name)
    .bind(info.settings.seed as i64)
    .bind(&status_str)
    .bind(&settings_json)
    .bind(info.created_at)
    .bind(info.updated_at)
    .execute(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Failed to save game: {}", e)))?;
    Ok(())
}

pub async fn delete_game_row(pool: &PgPool, game_id: Uuid) -> Result<(), DbError> {
    // We don't actually delete — just update status (matches in-memory behavior)
    sqlx::query("UPDATE games SET status = 'completed', updated_at = NOW() WHERE id = $1")
        .bind(game_id)
        .execute(pool)
        .await
        .map_err(|e| DbError::Internal(format!("Failed to delete game: {}", e)))?;
    Ok(())
}

// ============================================================
// Game state snapshots (JSONB in game_worlds)
// ============================================================

pub async fn save_game_state(
    pool: &PgPool,
    game_id: Uuid,
    turn: i16,
    state: &GameState,
) -> Result<(), DbError> {
    let state_json = serde_json::to_value(state)
        .map_err(|e| DbError::SerializationError(e.to_string()))?;

    sqlx::query(
        r#"INSERT INTO game_worlds (game_id, turn, data, created_at)
           VALUES ($1, $2, $3, NOW())
           ON CONFLICT (game_id, turn) DO UPDATE SET
             data = EXCLUDED.data,
             created_at = NOW()"#,
    )
    .bind(game_id)
    .bind(turn as i32)
    .bind(&state_json)
    .execute(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Failed to save game state: {}", e)))?;
    Ok(())
}

pub async fn load_latest_game_state(
    pool: &PgPool,
    game_id: Uuid,
) -> Result<Option<(i16, GameState)>, DbError> {
    let row = sqlx::query_as::<_, GameWorldRow>(
        "SELECT turn, data FROM game_worlds WHERE game_id = $1 ORDER BY turn DESC LIMIT 1"
    )
    .bind(game_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Failed to load game state: {}", e)))?;

    match row {
        Some(r) => {
            let state: GameState = serde_json::from_value(r.data)
                .map_err(|e| DbError::SerializationError(format!("Failed to deserialize GameState: {}", e)))?;
            Ok(Some((r.turn as i16, state)))
        }
        None => Ok(None),
    }
}

#[derive(sqlx::FromRow)]
struct GameWorldRow {
    turn: i32,
    data: serde_json::Value,
}

// ============================================================
// Player persistence
// ============================================================

pub async fn save_player(pool: &PgPool, player: &Player) -> Result<(), DbError> {
    sqlx::query(
        r#"INSERT INTO game_players (game_id, user_id, nation_id, joined_at, is_done_this_turn)
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (game_id, user_id) DO UPDATE SET
             nation_id = EXCLUDED.nation_id,
             is_done_this_turn = EXCLUDED.is_done_this_turn"#,
    )
    .bind(player.game_id)
    .bind(player.user_id)
    .bind(player.nation_id as i32)
    .bind(player.joined_at)
    .bind(player.is_done_this_turn)
    .execute(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Failed to save player: {}", e)))?;
    Ok(())
}

pub async fn update_player_done(
    pool: &PgPool,
    game_id: Uuid,
    user_id: Uuid,
    done: bool,
) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE game_players SET is_done_this_turn = $1 WHERE game_id = $2 AND user_id = $3"
    )
    .bind(done)
    .bind(game_id)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Failed to update player done: {}", e)))?;
    Ok(())
}

pub async fn reset_players_done(pool: &PgPool, game_id: Uuid) -> Result<(), DbError> {
    sqlx::query("UPDATE game_players SET is_done_this_turn = false WHERE game_id = $1")
        .bind(game_id)
        .execute(pool)
        .await
        .map_err(|e| DbError::Internal(format!("Failed to reset players done: {}", e)))?;
    Ok(())
}

pub async fn delete_player(pool: &PgPool, game_id: Uuid, nation_id: u8) -> Result<(), DbError> {
    sqlx::query("DELETE FROM game_players WHERE game_id = $1 AND nation_id = $2")
        .bind(game_id)
        .bind(nation_id as i32)
        .execute(pool)
        .await
        .map_err(|e| DbError::Internal(format!("Failed to delete player: {}", e)))?;
    Ok(())
}

pub async fn load_players(pool: &PgPool, game_id: Uuid) -> Result<Vec<Player>, DbError> {
    let rows = sqlx::query_as::<_, PlayerRow>(
        "SELECT game_id, user_id, nation_id, joined_at, is_done_this_turn FROM game_players WHERE game_id = $1"
    )
    .bind(game_id)
    .fetch_all(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Failed to load players: {}", e)))?;

    Ok(rows.into_iter().map(|r| Player {
        game_id: r.game_id,
        user_id: r.user_id,
        nation_id: r.nation_id as u8,
        joined_at: r.joined_at,
        is_done_this_turn: r.is_done_this_turn,
    }).collect())
}

#[derive(sqlx::FromRow)]
struct PlayerRow {
    game_id: Uuid,
    user_id: Uuid,
    nation_id: i32,
    joined_at: DateTime<Utc>,
    is_done_this_turn: bool,
}

// ============================================================
// Action persistence
// ============================================================

pub async fn save_action(pool: &PgPool, action: &SubmittedAction) -> Result<(), DbError> {
    let action_json = serde_json::to_value(&action.action)
        .map_err(|e| DbError::SerializationError(e.to_string()))?;

    sqlx::query(
        r#"INSERT INTO game_actions (id, game_id, nation_id, turn, action, submitted_at, action_order)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(action.id)
    .bind(action.game_id)
    .bind(action.nation_id as i32)
    .bind(action.turn as i32)
    .bind(&action_json)
    .bind(action.submitted_at)
    .bind(action.order as i32)
    .execute(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Failed to save action: {}", e)))?;
    Ok(())
}

pub async fn delete_action(pool: &PgPool, action_id: Uuid) -> Result<(), DbError> {
    sqlx::query("DELETE FROM game_actions WHERE id = $1")
        .bind(action_id)
        .execute(pool)
        .await
        .map_err(|e| DbError::Internal(format!("Failed to delete action: {}", e)))?;
    Ok(())
}

pub async fn load_actions(pool: &PgPool, game_id: Uuid) -> Result<Vec<SubmittedAction>, DbError> {
    let rows = sqlx::query_as::<_, ActionRow>(
        "SELECT id, game_id, nation_id, turn, action, submitted_at, action_order FROM game_actions WHERE game_id = $1 ORDER BY turn, nation_id, action_order"
    )
    .bind(game_id)
    .fetch_all(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Failed to load actions: {}", e)))?;

    let mut actions = Vec::new();
    for r in rows {
        let action: conquer_core::actions::Action = serde_json::from_value(r.action)
            .map_err(|e| DbError::SerializationError(format!("Failed to deserialize action: {}", e)))?;
        actions.push(SubmittedAction {
            id: r.id,
            game_id: r.game_id,
            nation_id: r.nation_id as u8,
            turn: r.turn as i16,
            action,
            submitted_at: r.submitted_at,
            order: r.action_order as u32,
        });
    }
    Ok(actions)
}

#[derive(sqlx::FromRow)]
struct ActionRow {
    id: Uuid,
    game_id: Uuid,
    nation_id: i32,
    turn: i32,
    action: serde_json::Value,
    submitted_at: DateTime<Utc>,
    action_order: i32,
}

// ============================================================
// Chat persistence
// ============================================================

pub async fn save_chat_message(pool: &PgPool, msg: &ChatMessage) -> Result<(), DbError> {
    sqlx::query(
        r#"INSERT INTO chat_messages (id, game_id, sender_nation_id, channel, content, created_at)
           VALUES ($1, $2, $3, $4, $5, $6)
           ON CONFLICT (id) DO NOTHING"#,
    )
    .bind(msg.id)
    .bind(msg.game_id)
    .bind(msg.sender_nation_id.map(|n| n as i32))
    .bind(&msg.channel)
    .bind(&msg.content)
    .bind(msg.created_at)
    .execute(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Failed to save chat message: {}", e)))?;
    Ok(())
}

pub async fn load_chat_messages(pool: &PgPool, game_id: Uuid) -> Result<Vec<ChatMessage>, DbError> {
    let rows = sqlx::query_as::<_, ChatRow>(
        "SELECT id, game_id, sender_nation_id, channel, content, created_at FROM chat_messages WHERE game_id = $1 ORDER BY created_at ASC LIMIT 1000"
    )
    .bind(game_id)
    .fetch_all(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Failed to load chat: {}", e)))?;

    Ok(rows.into_iter().map(|r| {
        let sender_nation_id = r.sender_nation_id.map(|n| n as u8);
        let is_system = sender_nation_id.is_none();
        let sender_name = if is_system {
            "SYSTEM".to_string()
        } else {
            format!("Nation {}", sender_nation_id.unwrap())
        };
        ChatMessage {
            id: r.id,
            game_id: r.game_id,
            sender_nation_id,
            sender_name,
            channel: r.channel,
            content: r.content,
            created_at: r.created_at,
            is_system,
        }
    }).collect())
}

#[derive(sqlx::FromRow)]
struct ChatRow {
    id: Uuid,
    game_id: Uuid,
    sender_nation_id: Option<i32>,
    channel: String,
    content: String,
    created_at: DateTime<Utc>,
}

// ============================================================
// Invite persistence
// ============================================================

pub async fn save_invite(pool: &PgPool, invite: &GameInvite) -> Result<(), DbError> {
    sqlx::query(
        r#"INSERT INTO game_invites (id, game_id, invite_code, created_by, expires_at, max_uses, uses)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           ON CONFLICT (id) DO UPDATE SET
             uses = EXCLUDED.uses"#,
    )
    .bind(invite.id)
    .bind(invite.game_id)
    .bind(&invite.invite_code)
    .bind(invite.created_by)
    .bind(invite.expires_at)
    .bind(invite.max_uses.map(|u| u as i32))
    .bind(invite.uses as i32)
    .execute(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Failed to save invite: {}", e)))?;
    Ok(())
}

pub async fn delete_invite(pool: &PgPool, invite_id: Uuid) -> Result<(), DbError> {
    sqlx::query("DELETE FROM game_invites WHERE id = $1")
        .bind(invite_id)
        .execute(pool)
        .await
        .map_err(|e| DbError::Internal(format!("Failed to delete invite: {}", e)))?;
    Ok(())
}

pub async fn load_invites(pool: &PgPool, game_id: Uuid) -> Result<Vec<GameInvite>, DbError> {
    let rows = sqlx::query_as::<_, InviteRow>(
        "SELECT id, game_id, invite_code, created_by, expires_at, max_uses, uses FROM game_invites WHERE game_id = $1"
    )
    .bind(game_id)
    .fetch_all(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Failed to load invites: {}", e)))?;

    Ok(rows.into_iter().map(|r| GameInvite {
        id: r.id,
        game_id: r.game_id,
        invite_code: r.invite_code,
        created_by: r.created_by,
        expires_at: r.expires_at,
        max_uses: r.max_uses.map(|u| u as u32),
        uses: r.uses as u32,
    }).collect())
}

#[derive(sqlx::FromRow)]
struct InviteRow {
    id: Uuid,
    game_id: Uuid,
    invite_code: String,
    created_by: Uuid,
    expires_at: Option<DateTime<Utc>>,
    max_uses: Option<i32>,
    uses: i32,
}

// ============================================================
// Full hydration: load all data from Postgres into memory
// ============================================================

#[derive(Debug, Clone)]
struct GameMetaRow {
    id: Uuid,
    name: String,
    seed: i64,
    status: String,
    settings: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

pub async fn load_all_games(pool: &PgPool) -> Result<Vec<ManagedGame>, DbError> {
    // Load game metadata
    let meta_rows = sqlx::query_as::<_, GameMetaRowSqlx>(
        "SELECT id, name, seed, status, settings, created_at, updated_at FROM games"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| DbError::Internal(format!("Failed to load games: {}", e)))?;

    let mut games = Vec::new();

    for meta in meta_rows {
        let status = match meta.status.as_str() {
            "waiting_for_players" => GameStatus::WaitingForPlayers,
            "active" => GameStatus::Active,
            "paused" => GameStatus::Paused,
            "completed" => GameStatus::Completed,
            s => {
                tracing::warn!("Unknown game status '{}', defaulting to completed", s);
                GameStatus::Completed
            }
        };

        let settings: GameSettings = serde_json::from_value(meta.settings.clone())
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to deserialize settings for game {}: {}", meta.id, e);
                GameSettings::default()
            });

        // Load latest game state snapshot
        let state_opt = load_latest_game_state(pool, meta.id).await?;
        let (current_turn, state) = match state_opt {
            Some((turn, state)) => (turn, state),
            None => {
                tracing::warn!("No state snapshot for game {}, skipping", meta.id);
                continue;
            }
        };

        // Load players
        let players = load_players(pool, meta.id).await?;

        // Load actions
        let actions = load_actions(pool, meta.id).await?;

        // Load chat
        let chat_messages = load_chat_messages(pool, meta.id).await?;

        // Load invites
        let invites = load_invites(pool, meta.id).await?;

        // Fix up chat sender names from loaded state
        let mut fixed_chat: Vec<ChatMessage> = chat_messages.into_iter().map(|mut msg| {
            if let Some(nid) = msg.sender_nation_id {
                if (nid as usize) < state.nations.len() {
                    let n = &state.nations[nid as usize];
                    if !n.name.is_empty() {
                        msg.sender_name = format!("{} ({})", n.name, n.leader);
                    }
                }
            }
            msg
        }).collect();

        let rng = conquer_engine::rng::ConquerRng::new(settings.seed as u32);

        let info = GameInfo {
            id: meta.id,
            name: meta.name,
            status,
            settings,
            created_at: meta.created_at,
            updated_at: meta.updated_at,
            current_turn,
            player_count: players.len(),
        };

        let managed = ManagedGame {
            info,
            state,
            rng,
            players,
            actions,
            news: Vec::new(), // News not persisted (ephemeral)
            chat_messages: fixed_chat,
            invites,
            spectators: Vec::new(),
            turn_snapshots: Vec::new(), // Snapshots live in game_worlds table
        };

        games.push(managed);
    }

    Ok(games)
}

#[derive(sqlx::FromRow)]
struct GameMetaRowSqlx {
    id: Uuid,
    name: String,
    seed: i64,
    status: String,
    settings: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
