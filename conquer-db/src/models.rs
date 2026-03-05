use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// Game Status
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameStatus {
    WaitingForPlayers,
    Active,
    Paused,
    Completed,
}

impl std::fmt::Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameStatus::WaitingForPlayers => write!(f, "waiting_for_players"),
            GameStatus::Active => write!(f, "active"),
            GameStatus::Paused => write!(f, "paused"),
            GameStatus::Completed => write!(f, "completed"),
        }
    }
}

// ============================================================
// Game Settings
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub map_x: usize,
    pub map_y: usize,
    pub max_players: usize,
    pub npc_count: usize,
    pub monster_count: usize,
    pub seed: u64,
    pub turn_timer_hours: Option<f64>,
    pub auto_advance: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            map_x: 32,
            map_y: 32,
            max_players: 10,
            npc_count: 10,
            monster_count: 5,
            seed: 42,
            turn_timer_hours: None,
            auto_advance: false,
        }
    }
}

// ============================================================
// Game Info (metadata, not full state)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub id: Uuid,
    pub name: String,
    pub status: GameStatus,
    pub settings: GameSettings,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub current_turn: i16,
    pub player_count: usize,
}

// ============================================================
// User
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub is_admin: bool,
}

// ============================================================
// Player (user ↔ game mapping)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub game_id: Uuid,
    pub user_id: Uuid,
    pub nation_id: u8,
    pub joined_at: DateTime<Utc>,
    pub is_done_this_turn: bool,
}

// ============================================================
// Chat Message
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: Uuid,
    pub game_id: Uuid,
    pub sender_nation_id: Option<u8>,
    /// Sender display: "NationName (LeaderName)" or "SYSTEM"
    pub sender_name: String,
    pub channel: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    /// true for system-generated messages (turn advance, diplomacy, etc.)
    #[serde(default)]
    pub is_system: bool,
}

// ============================================================
// Game Invite
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInvite {
    pub id: Uuid,
    pub game_id: Uuid,
    pub invite_code: String,
    pub created_by: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
    pub max_uses: Option<u32>,
    pub uses: u32,
}

// ============================================================
// Submitted Action (action stored for a turn)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmittedAction {
    pub id: Uuid,
    pub game_id: Uuid,
    pub nation_id: u8,
    pub turn: i16,
    pub action: conquer_core::actions::Action,
    pub submitted_at: DateTime<Utc>,
    pub order: u32,
}

// ============================================================
// News entry
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsEntry {
    pub turn: i16,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}
