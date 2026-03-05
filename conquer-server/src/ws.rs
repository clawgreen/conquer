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
    /// Chat message
    ChatMessage {
        sender_nation_id: Option<u8>,
        channel: String,
        content: String,
        timestamp: String,
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

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum ClientMessage {
    /// Submit a game action
    Action {
        action: conquer_core::actions::Action,
    },
    /// Send a chat message
    ChatSend {
        channel: String,
        content: String,
    },
    /// Ping
    Ping,
}

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

/// Manages WebSocket connections across all games
#[derive(Clone)]
pub struct ConnectionManager {
    /// game_id -> broadcast channel
    channels: Arc<RwLock<HashMap<Uuid, GameChannel>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        ConnectionManager {
            channels: Arc::new(RwLock::new(HashMap::new())),
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
    }

    #[test]
    fn test_client_message_deserialization() {
        let json = r#"{"type":"ping","data":null}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::Ping));

        let json = r#"{"type":"chat_send","data":{"channel":"public","content":"Hello!"}}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, ClientMessage::ChatSend { .. }));
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
