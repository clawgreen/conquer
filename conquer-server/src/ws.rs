// conquer-server/src/ws.rs — WebSocket connection manager and message protocol
//
// T312-T320: WebSocket upgrade, per-game broadcast, heartbeat, reconnection

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// WebSocket Message Protocol (T313)
// ============================================================

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum ServerMessage {
    /// Full or partial map update
    MapUpdate {
        #[serde(skip_serializing_if = "Option::is_none")]
        sectors: Option<serde_json::Value>,
    },
    /// Nation data update
    NationUpdate {
        nation_id: u8,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },
    /// Army update
    ArmyUpdate {
        nation_id: u8,
        army_id: u8,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },
    /// News broadcast
    News {
        turn: i16,
        messages: Vec<String>,
    },
    /// Turn has started
    TurnStart {
        turn: i16,
        season: String,
    },
    /// Turn has ended, new turn begins
    TurnEnd {
        old_turn: i16,
        new_turn: i16,
    },
    /// Player joined the game
    PlayerJoined {
        nation_id: u8,
        nation_name: String,
        race: char,
    },
    /// Player marked done for this turn
    PlayerDone {
        nation_id: u8,
        nation_name: String,
    },
    /// Chat message (T388)
    ChatMessage {
        sender_nation_id: Option<u8>,
        sender_name: String,
        channel: String,
        content: String,
        timestamp: String,
        is_system: bool,
    },
    /// Chat history response (sent on connect or request)
    ChatHistory {
        channel: String,
        messages: Vec<ChatHistoryEntry>,
    },
    /// Player presence update (T405)
    PresenceUpdate {
        nation_id: u8,
        status: String, // "online", "offline"
    },
    /// System message
    SystemMessage {
        content: String,
    },
    /// Pong response
    Pong,
    /// Error message
    Error {
        message: String,
    },
}

/// A single chat message entry for history payloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatHistoryEntry {
    pub sender_nation_id: Option<u8>,
    pub sender_name: String,
    pub channel: String,
    pub content: String,
    pub timestamp: String,
    pub is_system: bool,
}

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum ClientMessage {
    /// Submit a game action
    Action {
        action: conquer_core::actions::Action,
    },
    /// Send a chat message (T388)
    ChatSend {
        channel: String,
        content: String,
    },
    /// Request chat history for a channel
    ChatHistoryRequest {
        channel: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        before: Option<String>,
        #[serde(default = "default_chat_limit")]
        limit: usize,
    },
    /// Ping
    Ping,
}

fn default_chat_limit() -> usize { 50 }

// ============================================================
// Connection Manager (T314)
// ============================================================

/// Per-game connection pool for broadcasting messages
#[derive(Debug)]
pub struct GameChannel {
    /// Broadcast sender for this game
    pub sender: broadcast::Sender<ServerMessage>,
}

impl GameChannel {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(256);
        GameChannel { sender }
    }
}

/// Per-connection metadata for presence tracking (T405)
#[derive(Debug, Clone)]
pub struct ConnectedPlayer {
    pub nation_id: u8,
    pub connected_at: std::time::Instant,
}

/// Manages WebSocket connections across all games
#[derive(Clone)]
pub struct ConnectionManager {
    /// game_id -> broadcast channel
    channels: Arc<RwLock<HashMap<Uuid, GameChannel>>>,
    /// game_id -> set of connected nation_ids (for presence)
    presence: Arc<RwLock<HashMap<Uuid, HashMap<u8, usize>>>>, // nation_id -> connection count
}

