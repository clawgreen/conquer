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
}

impl GameStore {
    pub fn new() -> Self {
        GameStore {
            games: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
            username_index: Arc::new(RwLock::new(HashMap::new())),
            email_index: Arc::new(RwLock::new(HashMap::new())),
            invite_index: Arc::new(RwLock::new(HashMap::new())),
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
    // Chat operations
    // ========================================================

    /// Send a chat message
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

        let msg = ChatMessage {
            id: Uuid::new_v4(),
            game_id,
            sender_nation_id,
            channel: channel.to_string(),
            content: content.to_string(),
            created_at: Utc::now(),
        };
        game.chat_messages.push(msg.clone());
        Ok(msg)
    }

    /// Get chat messages with pagination
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
}
