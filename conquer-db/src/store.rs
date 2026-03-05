// conquer-db/src/store.rs — In-memory game store
//
// Thread-safe storage for games, users, players, actions, chat, invites.
// Uses Arc<RwLock<>> for concurrent access from Axum handlers.
// This is the primary store for testing; Postgres can replace it later.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use conquer_core::*;
use conquer_core::actions::Action;
use conquer_engine::worldgen;
use conquer_engine::rng::ConquerRng;

use crate::auth::AuthManager;
use crate::error::DbError;
use crate::models::*;

// ============================================================
// Per-game state container
// ============================================================

#[derive(Debug, Clone)]
pub struct ManagedGame {
    pub info: GameInfo,
    pub state: GameState,
    pub rng: ConquerRng,
    pub players: Vec<Player>,
    pub actions: Vec<SubmittedAction>,
    pub news: Vec<NewsEntry>,
    pub chat_messages: Vec<ChatMessage>,
    pub invites: Vec<GameInvite>,
    /// Spectators watching the game (T428)
    pub spectators: Vec<Spectator>,
    /// Turn snapshots for rollback (T426)
    pub turn_snapshots: Vec<TurnSnapshot>,
}

// ============================================================
// GameStore — the central in-memory store
// ============================================================

#[derive(Clone)]
pub struct GameStore {
    games: Arc<RwLock<HashMap<Uuid, ManagedGame>>>,
    users: Arc<RwLock<HashMap<Uuid, User>>>,
    /// username -> user_id index
    username_index: Arc<RwLock<HashMap<String, Uuid>>>,
    /// email -> user_id index
    email_index: Arc<RwLock<HashMap<String, Uuid>>>,
    /// invite_code -> game_id index
    invite_index: Arc<RwLock<HashMap<String, Uuid>>>,
    /// Per-user notifications (T432)
    notifications: Arc<RwLock<HashMap<Uuid, Vec<Notification>>>>,
    /// Per-user notification preferences (T434)
    notification_prefs: Arc<RwLock<HashMap<Uuid, NotificationPreferences>>>,
}