impl ConnectionManager {
    pub fn new() -> Self {
        ConnectionManager {
            channels: Arc::new(RwLock::new(HashMap::new())),
            presence: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a broadcast channel for a game
    pub async fn get_or_create_channel(&self, game_id: Uuid) -> broadcast::Sender<ServerMessage> {
        let channels = self.channels.read().await;
        if let Some(ch) = channels.get(&game_id) {
            return ch.sender.clone();
        }
        drop(channels);

        let mut channels = self.channels.write().await;
        // Double-check after acquiring write lock
        if let Some(ch) = channels.get(&game_id) {
            return ch.sender.clone();
        }
        let channel = GameChannel::new();
        let sender = channel.sender.clone();
        channels.insert(game_id, channel);
        sender
    }

    /// Subscribe to a game's broadcast channel
    pub async fn subscribe(&self, game_id: Uuid) -> broadcast::Receiver<ServerMessage> {
        let sender = self.get_or_create_channel(game_id).await;
        sender.subscribe()
    }

    /// Broadcast a message to all connections in a game
    pub async fn broadcast(&self, game_id: Uuid, msg: ServerMessage) {
        let channels = self.channels.read().await;
        if let Some(ch) = channels.get(&game_id) {
            // Ignore error (no receivers)
            let _ = ch.sender.send(msg);
        }
    }

    /// Broadcast to a specific nation only (for scoped events)
    /// For now, broadcasts to everyone — nation filtering done client-side
    /// In production, we'd track per-connection nation_id
    pub async fn broadcast_to_nation(&self, game_id: Uuid, _nation_id: u8, msg: ServerMessage) {
        self.broadcast(game_id, msg).await;
    }

    /// Register a player connection (T405 presence)
    pub async fn player_connected(&self, game_id: Uuid, nation_id: u8) {
        let mut presence = self.presence.write().await;
        let game_presence = presence.entry(game_id).or_insert_with(HashMap::new);
        let count = game_presence.entry(nation_id).or_insert(0);
        *count += 1;
    }

    /// Unregister a player connection (T405 presence)
    pub async fn player_disconnected(&self, game_id: Uuid, nation_id: u8) {
        let mut presence = self.presence.write().await;
        if let Some(game_presence) = presence.get_mut(&game_id) {
            if let Some(count) = game_presence.get_mut(&nation_id) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    game_presence.remove(&nation_id);
                }
            }
        }
    }

    /// Get online nation IDs for a game (T405)
    pub async fn get_online_nations(&self, game_id: Uuid) -> Vec<u8> {
        let presence = self.presence.read().await;
        presence.get(&game_id)
            .map(|m| m.keys().copied().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_message_serialization() {
        let msg = ServerMessage::TurnEnd { old_turn: 5, new_turn: 6 };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("turn_end"));
        assert!(json.contains("old_turn"));

        let msg = ServerMessage::PlayerJoined {
            nation_id: 1,
            nation_name: "Gondor".to_string(),
            race: 'H',
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("player_joined"));
        assert!(json.contains("Gondor"));

        let msg = ServerMessage::ChatMessage {
            sender_nation_id: Some(1),
            sender_name: "Gondor (Aragorn)".to_string(),
            channel: "public".to_string(),
            content: "Hello!".to_string(),
            timestamp: "2026-03-05T00:00:00Z".to_string(),
            is_system: false,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("chat_message"));
        assert!(json.contains("sender_name"));

        let msg = ServerMessage::PresenceUpdate {
            nation_id: 1,
            status: "online".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("presence_update"));
    }

    #[test]
    fn test_client_message_deserialization() {
        let json = r#"{"type":"ping","data":null}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Ping));

        let json = r#"{"type":"chat_send","data":{"channel":"public","content":"Hello!"}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::ChatSend { .. }));

        let json = r#"{"type":"chat_history_request","data":{"channel":"public","limit":50}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::ChatHistoryRequest { .. }));
    }

    #[tokio::test]
    async fn test_connection_manager_broadcast() {
        let mgr = ConnectionManager::new();
        let game_id = Uuid::new_v4();

        let mut rx = mgr.subscribe(game_id).await;
        mgr.broadcast(game_id, ServerMessage::Pong).await;

        let msg = rx.recv().await.unwrap();
        assert!(matches!(msg, ServerMessage::Pong));
    }
}
