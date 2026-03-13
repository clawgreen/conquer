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
    // Phase 6 additions (T413-T418)
    #[serde(default = "default_mountain_pct")]
    pub mountain_pct: u8,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default = "default_min_players")]
    pub min_players: usize,
    #[serde(default)]
    pub npc_cheat: bool,
    #[serde(default)]
    pub npc_see_cities: bool,
    #[serde(default = "default_true")]
    pub monster_respawn: bool,
    #[serde(default = "default_true")]
    pub npc_messages: bool,
    #[serde(default = "default_true")]
    pub trade_enabled: bool,
    #[serde(default = "default_true")]
    pub random_events: bool,
    #[serde(default = "default_true")]
    pub storms_enabled: bool,
    #[serde(default)]
    pub starting_gold: Option<i64>,
    /// Game creator user_id (admin of the game)
    #[serde(default)]
    pub creator_id: Option<Uuid>,
    /// Whether the game is publicly listed in the browser
    #[serde(default = "default_true")]
    pub public_game: bool,
}

fn default_mountain_pct() -> u8 {
    25
}
fn default_min_players() -> usize {
    2
}
fn default_true() -> bool {
    true
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
            mountain_pct: 25,
            password: None,
            min_players: 2,
            npc_cheat: false,
            npc_see_cities: false,
            monster_respawn: true,
            npc_messages: true,
            trade_enabled: true,
            random_events: true,
            storms_enabled: true,
            starting_gold: None,
            creator_id: None,
            public_game: true,
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

// ============================================================
// User Profile (T409-T411)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub games_played: u32,
    pub games_won: u32,
    pub games_lost: u32,
    pub game_history: Vec<GameHistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameHistoryEntry {
    pub game_id: Uuid,
    pub game_name: String,
    pub nation_name: String,
    pub race: char,
    pub class: i16,
    pub final_score: i64,
    pub outcome: String, // "active", "won", "lost", "left"
    pub joined_at: DateTime<Utc>,
}

// ============================================================
// Notification (T432-T434)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_type: NotificationType,
    pub game_id: Option<Uuid>,
    pub message: String,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    YourTurn,
    GameStarted,
    GameInvite,
    UnderAttack,
    TurnAdvanced,
    PlayerJoined,
    GameCompleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub your_turn: bool,
    pub game_started: bool,
    pub game_invite: bool,
    pub under_attack: bool,
    pub turn_advanced: bool,
    pub player_joined: bool,
    pub game_completed: bool,
    pub email_enabled: bool,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        NotificationPreferences {
            your_turn: true,
            game_started: true,
            game_invite: true,
            under_attack: true,
            turn_advanced: true,
            player_joined: false,
            game_completed: true,
            email_enabled: false,
        }
    }
}

// ============================================================
// Spectator (T428-T431)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spectator {
    pub game_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

// ============================================================
// Turn Snapshot for rollback (T426)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnSnapshot {
    pub game_id: Uuid,
    pub turn: i16,
    pub state_json: String,
    pub created_at: DateTime<Utc>,
}