impl GameStore {
    pub fn new() -> Self {
        GameStore {
            games: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
            username_index: Arc::new(RwLock::new(HashMap::new())),
            email_index: Arc::new(RwLock::new(HashMap::new())),
            invite_index: Arc::new(RwLock::new(HashMap::new())),
            notifications: Arc::new(RwLock::new(HashMap::new())),
            notification_prefs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ========================================================
    // User operations
    // ========================================================

    /// Register a new user
    pub async fn create_user(
        &self,
        username: &str,
        email: &str,
        password: &str,
        display_name: Option<&str>,
    ) -> Result<User, DbError> {
        let username_lower = username.to_lowercase();
        let email_lower = email.to_lowercase();

        // Check uniqueness
        {
            let idx = self.username_index.read().await;
            if idx.contains_key(&username_lower) {
                return Err(DbError::AlreadyExists(format!("Username '{}' taken", username)));
            }
        }
        {
            let idx = self.email_index.read().await;
            if idx.contains_key(&email_lower) {
                return Err(DbError::AlreadyExists(format!("Email '{}' already registered", email)));
            }
        }

        let password_hash = AuthManager::hash_password(password)?;
        let user = User {
            id: Uuid::new_v4(),
            username: username_lower.clone(),
            email: email_lower.clone(),
            password_hash,
            display_name: display_name.unwrap_or(username).to_string(),
            created_at: Utc::now(),
            is_admin: false,
        };

        let id = user.id;
        self.users.write().await.insert(id, user.clone());
        self.username_index.write().await.insert(username_lower, id);
        self.email_index.write().await.insert(email_lower, id);

        Ok(user)
    }

    /// Authenticate user by username + password, return user
    pub async fn authenticate_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<User, DbError> {
        let username_lower = username.to_lowercase();
        let user_id = {
            let idx = self.username_index.read().await;
            idx.get(&username_lower)
                .copied()
                .ok_or_else(|| DbError::AuthError("Invalid credentials".to_string()))?
        };

        let users = self.users.read().await;
        let user = users
            .get(&user_id)
            .ok_or_else(|| DbError::AuthError("Invalid credentials".to_string()))?;

        if !AuthManager::verify_password(password, &user.password_hash)? {
            return Err(DbError::AuthError("Invalid credentials".to_string()));
        }

        Ok(user.clone())
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: Uuid) -> Result<User, DbError> {
        let users = self.users.read().await;
        users
            .get(&user_id)
            .cloned()
            .ok_or_else(|| DbError::NotFound(format!("User {}", user_id)))
    }

    /// Check if user is admin
    pub async fn is_admin(&self, user_id: Uuid) -> bool {
        let users = self.users.read().await;
        users.get(&user_id).map(|u| u.is_admin).unwrap_or(false)
    }

    // ========================================================
    // Game operations
    // ========================================================

    /// Create a new game with worldgen
    pub async fn create_game(
        &self,
        name: &str,
        settings: GameSettings,
    ) -> Result<GameInfo, DbError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        // Generate world
        let mut rng = ConquerRng::new(settings.seed as u32);
        let mut state = GameState::new(settings.map_x, settings.map_y);
        // Default 25% water coverage
        worldgen::create_world(&mut state, &mut rng, 25);

        let info = GameInfo {
            id,
            name: name.to_string(),
            status: GameStatus::WaitingForPlayers,
            settings: settings.clone(),
            created_at: now,
            updated_at: now,
            current_turn: state.world.turn,
            player_count: 0,
        };

        let managed = ManagedGame {
            info: info.clone(),
            state,
            rng,
            players: Vec::new(),
            actions: Vec::new(),
            news: Vec::new(),
            chat_messages: Vec::new(),
            invites: Vec::new(),
            spectators: Vec::new(),
            turn_snapshots: Vec::new(),
        };

        self.games.write().await.insert(id, managed);
        Ok(info)
    }

    /// List games, optionally filtered by status
    pub async fn list_games(&self, status_filter: Option<GameStatus>) -> Vec<GameInfo> {
        let games = self.games.read().await;
        games
            .values()
            .filter(|g| status_filter.map_or(true, |s| g.info.status == s))
            .map(|g| g.info.clone())
            .collect()
    }

    /// Get game info by ID
    pub async fn get_game_info(&self, game_id: Uuid) -> Result<GameInfo, DbError> {
        let games = self.games.read().await;
        games
            .get(&game_id)
            .map(|g| g.info.clone())
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))
    }

    /// Get full game state (for internal use / turn processing)
    pub async fn get_game_state(&self, game_id: Uuid) -> Result<GameState, DbError> {
        let games = self.games.read().await;
        games
            .get(&game_id)
            .map(|g| g.state.clone())
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))
    }

    /// Delete (archive) a game
    pub async fn delete_game(&self, game_id: Uuid) -> Result<(), DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        game.info.status = GameStatus::Completed;
        game.info.updated_at = Utc::now();
        Ok(())
    }

    // ========================================================
    // Player operations
    // ========================================================

    /// Join a game as a new nation.
    /// Returns the nation_id assigned.
    pub async fn join_game(
        &self,
        game_id: Uuid,
        user_id: Uuid,
        nation_name: &str,
        leader_name: &str,
        race: char,
        class: i16,
        mark: char,
    ) -> Result<Player, DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        // Check if user already in this game
        if game.players.iter().any(|p| p.user_id == user_id) {
            return Err(DbError::AlreadyExists("Already joined this game".to_string()));
        }

        // Find first available nation slot (skip 0 = God)
        let nation_id = (1..NTOTAL as u8)
            .find(|&i| {
                let n = &game.state.nations[i as usize];
                !n.is_active() && !game.players.iter().any(|p| p.nation_id == i)
            })
            .ok_or(DbError::GameFull)?;

        // Initialize nation in game state
        let nation = &mut game.state.nations[nation_id as usize];
        nation.name = nation_name.to_string();
        nation.leader = leader_name.to_string();
        nation.race = race;
        nation.class = class;
        nation.mark = mark;
        nation.active = 1; // PC_GOOD default
        nation.tax_rate = 15;
        nation.eat_rate = 6;
        nation.popularity = 50;
        nation.communications = 50;
        nation.wealth = 50;
        nation.knowledge = 50;
        nation.farm_ability = 50;
        nation.mine_ability = 50;
        nation.reputation = 50;
        nation.spoil_rate = 50;

        let player = Player {
            game_id,
            user_id,
            nation_id,
            joined_at: Utc::now(),
            is_done_this_turn: false,
        };
        game.players.push(player.clone());
        game.info.player_count = game.players.len();
        game.info.updated_at = Utc::now();

        // System message: nation joined (T395)
        let race_name = match race {
            'H' => "Human", 'E' => "Elf", 'D' => "Dwarf", 'O' => "Orc",
            'L' => "Lizard", 'P' => "Pirate", 'S' => "Savage", 'N' => "Nomad",
            _ => "Unknown",
        };
        let class_name = match class {
            0 => "Monster", 1 => "King", 2 => "Emperor", 3 => "Wizard",
            4 => "Priest", 5 => "Pirate", 6 => "Trader", 7 => "Warlord",
            _ => "Adventurer",
        };
        let sys_msg = ChatMessage {
            id: Uuid::new_v4(),
            game_id,
            sender_nation_id: None,
            sender_name: "SYSTEM".to_string(),
            channel: "public".to_string(),
            content: format!("⚔ The nation of {} ({} {}) has entered the world!", nation_name, race_name, class_name),
            created_at: Utc::now(),
            is_system: true,
        };
        game.chat_messages.push(sys_msg);

        Ok(player)
    }

    /// Get player info for a user in a game
    pub async fn get_player(
        &self,
        game_id: Uuid,
        user_id: Uuid,
    ) -> Result<Player, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        game.players
            .iter()
            .find(|p| p.user_id == user_id)
            .cloned()
            .ok_or_else(|| DbError::NotFound("Player not in game".to_string()))
    }

    /// List players in a game
    pub async fn list_players(&self, game_id: Uuid) -> Result<Vec<Player>, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        Ok(game.players.clone())
    }

    /// Mark player as done for this turn
    pub async fn set_player_done(
        &self,
        game_id: Uuid,
        user_id: Uuid,
        done: bool,
    ) -> Result<(), DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        let player = game.players
            .iter_mut()
            .find(|p| p.user_id == user_id)
            .ok_or_else(|| DbError::NotFound("Player not in game".to_string()))?;
        player.is_done_this_turn = done;
        Ok(())
    }

    /// Check if all players are done
    pub async fn all_players_done(&self, game_id: Uuid) -> Result<bool, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        Ok(!game.players.is_empty() && game.players.iter().all(|p| p.is_done_this_turn))
    }

    // ========================================================
    // Action operations
    // ========================================================

    /// Submit an action for the current turn
    pub async fn submit_action(
        &self,
        game_id: Uuid,
        nation_id: u8,
        action: Action,
    ) -> Result<SubmittedAction, DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        let turn = game.state.world.turn;
        let order = game.actions.iter()
            .filter(|a| a.turn == turn && a.nation_id == nation_id)
            .count() as u32;

        let submitted = SubmittedAction {
            id: Uuid::new_v4(),
            game_id,
            nation_id,
            turn,
            action,
            submitted_at: Utc::now(),
            order,
        };

        game.actions.push(submitted.clone());
        Ok(submitted)
    }

    /// Get actions for a nation in the current turn
    pub async fn get_actions(
        &self,
        game_id: Uuid,
        nation_id: u8,
    ) -> Result<Vec<SubmittedAction>, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        let turn = game.state.world.turn;
        Ok(game.actions.iter()
            .filter(|a| a.turn == turn && a.nation_id == nation_id)
            .cloned()
            .collect())
    }

    /// Retract an action
    pub async fn retract_action(
        &self,
        game_id: Uuid,
        action_id: Uuid,
        nation_id: u8,
    ) -> Result<(), DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        let turn = game.state.world.turn;
        let idx = game.actions.iter().position(|a| {
            a.id == action_id && a.nation_id == nation_id && a.turn == turn
        });
        match idx {
            Some(i) => { game.actions.remove(i); Ok(()) }
            None => Err(DbError::NotFound(format!("Action {}", action_id))),
        }
    }

    // ========================================================
    // Turn processing
    // ========================================================

    /// Run a turn advance. Applies all actions, runs the update pipeline.
    /// Returns the new turn number.
    pub async fn run_turn(&self, game_id: Uuid) -> Result<i16, DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        if game.info.status != GameStatus::Active && game.info.status != GameStatus::WaitingForPlayers {
            return Err(DbError::InvalidState(format!(
                "Cannot advance turn: game is {}", game.info.status
            )));
        }

        let current_turn = game.state.world.turn;

        // Save snapshot before advancing (T426 — rollback support)
        if let Ok(state_json) = serde_json::to_string(&game.state) {
            let snapshot = TurnSnapshot {
                game_id,
                turn: current_turn,
                state_json,
                created_at: Utc::now(),
            };
            game.turn_snapshots.push(snapshot);
            if game.turn_snapshots.len() > 10 {
                game.turn_snapshots.drain(..game.turn_snapshots.len() - 10);
            }
        }

        // Collect actions for this turn, sorted by nation then order
        let mut turn_actions: Vec<_> = game.actions.iter()
            .filter(|a| a.turn == current_turn)
            .cloned()
            .collect();
        turn_actions.sort_by_key(|a| (a.nation_id, a.order));

        // Apply actions to game state via the engine's execute system
        for sa in &turn_actions {
            apply_action_to_state(&mut game.state, &sa.action);
        }

        // Run the turn update pipeline
        // For now, just advance the turn counter and do basic updates
        game.state.world.turn += 1;
        let new_turn = game.state.world.turn;

        // Reset player done flags
        for player in &mut game.players {
            player.is_done_this_turn = false;
        }

        // Update game info
        game.info.current_turn = new_turn;
        game.info.updated_at = Utc::now();

        // If game was waiting, move to active
        if game.info.status == GameStatus::WaitingForPlayers {
            game.info.status = GameStatus::Active;
        }

        // Add news entry
        game.news.push(NewsEntry {
            turn: new_turn,
            message: format!("Turn {} has begun", new_turn),
            timestamp: Utc::now(),
        });

        // System message: turn advance (T394)
        let season = ["Winter", "Spring", "Summer", "Fall"][(new_turn % 4) as usize];
        let year = (new_turn as i32 + 3) / 4;
        let sys_msg = ChatMessage {
            id: Uuid::new_v4(),
            game_id,
            sender_nation_id: None,
            sender_name: "SYSTEM".to_string(),
            channel: "public".to_string(),
            content: format!("━━━ Turn {} ({}, Year {}) has begun ━━━", new_turn, season, year),
            created_at: Utc::now(),
            is_system: true,
        };
        game.chat_messages.push(sys_msg);

        Ok(new_turn)
    }

    // ========================================================
    // News operations
    // ========================================================

    /// Get news for a game
    pub async fn get_news(
        &self,
        game_id: Uuid,
        turn: Option<i16>,
    ) -> Result<Vec<NewsEntry>, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        Ok(match turn {
            Some(t) => game.news.iter().filter(|n| n.turn == t).cloned().collect(),
            None => game.news.clone(),
        })
    }

    // ========================================================
    // Chat operations (T388-T399)
    // ========================================================

    /// Send a chat message with rate limiting (T393)
    /// Returns Err if rate limited (max 5 messages per 10 seconds per nation)
    pub async fn send_chat(
        &self,
        game_id: Uuid,
        sender_nation_id: Option<u8>,
        channel: &str,
        content: &str,
    ) -> Result<ChatMessage, DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        // Rate limiting: max 5 messages in 10 seconds per nation (T393)
        if let Some(nid) = sender_nation_id {
            let now = Utc::now();
            let window = chrono::Duration::seconds(10);
            let recent_count = game.chat_messages.iter()
                .filter(|m| m.sender_nation_id == Some(nid) && !m.is_system)
                .filter(|m| now.signed_duration_since(m.created_at) < window)
                .count();
            if recent_count >= 5 {
                return Err(DbError::InvalidState("Rate limited: too many messages".to_string()));
            }
        }

        // Build sender name
        let sender_name = match sender_nation_id {
            Some(nid) if (nid as usize) < game.state.nations.len() => {
                let n = &game.state.nations[nid as usize];
                if n.name.is_empty() {
                    format!("Nation {}", nid)
                } else {
                    format!("{} ({})", n.name, n.leader)
                }
            }
            Some(nid) => format!("Nation {}", nid),
            None => "SYSTEM".to_string(),
        };

        let msg = ChatMessage {
            id: Uuid::new_v4(),
            game_id,
            sender_nation_id,
            sender_name,
            channel: channel.to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
            is_system: sender_nation_id.is_none(),
        };
        game.chat_messages.push(msg.clone());

        // Ring buffer: keep max 1000 messages per game (configurable history)
        if game.chat_messages.len() > 1000 {
            game.chat_messages.drain(..game.chat_messages.len() - 1000);
        }

        Ok(msg)
    }

    /// Send a system message (no rate limiting) (T394-T399)
    pub async fn send_system_message(
        &self,
        game_id: Uuid,
        channel: &str,
        content: &str,
    ) -> Result<ChatMessage, DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        let msg = ChatMessage {
            id: Uuid::new_v4(),
            game_id,
            sender_nation_id: None,
            sender_name: "SYSTEM".to_string(),
            channel: channel.to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
            is_system: true,
        };
        game.chat_messages.push(msg.clone());

        // Ring buffer
        if game.chat_messages.len() > 1000 {
            game.chat_messages.drain(..game.chat_messages.len() - 1000);
        }

        Ok(msg)
    }

    /// Get chat messages with pagination (T392)
    pub async fn get_chat(
        &self,
        game_id: Uuid,
        channel: &str,
        limit: usize,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<ChatMessage>, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        let mut msgs: Vec<_> = game.chat_messages.iter()
            .filter(|m| m.channel == channel)
            .filter(|m| before.map_or(true, |b| m.created_at < b))
            .cloned()
            .collect();
        msgs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        msgs.truncate(limit);
        Ok(msgs)
    }

    /// Get chat messages visible to a specific nation (T389-T390)
    /// Public channel: visible to all. Private channels (nation_X_Y): visible to X and Y only.
    pub async fn get_chat_for_nation(
        &self,
        game_id: Uuid,
        nation_id: u8,
        channel: &str,
        limit: usize,
        before: Option<DateTime<Utc>>,
    ) -> Result<Vec<ChatMessage>, DbError> {
        // Validate the nation can see this channel
        if channel != "public" && !Self::nation_can_see_channel(nation_id, channel) {
            return Err(DbError::Unauthorized("Cannot access this channel".to_string()));
        }
        self.get_chat(game_id, channel, limit, before).await
    }

    /// Check if a nation can see a private channel (public for WS handler)
    /// Private channels are named "nation_X_Y" where X < Y
    pub fn nation_can_see_channel_pub(nation_id: u8, channel: &str) -> bool {
        Self::nation_can_see_channel(nation_id, channel)
    }

    /// Check if a nation can see a private channel
    /// Private channels are named "nation_X_Y" where X < Y
    fn nation_can_see_channel(nation_id: u8, channel: &str) -> bool {
        if channel == "public" {
            return true;
        }
        // Parse "nation_X_Y" format
        let parts: Vec<&str> = channel.split('_').collect();
        if parts.len() == 3 && parts[0] == "nation" {
            if let (Ok(a), Ok(b)) = (parts[1].parse::<u8>(), parts[2].parse::<u8>()) {
                return nation_id == a || nation_id == b;
            }
        }
        false
    }

    /// Get the canonical channel name for a private nation-to-nation channel
    /// Always orders nation IDs so smaller is first: "nation_1_3" not "nation_3_1"
    pub fn private_channel_name(nation_a: u8, nation_b: u8) -> String {
        let (lo, hi) = if nation_a < nation_b { (nation_a, nation_b) } else { (nation_b, nation_a) };
        format!("nation_{}_{}", lo, hi)
    }

    /// List available channels for a nation in a game
    pub async fn list_channels_for_nation(
        &self,
        game_id: Uuid,
        nation_id: u8,
    ) -> Result<Vec<String>, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        let mut channels = vec!["public".to_string()];
        // Find all private channels this nation participates in
        let mut seen = std::collections::HashSet::new();
        for msg in &game.chat_messages {
            if Self::nation_can_see_channel(nation_id, &msg.channel) && msg.channel != "public" {
                if seen.insert(msg.channel.clone()) {
                    channels.push(msg.channel.clone());
                }
            }
        }
        Ok(channels)
    }

    /// Get connected nation IDs for a game (for presence tracking) (T405)
    /// Note: actual presence is tracked via WebSocket connections;
    /// this returns player nation_ids for the game
    pub async fn get_player_nation_ids(
        &self,
        game_id: Uuid,
    ) -> Result<Vec<u8>, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        Ok(game.players.iter().map(|p| p.nation_id).collect())
    }

    // ========================================================
    // Invite operations
    // ========================================================

    /// Create an invite code for a game
    pub async fn create_invite(
        &self,
        game_id: Uuid,
        created_by: Uuid,
        max_uses: Option<u32>,
        expires_hours: Option<f64>,
    ) -> Result<GameInvite, DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        let code = format!("{}", Uuid::new_v4().as_simple());
        let code_short = code[..8].to_string();

        let invite = GameInvite {
            id: Uuid::new_v4(),
            game_id,
            invite_code: code_short.clone(),
            created_by,
            expires_at: expires_hours.map(|h| {
                Utc::now() + chrono::Duration::seconds((h * 3600.0) as i64)
            }),
            max_uses,
            uses: 0,
        };
        game.invites.push(invite.clone());
        self.invite_index.write().await.insert(code_short, game_id);
        Ok(invite)
    }

    /// Look up an invite by code
    pub async fn get_invite(&self, code: &str) -> Result<(GameInvite, GameInfo), DbError> {
        let game_id = {
            let idx = self.invite_index.read().await;
            idx.get(code)
                .copied()
                .ok_or_else(|| DbError::NotFound(format!("Invite code '{}'", code)))?
        };

        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        let invite = game.invites.iter()
            .find(|i| i.invite_code == code)
            .cloned()
            .ok_or_else(|| DbError::NotFound(format!("Invite code '{}'", code)))?;

        // Check expiry
        if let Some(exp) = invite.expires_at {
            if Utc::now() > exp {
                return Err(DbError::InvalidState("Invite expired".to_string()));
            }
        }

        // Check uses
        if let Some(max) = invite.max_uses {
            if invite.uses >= max {
                return Err(DbError::InvalidState("Invite max uses reached".to_string()));
            }
        }

        Ok((invite, game.info.clone()))
    }

    /// Use an invite (increment usage)
    pub async fn use_invite(&self, code: &str) -> Result<(), DbError> {
        let game_id = {
            let idx = self.invite_index.read().await;
            idx.get(code)
                .copied()
                .ok_or_else(|| DbError::NotFound(format!("Invite code '{}'", code)))?
        };

        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        let invite = game.invites.iter_mut()
            .find(|i| i.invite_code == code)
            .ok_or_else(|| DbError::NotFound(format!("Invite code '{}'", code)))?;

        invite.uses += 1;
        Ok(())
    }

    // ========================================================
    // Map / visibility helpers
    // ========================================================

    /// Get the visible map for a nation (fog of war)
    /// Returns sectors the nation can see, with hidden sectors zeroed out
    pub async fn get_visible_map(
        &self,
        game_id: Uuid,
        nation_id: u8,
    ) -> Result<Vec<Vec<Option<Sector>>>, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        let state = &game.state;
        let map_x = state.world.map_x as usize;
        let map_y = state.world.map_y as usize;

        let mut visible = vec![vec![None; map_y]; map_x];

        for x in 0..map_x {
            for y in 0..map_y {
                // A nation can see sectors it owns
                if state.sectors[x][y].owner == nation_id {
                    visible[x][y] = Some(state.sectors[x][y].clone());
                    // Also reveal neighbors
                    for dx in -2i32..=2 {
                        for dy in -2i32..=2 {
                            let nx = x as i32 + dx;
                            let ny = y as i32 + dy;
                            if state.on_map(nx, ny) {
                                let nx = nx as usize;
                                let ny = ny as usize;
                                if visible[nx][ny].is_none() {
                                    visible[nx][ny] = Some(state.sectors[nx][ny].clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Also reveal around armies
        let nation = &state.nations[nation_id as usize];
        for army in &nation.armies {
            if army.soldiers > 0 {
                let ax = army.x as i32;
                let ay = army.y as i32;
                let see_range = ARMYSEE as i32;
                for dx in -see_range..=see_range {
                    for dy in -see_range..=see_range {
                        let nx = ax + dx;
                        let ny = ay + dy;
                        if state.on_map(nx, ny) {
                            let nx = nx as usize;
                            let ny = ny as usize;
                            if visible[nx][ny].is_none() {
                                visible[nx][ny] = Some(state.sectors[nx][ny].clone());
                            }
                        }
                    }
                }
            }
        }

        // Also reveal around navies
        for navy in &nation.navies {
            if navy.has_ships() {
                let nx_pos = navy.x as i32;
                let ny_pos = navy.y as i32;
                let see_range = NAVYSEE as i32;
                for dx in -see_range..=see_range {
                    for dy in -see_range..=see_range {
                        let nx = nx_pos + dx;
                        let ny = ny_pos + dy;
                        if state.on_map(nx, ny) {
                            let nx = nx as usize;
                            let ny = ny as usize;
                            if visible[nx][ny].is_none() {
                                visible[nx][ny] = Some(state.sectors[nx][ny].clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(visible)
    }

    /// Get nation data (without password) for a specific nation
    pub async fn get_nation(
        &self,
        game_id: Uuid,
        nation_id: u8,
    ) -> Result<Nation, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        if nation_id as usize >= NTOTAL {
            return Err(DbError::NotFound(format!("Nation {}", nation_id)));
        }
        let mut nation = game.state.nations[nation_id as usize].clone();
        nation.password = String::new(); // Never expose password
        Ok(nation)
    }

    /// Get public info for all nations (name, race, class, mark — no hidden stats)
    pub async fn get_public_nations(
        &self,
        game_id: Uuid,
    ) -> Result<Vec<PublicNationInfo>, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        let mut nations = Vec::new();
        for (i, n) in game.state.nations.iter().enumerate() {
            if n.is_active() {
                nations.push(PublicNationInfo {
                    nation_id: i as u8,
                    name: n.name.clone(),
                    leader: n.leader.clone(),
                    race: n.race,
                    class: n.class,
                    mark: n.mark,
                    score: n.score,
                    total_sectors: n.total_sectors,
                });
            }
        }
        Ok(nations)
    }

    /// Get budget/spreadsheet for a nation
    pub async fn get_budget(
        &self,
        game_id: Uuid,
        nation_id: u8,
    ) -> Result<Spreadsheet, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        if nation_id as usize >= NTOTAL {
            return Err(DbError::NotFound(format!("Nation {}", nation_id)));
        }
        // Calculate spreadsheet from current state
        let sprd = conquer_engine::economy::spreadsheet(
            &game.state,
            nation_id as usize,
        );
        Ok(sprd)
    }

    /// Get scores for all active nations
    pub async fn get_scores(&self, game_id: Uuid) -> Result<Vec<ScoreEntry>, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        let mut scores: Vec<ScoreEntry> = game.state.nations.iter().enumerate()
            .filter(|(_, n)| n.is_active())
            .map(|(i, n)| ScoreEntry {
                nation_id: i as u8,
                name: n.name.clone(),
                race: n.race,
                score: n.score,
            })
            .collect();
        scores.sort_by(|a, b| b.score.cmp(&a.score));
        Ok(scores)
    }

    // ========================================================
    // User profile operations (T409-T411)
    // ========================================================

    /// Get user profile with game history
    pub async fn get_user_profile(&self, user_id: Uuid) -> Result<UserProfile, DbError> {
        let users = self.users.read().await;
        let user = users.get(&user_id)
            .ok_or_else(|| DbError::NotFound(format!("User {}", user_id)))?;

        let games = self.games.read().await;
        let mut history = Vec::new();
        let mut won = 0u32;
        let mut lost = 0u32;

        for game in games.values() {
            if let Some(player) = game.players.iter().find(|p| p.user_id == user_id) {
                let nation = &game.state.nations[player.nation_id as usize];
                let outcome = if game.info.status == GameStatus::Completed {
                    // Simple heuristic: highest score wins
                    let max_score = game.state.nations.iter()
                        .filter(|n| n.is_active())
                        .map(|n| n.score)
                        .max()
                        .unwrap_or(0);
                    if nation.score == max_score && nation.is_active() {
                        won += 1;
                        "won".to_string()
                    } else {
                        lost += 1;
                        "lost".to_string()
                    }
                } else if !nation.is_active() {
                    lost += 1;
                    "eliminated".to_string()
                } else {
                    "active".to_string()
                };
                history.push(GameHistoryEntry {
                    game_id: game.info.id,
                    game_name: game.info.name.clone(),
                    nation_name: nation.name.clone(),
                    race: nation.race,
                    class: nation.class,
                    final_score: nation.score,
                    outcome,
                    joined_at: player.joined_at,
                });
            }
        }

        Ok(UserProfile {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            created_at: user.created_at,
            games_played: history.len() as u32,
            games_won: won,
            games_lost: lost,
            game_history: history,
        })
    }

    /// Update user profile (T410)
    pub async fn update_user_profile(
        &self,
        user_id: Uuid,
        display_name: Option<&str>,
        email: Option<&str>,
    ) -> Result<User, DbError> {
        let mut users = self.users.write().await;
        let user = users.get_mut(&user_id)
            .ok_or_else(|| DbError::NotFound(format!("User {}", user_id)))?;

        if let Some(name) = display_name {
            user.display_name = name.to_string();
        }
        if let Some(new_email) = email {
            let email_lower = new_email.to_lowercase();
            if email_lower != user.email {
                // Check uniqueness
                let idx = self.email_index.read().await;
                if idx.contains_key(&email_lower) {
                    return Err(DbError::AlreadyExists("Email already registered".to_string()));
                }
                drop(idx);
                let mut idx = self.email_index.write().await;
                idx.remove(&user.email);
                idx.insert(email_lower.clone(), user_id);
                user.email = email_lower;
            }
        }
        Ok(user.clone())
    }

    /// Change password (T410)
    pub async fn change_password(
        &self,
        user_id: Uuid,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), DbError> {
        let mut users = self.users.write().await;
        let user = users.get_mut(&user_id)
            .ok_or_else(|| DbError::NotFound(format!("User {}", user_id)))?;

        if !AuthManager::verify_password(old_password, &user.password_hash)? {
            return Err(DbError::AuthError("Incorrect current password".to_string()));
        }

        user.password_hash = AuthManager::hash_password(new_password)?;
        Ok(())
    }

    // ========================================================
    // Game settings (T415-T418)
    // ========================================================

    /// Update game settings (creator only)
    pub async fn update_game_settings(
        &self,
        game_id: Uuid,
        user_id: Uuid,
        settings: GameSettings,
    ) -> Result<GameInfo, DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        // Only creator can update settings
        if game.info.settings.creator_id != Some(user_id) {
            return Err(DbError::Unauthorized("Only game creator can modify settings".to_string()));
        }

        // Can only change settings before game starts
        if game.info.status != GameStatus::WaitingForPlayers {
            return Err(DbError::InvalidState("Cannot change settings after game starts".to_string()));
        }

        game.info.settings = settings;
        game.info.updated_at = Utc::now();
        Ok(game.info.clone())
    }

    /// Check if a user is the game creator/admin
    pub async fn is_game_admin(&self, game_id: Uuid, user_id: Uuid) -> Result<bool, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        Ok(game.info.settings.creator_id == Some(user_id))
    }

    // ========================================================
    // Invite management (T419-T422)
    // ========================================================

    /// List invites for a game
    pub async fn list_invites(&self, game_id: Uuid) -> Result<Vec<GameInvite>, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        Ok(game.invites.clone())
    }

    /// Revoke an invite
    pub async fn revoke_invite(&self, game_id: Uuid, invite_id: Uuid) -> Result<(), DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        let idx = game.invites.iter().position(|i| i.id == invite_id)
            .ok_or_else(|| DbError::NotFound(format!("Invite {}", invite_id)))?;
        let code = game.invites[idx].invite_code.clone();
        game.invites.remove(idx);
        self.invite_index.write().await.remove(&code);
        Ok(())
    }

    /// List public games for the game browser (T422)
    pub async fn list_public_games(&self) -> Vec<GameInfo> {
        let games = self.games.read().await;
        games.values()
            .filter(|g| g.info.settings.public_game && g.info.status != GameStatus::Completed)
            .map(|g| g.info.clone())
            .collect()
    }

    // ========================================================
    // Admin operations (T423-T427)
    // ========================================================

    /// Pause/resume a game (game admin)
    pub async fn set_game_status(
        &self,
        game_id: Uuid,
        status: GameStatus,
    ) -> Result<GameInfo, DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        game.info.status = status;
        game.info.updated_at = Utc::now();
        Ok(game.info.clone())
    }

    /// Kick a player from a game
    pub async fn kick_player(
        &self,
        game_id: Uuid,
        nation_id: u8,
    ) -> Result<(), DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        game.players.retain(|p| p.nation_id != nation_id);
        game.info.player_count = game.players.len();
        // Deactivate the nation
        if (nation_id as usize) < NTOTAL {
            game.state.nations[nation_id as usize].active = 0;
        }
        game.info.updated_at = Utc::now();
        Ok(())
    }

    /// Save a turn snapshot for rollback (T426)
    pub async fn save_turn_snapshot(&self, game_id: Uuid) -> Result<(), DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        let state_json = serde_json::to_string(&game.state)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        let snapshot = TurnSnapshot {
            game_id,
            turn: game.state.world.turn,
            state_json,
            created_at: Utc::now(),
        };
        game.turn_snapshots.push(snapshot);

        // Keep max 10 snapshots
        if game.turn_snapshots.len() > 10 {
            game.turn_snapshots.drain(..game.turn_snapshots.len() - 10);
        }
        Ok(())
    }

    /// Rollback to a previous turn (T426)
    pub async fn rollback_turn(
        &self,
        game_id: Uuid,
        target_turn: i16,
    ) -> Result<i16, DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        let snapshot = game.turn_snapshots.iter()
            .find(|s| s.turn == target_turn)
            .cloned()
            .ok_or_else(|| DbError::NotFound(format!("No snapshot for turn {}", target_turn)))?;

        let restored: GameState = serde_json::from_str(&snapshot.state_json)
            .map_err(|e| DbError::SerializationError(e.to_string()))?;

        game.state = restored;
        game.info.current_turn = target_turn;
        game.info.updated_at = Utc::now();

        // Reset player done flags
        for player in &mut game.players {
            player.is_done_this_turn = false;
        }

        // Remove snapshots after this turn
        game.turn_snapshots.retain(|s| s.turn <= target_turn);

        Ok(target_turn)
    }

    /// List available turn snapshots for rollback
    pub async fn list_turn_snapshots(&self, game_id: Uuid) -> Result<Vec<(i16, DateTime<Utc>)>, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        Ok(game.turn_snapshots.iter()
            .map(|s| (s.turn, s.created_at))
            .collect())
    }

    // ========================================================
    // Spectator operations (T428-T431)
    // ========================================================

    /// Join as spectator
    pub async fn join_as_spectator(
        &self,
        game_id: Uuid,
        user_id: Uuid,
    ) -> Result<Spectator, DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        // Check not already a player
        if game.players.iter().any(|p| p.user_id == user_id) {
            return Err(DbError::InvalidState("Already a player in this game".to_string()));
        }
        // Check not already a spectator
        if game.spectators.iter().any(|s| s.user_id == user_id) {
            return Err(DbError::AlreadyExists("Already spectating".to_string()));
        }

        let spec = Spectator {
            game_id,
            user_id,
            joined_at: Utc::now(),
        };
        game.spectators.push(spec.clone());
        Ok(spec)
    }

    /// Leave spectator mode
    pub async fn leave_spectator(
        &self,
        game_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), DbError> {
        let mut games = self.games.write().await;
        let game = games.get_mut(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;
        game.spectators.retain(|s| s.user_id != user_id);
        Ok(())
    }

    /// Check if user is a spectator
    pub async fn is_spectator(&self, game_id: Uuid, user_id: Uuid) -> bool {
        let games = self.games.read().await;
        games.get(&game_id)
            .map(|g| g.spectators.iter().any(|s| s.user_id == user_id))
            .unwrap_or(false)
    }

    /// Get spectator-visible map (public info, no fog of war bypass)
    pub async fn get_spectator_map(
        &self,
        game_id: Uuid,
    ) -> Result<Vec<Vec<Option<Sector>>>, DbError> {
        let games = self.games.read().await;
        let game = games.get(&game_id)
            .ok_or_else(|| DbError::NotFound(format!("Game {}", game_id)))?;

        let state = &game.state;
        let map_x = state.world.map_x as usize;
        let map_y = state.world.map_y as usize;

        // Spectators see sectors owned by any active nation (union of public info)
        let mut visible = vec![vec![None; map_y]; map_x];
        for x in 0..map_x {
            for y in 0..map_y {
                let sector = &state.sectors[x][y];
                if sector.owner > 0 && (sector.owner as usize) < NTOTAL
                    && state.nations[sector.owner as usize].is_active() {
                    visible[x][y] = Some(sector.clone());
                }
            }
        }
        Ok(visible)
    }

    // ========================================================
    // Notification operations (T432-T434)
    // ========================================================

    /// Add a notification for a user
    pub async fn add_notification(
        &self,
        user_id: Uuid,
        event_type: NotificationType,
        game_id: Option<Uuid>,
        message: &str,
    ) -> Result<Notification, DbError> {
        // Check if user has this event enabled
        let prefs = self.notification_prefs.read().await;
        let user_prefs = prefs.get(&user_id).cloned().unwrap_or_default();
        let enabled = match event_type {
            NotificationType::YourTurn => user_prefs.your_turn,
            NotificationType::GameStarted => user_prefs.game_started,
            NotificationType::GameInvite => user_prefs.game_invite,
            NotificationType::UnderAttack => user_prefs.under_attack,
            NotificationType::TurnAdvanced => user_prefs.turn_advanced,
            NotificationType::PlayerJoined => user_prefs.player_joined,
            NotificationType::GameCompleted => user_prefs.game_completed,
        };
        drop(prefs);

        if !enabled {
            return Err(DbError::InvalidState("Notification disabled by user preferences".to_string()));
        }

        let notif = Notification {
            id: Uuid::new_v4(),
            user_id,
            event_type,
            game_id,
            message: message.to_string(),
            read: false,
            created_at: Utc::now(),
        };

        let mut notifications = self.notifications.write().await;
        notifications.entry(user_id).or_insert_with(Vec::new).push(notif.clone());

        // Cap at 100 per user
        if let Some(list) = notifications.get_mut(&user_id) {
            if list.len() > 100 {
                list.drain(..list.len() - 100);
            }
        }

        Ok(notif)
    }

    /// Get notifications for a user
    pub async fn get_notifications(
        &self,
        user_id: Uuid,
        unread_only: bool,
    ) -> Vec<Notification> {
        let notifications = self.notifications.read().await;
        notifications.get(&user_id)
            .map(|list| {
                if unread_only {
                    list.iter().filter(|n| !n.read).cloned().collect()
                } else {
                    list.clone()
                }
            })
            .unwrap_or_default()
    }

    /// Mark a notification as read
    pub async fn mark_notification_read(
        &self,
        user_id: Uuid,
        notif_id: Uuid,
    ) -> Result<(), DbError> {
        let mut notifications = self.notifications.write().await;
        if let Some(list) = notifications.get_mut(&user_id) {
            if let Some(notif) = list.iter_mut().find(|n| n.id == notif_id) {
                notif.read = true;
                return Ok(());
            }
        }
        Err(DbError::NotFound(format!("Notification {}", notif_id)))
    }

    /// Mark all notifications as read
    pub async fn mark_all_read(&self, user_id: Uuid) {
        let mut notifications = self.notifications.write().await;
        if let Some(list) = notifications.get_mut(&user_id) {
            for n in list.iter_mut() {
                n.read = true;
            }
        }
    }

    /// Get notification preferences
    pub async fn get_notification_prefs(&self, user_id: Uuid) -> NotificationPreferences {
        let prefs = self.notification_prefs.read().await;
        prefs.get(&user_id).cloned().unwrap_or_default()
    }

    /// Update notification preferences
    pub async fn set_notification_prefs(
        &self,
        user_id: Uuid,
        prefs: NotificationPreferences,
    ) {
        let mut all_prefs = self.notification_prefs.write().await;
        all_prefs.insert(user_id, prefs);
    }

    /// Broadcast notifications to all players in a game for an event
    pub async fn notify_game_players(
        &self,
        game_id: Uuid,
        event_type: NotificationType,
        message: &str,
        exclude_user: Option<Uuid>,
    ) {
        let player_user_ids: Vec<Uuid> = {
            let games = self.games.read().await;
            match games.get(&game_id) {
                Some(g) => g.players.iter()
                    .filter(|p| exclude_user.map_or(true, |ex| p.user_id != ex))
                    .map(|p| p.user_id)
                    .collect(),
                None => return,
            }
        };

        for uid in player_user_ids {
            let _ = self.add_notification(uid, event_type, Some(game_id), message).await;
        }
    }

    // ========================================================
    // Server stats (T427)
    // ========================================================

    /// Get server stats
    pub async fn server_stats(&self) -> ServerStats {
        let games = self.games.read().await;
        let users = self.users.read().await;
        let active_games = games.values()
            .filter(|g| g.info.status == GameStatus::Active || g.info.status == GameStatus::WaitingForPlayers)
            .count();
        let total_players: usize = games.values().map(|g| g.players.len()).sum();
        ServerStats {
            total_games: games.len(),
            active_games,
            total_users: users.len(),
            total_players,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStats {
    pub total_games: usize,
    pub active_games: usize,
    pub total_users: usize,
    pub total_players: usize,
}

// ============================================================
// Public nation info (no hidden stats)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicNationInfo {
    pub nation_id: u8,
    pub name: String,
    pub leader: String,
    pub race: char,
    pub class: i16,
    pub mark: char,
    pub score: i64,
    pub total_sectors: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreEntry {
    pub nation_id: u8,
    pub name: String,
    pub race: char,
    pub score: i64,
}

// ============================================================
// Action application helper
// ============================================================

fn apply_action_to_state(state: &mut GameState, action: &Action) {
    match action {
        Action::AdjustArmyStat { nation, army, status } => {
            let n = *nation as usize;
            let a = *army as usize;
            if n < NTOTAL && a < MAXARM {
                state.nations[n].armies[a].status = *status;
            }
        }
        Action::AdjustArmyMen { nation, army, soldiers, unit_type } => {
            let n = *nation as usize;
            let a = *army as usize;
            if n < NTOTAL && a < MAXARM {
                state.nations[n].armies[a].soldiers = *soldiers;
                state.nations[n].armies[a].unit_type = *unit_type;
            }
        }
        Action::MoveArmy { nation, army, x, y } => {
            let n = *nation as usize;
            let a = *army as usize;
            if n < NTOTAL && a < MAXARM {
                state.nations[n].armies[a].x = *x as u8;
                state.nations[n].armies[a].y = *y as u8;
            }
        }
        Action::MoveNavy { nation, fleet, x, y } => {
            let n = *nation as usize;
            let f = *fleet as usize;
            if n < NTOTAL && f < MAXNAVY {
                state.nations[n].navies[f].x = *x as u8;
                state.nations[n].navies[f].y = *y as u8;
            }
        }
        Action::DesignateSector { nation: _, x, y, designation } => {
            let sx = *x as usize;
            let sy = *y as usize;
            if sx < state.sectors.len() && sy < state.sectors[0].len() {
                if let Some(d) = Designation::from_char(*designation) {
                    state.sectors[sx][sy].designation = d as u8;
                }
            }
        }
        Action::TakeSectorOwnership { nation, x, y } => {
            let sx = *x as usize;
            let sy = *y as usize;
            if sx < state.sectors.len() && sy < state.sectors[0].len() {
                state.sectors[sx][sy].owner = *nation as u8;
            }
        }
        Action::AdjustDiplomacy { nation_a, nation_b, status } => {
            let a = *nation_a as usize;
            let b = *nation_b as usize;
            if a < NTOTAL && b < NTOTAL {
                state.nations[a].diplomacy[b] = *status as u8;
            }
        }
        Action::AdjustNavyGold { nation, gold } => {
            let n = *nation as usize;
            if n < NTOTAL {
                state.nations[n].treasury_gold += gold;
            }
        }
        Action::IncreaseFort { nation: _, x, y } => {
            let sx = *x as usize;
            let sy = *y as usize;
            if sx < state.sectors.len() && sy < state.sectors[0].len() {
                state.sectors[sx][sy].fortress = state.sectors[sx][sy].fortress.saturating_add(1);
            }
        }
        Action::ChangeMagic { nation, powers, new_power: _ } => {
            let n = *nation as usize;
            if n < NTOTAL {
                state.nations[n].powers = *powers;
            }
        }
        Action::AdjustSpellPoints { nation, cost } => {
            let n = *nation as usize;
            if n < NTOTAL {
                state.nations[n].spell_points -= *cost as i16;
            }
        }
        Action::AdjustSectorCiv { nation: _, people, x, y } => {
            let sx = *x as usize;
            let sy = *y as usize;
            if sx < state.sectors.len() && sy < state.sectors[0].len() {
                state.sectors[sx][sy].people = *people;
            }
        }
        Action::AddSectorCiv { nation: _, people, x, y } => {
            let sx = *x as usize;
            let sy = *y as usize;
            if sx < state.sectors.len() && sy < state.sectors[0].len() {
                state.sectors[sx][sy].people += people;
            }
        }
        Action::AdjustArmyMove { nation, army, movement } => {
            let n = *nation as usize;
            let a = *army as usize;
            if n < NTOTAL && a < MAXARM {
                state.nations[n].armies[a].movement = *movement as u8;
            }
        }
        Action::AdjustNavyMove { nation, fleet, movement } => {
            let n = *nation as usize;
            let f = *fleet as usize;
            if n < NTOTAL && f < MAXNAVY {
                state.nations[n].navies[f].movement = *movement as u8;
            }
        }
        Action::IncreaseAttack { nation } => {
            let n = *nation as usize;
            if n < NTOTAL {
                state.nations[n].attack_plus += 1;
            }
        }
        Action::IncreaseDefense { nation } => {
            let n = *nation as usize;
            if n < NTOTAL {
                state.nations[n].defense_plus += 1;
            }
        }
        Action::DestroyNation { target, by: _ } => {
            let t = *target as usize;
            if t < NTOTAL {
                state.nations[t].active = 0;
            }
        }
        Action::ChangeName { nation, name } => {
            let n = *nation as usize;
            if n < NTOTAL {
                state.nations[n].name = name.clone();
            }
        }
        Action::ChangePassword { nation, password } => {
            let n = *nation as usize;
            if n < NTOTAL {
                state.nations[n].password = password.clone();
            }
        }
        Action::AdjustNavyMerchant { nation, fleet, merchant } => {
            let n = *nation as usize;
            let f = *fleet as usize;
            if n < NTOTAL && f < MAXNAVY {
                state.nations[n].navies[f].merchant = *merchant as u16;
            }
        }
        Action::AdjustNavyCrew { nation, fleet, crew, army_num } => {
            let n = *nation as usize;
            let f = *fleet as usize;
            if n < NTOTAL && f < MAXNAVY {
                state.nations[n].navies[f].crew = *crew as u8;
                state.nations[n].navies[f].army_num = *army_num as u8;
            }
        }
        Action::AdjustNavyWarships { nation, fleet, warships } => {
            let n = *nation as usize;
            let f = *fleet as usize;
            if n < NTOTAL && f < MAXNAVY {
                state.nations[n].navies[f].warships = *warships as u16;
            }
        }
        Action::AdjustNavyGalleys { nation, fleet, galleys } => {
            let n = *nation as usize;
            let f = *fleet as usize;
            if n < NTOTAL && f < MAXNAVY {
                state.nations[n].navies[f].galleys = *galleys as u16;
            }
        }
        Action::AdjustNavyHold { nation, fleet, army_num, people } => {
            let n = *nation as usize;
            let f = *fleet as usize;
            if n < NTOTAL && f < MAXNAVY {
                state.nations[n].navies[f].army_num = *army_num as u8;
                state.nations[n].navies[f].people = *people as u8;
            }
        }
        Action::AdjustPopulation { nation, popularity, terror, reputation } => {
            let n = *nation as usize;
            if n < NTOTAL {
                state.nations[n].popularity = *popularity as u8;
                state.nations[n].terror = *terror as u8;
                state.nations[n].reputation = *reputation as u8;
            }
        }
        Action::AdjustTax { nation, tax_rate, active, charity } => {
            let n = *nation as usize;
            if n < NTOTAL {
                state.nations[n].tax_rate = *tax_rate as u8;
                state.nations[n].active = *active as u8;
                state.nations[n].charity = *charity as u8;
            }
        }
        Action::BribeNation { nation: _, cost: _, target: _ } => {
            // Bribery is complex — handled by engine during turn processing
        }
        Action::HireMercenaries { nation: _, men: _ } => {
            // Handled by engine during turn processing
        }
        Action::DisbandToMerc { nation: _, men: _, attack: _, defense: _ } => {
            // Handled by engine during turn processing
        }
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_get_user() {
        let store = GameStore::new();
        let user = store.create_user("testuser", "test@example.com", "password123", None)
            .await.unwrap();
        assert_eq!(user.username, "testuser");

        let fetched = store.get_user(user.id).await.unwrap();
        assert_eq!(fetched.username, "testuser");
    }

    #[tokio::test]
    async fn test_authenticate_user() {
        let store = GameStore::new();
        store.create_user("testuser", "test@example.com", "password123", None)
            .await.unwrap();

        let user = store.authenticate_user("testuser", "password123").await.unwrap();
        assert_eq!(user.username, "testuser");

        let err = store.authenticate_user("testuser", "wrong").await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn test_duplicate_user() {
        let store = GameStore::new();
        store.create_user("testuser", "test@example.com", "pass", None).await.unwrap();
        let err = store.create_user("testuser", "other@example.com", "pass", None).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn test_create_and_list_games() {
        let store = GameStore::new();
        let settings = GameSettings::default();
        let game = store.create_game("Test Game", settings).await.unwrap();
        assert_eq!(game.name, "Test Game");
        assert_eq!(game.status, GameStatus::WaitingForPlayers);

        let games = store.list_games(None).await;
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, game.id);
    }

    #[tokio::test]
    async fn test_join_game() {
        let store = GameStore::new();
        let user = store.create_user("player1", "p1@test.com", "pass", None).await.unwrap();
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();

        let player = store.join_game(
            game.id, user.id, "Gondor", "Aragorn", 'H', 1, 'G'
        ).await.unwrap();
        assert!(player.nation_id >= 1);

        // Can't join twice
        let err = store.join_game(
            game.id, user.id, "Rohan", "Theoden", 'H', 1, 'R'
        ).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn test_submit_and_get_actions() {
        let store = GameStore::new();
        let user = store.create_user("player1", "p1@test.com", "pass", None).await.unwrap();
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();
        let player = store.join_game(
            game.id, user.id, "Gondor", "Aragorn", 'H', 1, 'G'
        ).await.unwrap();

        let action = Action::MoveArmy {
            nation: player.nation_id as i16,
            army: 0,
            x: 10,
            y: 10,
        };
        store.submit_action(game.id, player.nation_id, action.clone()).await.unwrap();

        let actions = store.get_actions(game.id, player.nation_id).await.unwrap();
        assert_eq!(actions.len(), 1);
    }

    #[tokio::test]
    async fn test_run_turn() {
        let store = GameStore::new();
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();
        let initial_turn = game.current_turn;

        let new_turn = store.run_turn(game.id).await.unwrap();
        assert_eq!(new_turn, initial_turn + 1);

        let info = store.get_game_info(game.id).await.unwrap();
        assert_eq!(info.current_turn, new_turn);
    }

    #[tokio::test]
    async fn test_chat() {
        let store = GameStore::new();
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();

        store.send_chat(game.id, Some(1), "public", "Hello world!").await.unwrap();
        store.send_chat(game.id, Some(2), "public", "Hi there!").await.unwrap();
        store.send_chat(game.id, Some(1), "nation_1_2", "Secret message").await.unwrap();

        let public = store.get_chat(game.id, "public", 50, None).await.unwrap();
        assert_eq!(public.len(), 2);

        let private = store.get_chat(game.id, "nation_1_2", 50, None).await.unwrap();
        assert_eq!(private.len(), 1);
    }

    #[tokio::test]
    async fn test_chat_rate_limiting() {
        let store = GameStore::new();
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();

        // Send 5 messages — should all succeed
        for i in 0..5 {
            store.send_chat(game.id, Some(1), "public", &format!("msg {}", i)).await.unwrap();
        }
        // 6th should be rate limited
        let result = store.send_chat(game.id, Some(1), "public", "spam").await;
        assert!(result.is_err());

        // System messages bypass rate limiting
        store.send_system_message(game.id, "public", "System msg").await.unwrap();
    }

    #[tokio::test]
    async fn test_private_channels() {
        let store = GameStore::new();
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();

        let channel = GameStore::private_channel_name(3, 1);
        assert_eq!(channel, "nation_1_3"); // sorted

        store.send_chat(game.id, Some(1), &channel, "Hello nation 3").await.unwrap();

        // Nation 1 can see it
        let msgs = store.get_chat_for_nation(game.id, 1, &channel, 50, None).await.unwrap();
        assert_eq!(msgs.len(), 1);

        // Nation 3 can see it
        let msgs = store.get_chat_for_nation(game.id, 3, &channel, 50, None).await.unwrap();
        assert_eq!(msgs.len(), 1);

        // Nation 2 cannot
        let result = store.get_chat_for_nation(game.id, 2, &channel, 50, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_system_messages() {
        let store = GameStore::new();
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();

        let msg = store.send_system_message(game.id, "public", "Turn 2 has begun").await.unwrap();
        assert!(msg.is_system);
        assert_eq!(msg.sender_name, "SYSTEM");
        assert!(msg.sender_nation_id.is_none());
    }

    #[tokio::test]
    async fn test_chat_sender_name() {
        let store = GameStore::new();
        let user = store.create_user("p1", "p1@test.com", "pass", None).await.unwrap();
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();
        store.join_game(game.id, user.id, "Gondor", "Aragorn", 'H', 1, 'G').await.unwrap();

        // Send from the joined nation (nation_id from join)
        let players = store.list_players(game.id).await.unwrap();
        let nid = players[0].nation_id;
        let msg = store.send_chat(game.id, Some(nid), "public", "Hello!").await.unwrap();
        assert!(msg.sender_name.contains("Gondor"));
        assert!(msg.sender_name.contains("Aragorn"));
    }

    #[tokio::test]
    async fn test_invites() {
        let store = GameStore::new();
        let user = store.create_user("admin", "admin@test.com", "pass", None).await.unwrap();
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();

        let invite = store.create_invite(game.id, user.id, Some(5), Some(24.0)).await.unwrap();
        assert_eq!(invite.uses, 0);

        let (found, info) = store.get_invite(&invite.invite_code).await.unwrap();
        assert_eq!(found.game_id, game.id);
        assert_eq!(info.name, "Test");

        store.use_invite(&invite.invite_code).await.unwrap();
        let (updated, _) = store.get_invite(&invite.invite_code).await.unwrap();
        assert_eq!(updated.uses, 1);
    }

    #[tokio::test]
    async fn test_visible_map() {
        let store = GameStore::new();
        let user = store.create_user("player1", "p1@test.com", "pass", None).await.unwrap();
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();
        let player = store.join_game(
            game.id, user.id, "Test", "Leader", 'H', 1, 'T'
        ).await.unwrap();

        let map = store.get_visible_map(game.id, player.nation_id).await.unwrap();
        assert_eq!(map.len(), 32);
        assert_eq!(map[0].len(), 32);
        // Not everything is visible
        let visible_count: usize = map.iter()
            .flat_map(|col| col.iter())
            .filter(|s| s.is_some())
            .count();
        // Should see some sectors but not all (newly joined nation)
        assert!(visible_count < 32 * 32);
    }

    #[tokio::test]
    async fn test_player_done_flags() {
        let store = GameStore::new();
        let user1 = store.create_user("p1", "p1@test.com", "pass", None).await.unwrap();
        let user2 = store.create_user("p2", "p2@test.com", "pass", None).await.unwrap();
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();
        store.join_game(game.id, user1.id, "Nation1", "L1", 'H', 1, 'A').await.unwrap();
        store.join_game(game.id, user2.id, "Nation2", "L2", 'E', 1, 'B').await.unwrap();

        assert!(!store.all_players_done(game.id).await.unwrap());

        store.set_player_done(game.id, user1.id, true).await.unwrap();
        assert!(!store.all_players_done(game.id).await.unwrap());

        store.set_player_done(game.id, user2.id, true).await.unwrap();
        assert!(store.all_players_done(game.id).await.unwrap());
    }

    // ========================================================
    // Phase 6 tests
    // ========================================================

    #[tokio::test]
    async fn test_user_profile() {
        let store = GameStore::new();
        let user = store.create_user("player1", "p1@test.com", "pass", Some("Player One")).await.unwrap();

        let profile = store.get_user_profile(user.id).await.unwrap();
        assert_eq!(profile.display_name, "Player One");
        assert_eq!(profile.games_played, 0);

        // Join a game
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();
        store.join_game(game.id, user.id, "Gondor", "Aragorn", 'H', 1, 'G').await.unwrap();

        let profile = store.get_user_profile(user.id).await.unwrap();
        assert_eq!(profile.games_played, 1);
        assert_eq!(profile.game_history[0].nation_name, "Gondor");
    }

    #[tokio::test]
    async fn test_update_profile() {
        let store = GameStore::new();
        let user = store.create_user("player1", "p1@test.com", "pass", None).await.unwrap();

        let updated = store.update_user_profile(user.id, Some("New Name"), None).await.unwrap();
        assert_eq!(updated.display_name, "New Name");
    }

    #[tokio::test]
    async fn test_change_password() {
        let store = GameStore::new();
        let user = store.create_user("player1", "p1@test.com", "oldpass", None).await.unwrap();

        // Wrong old password fails
        let err = store.change_password(user.id, "wrong", "newpass").await;
        assert!(err.is_err());

        // Correct old password succeeds
        store.change_password(user.id, "oldpass", "newpass").await.unwrap();

        // New password works
        let authed = store.authenticate_user("player1", "newpass").await.unwrap();
        assert_eq!(authed.id, user.id);
    }

    #[tokio::test]
    async fn test_game_admin() {
        let store = GameStore::new();
        let user = store.create_user("creator", "c@test.com", "pass", None).await.unwrap();

        let mut settings = GameSettings::default();
        settings.creator_id = Some(user.id);
        let game = store.create_game("AdminTest", settings).await.unwrap();

        assert!(store.is_game_admin(game.id, user.id).await.unwrap());

        let other = store.create_user("other", "o@test.com", "pass", None).await.unwrap();
        assert!(!store.is_game_admin(game.id, other.id).await.unwrap());
    }

    #[tokio::test]
    async fn test_kick_player() {
        let store = GameStore::new();
        let user = store.create_user("p1", "p1@test.com", "pass", None).await.unwrap();
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();
        let player = store.join_game(game.id, user.id, "Gondor", "Aragorn", 'H', 1, 'G').await.unwrap();

        store.kick_player(game.id, player.nation_id).await.unwrap();

        let players = store.list_players(game.id).await.unwrap();
        assert!(players.is_empty());
    }

    #[tokio::test]
    async fn test_turn_rollback() {
        let store = GameStore::new();
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();
        let initial_turn = game.current_turn;

        // Advance a couple turns (each saves a snapshot)
        let t1 = store.run_turn(game.id).await.unwrap();
        let t2 = store.run_turn(game.id).await.unwrap();
        assert_eq!(t2, initial_turn + 2);

        // Rollback to turn 1
        let rolled = store.rollback_turn(game.id, t1).await.unwrap();
        assert_eq!(rolled, t1);

        let info = store.get_game_info(game.id).await.unwrap();
        assert_eq!(info.current_turn, t1);
    }

    #[tokio::test]
    async fn test_spectator() {
        let store = GameStore::new();
        let user = store.create_user("spec", "spec@test.com", "pass", None).await.unwrap();
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();

        store.join_as_spectator(game.id, user.id).await.unwrap();
        assert!(store.is_spectator(game.id, user.id).await);

        // Can't join twice
        let err = store.join_as_spectator(game.id, user.id).await;
        assert!(err.is_err());

        store.leave_spectator(game.id, user.id).await.unwrap();
        assert!(!store.is_spectator(game.id, user.id).await);
    }

    #[tokio::test]
    async fn test_notifications() {
        let store = GameStore::new();
        let user = store.create_user("p1", "p1@test.com", "pass", None).await.unwrap();

        // Default prefs enable most notifications
        let notif = store.add_notification(
            user.id, NotificationType::YourTurn, None, "It's your turn!",
        ).await.unwrap();
        assert!(!notif.read);

        let unread = store.get_notifications(user.id, true).await;
        assert_eq!(unread.len(), 1);

        store.mark_notification_read(user.id, notif.id).await.unwrap();
        let unread = store.get_notifications(user.id, true).await;
        assert_eq!(unread.len(), 0);

        let all = store.get_notifications(user.id, false).await;
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn test_notification_preferences() {
        let store = GameStore::new();
        let user = store.create_user("p1", "p1@test.com", "pass", None).await.unwrap();

        // Disable player_joined (default is false, but let's be explicit)
        let mut prefs = NotificationPreferences::default();
        prefs.your_turn = false;
        store.set_notification_prefs(user.id, prefs).await;

        // Now your_turn notifications are suppressed
        let result = store.add_notification(
            user.id, NotificationType::YourTurn, None, "your turn",
        ).await;
        assert!(result.is_err()); // should fail because disabled

        // But game_started still works
        let result = store.add_notification(
            user.id, NotificationType::GameStarted, None, "game started",
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invite_management() {
        let store = GameStore::new();
        let user = store.create_user("admin", "a@test.com", "pass", None).await.unwrap();
        let game = store.create_game("Test", GameSettings::default()).await.unwrap();

        let inv1 = store.create_invite(game.id, user.id, Some(5), None).await.unwrap();
        let inv2 = store.create_invite(game.id, user.id, None, None).await.unwrap();

        let invites = store.list_invites(game.id).await.unwrap();
        assert_eq!(invites.len(), 2);

        store.revoke_invite(game.id, inv1.id).await.unwrap();
        let invites = store.list_invites(game.id).await.unwrap();
        assert_eq!(invites.len(), 1);
        assert_eq!(invites[0].id, inv2.id);
    }

    #[tokio::test]
    async fn test_server_stats() {
        let store = GameStore::new();
        store.create_user("p1", "p1@test.com", "pass", None).await.unwrap();
        store.create_game("G1", GameSettings::default()).await.unwrap();

        let stats = store.server_stats().await;
        assert_eq!(stats.total_users, 1);
        assert_eq!(stats.total_games, 1);
        assert_eq!(stats.active_games, 1); // waiting_for_players counts
    }
}
