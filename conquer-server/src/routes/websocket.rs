// conquer-server/src/routes/websocket.rs — WebSocket upgrade and handling (T312-T320)

use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::time::Duration;
use tokio::time::interval;
use uuid::Uuid;

use crate::app::AppState;
use crate::ws::{ClientMessage, ServerMessage};

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// JWT token for authentication
    pub token: String,
}

/// GET /api/games/:id/ws — WebSocket upgrade (T312)
pub async fn ws_upgrade(
    State(state): State<AppState>,
    Path(game_id): Path<Uuid>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // Validate JWT
    let claims = match state.jwt.validate_token(&query.token) {
        Ok(c) => c,
        Err(_) => {
            return axum::http::Response::builder()
                .status(401)
                .body(axum::body::Body::from("Invalid token"))
                .unwrap()
                .into_response();
        }
    };

    let user_id = match crate::jwt::JwtManager::user_id_from_claims(&claims) {
        Ok(id) => id,
        Err(_) => {
            return axum::http::Response::builder()
                .status(401)
                .body(axum::body::Body::from("Invalid user ID"))
                .unwrap()
                .into_response();
        }
    };

    // Verify player is in this game
    let player = match state.store.get_player(game_id, user_id).await {
        Ok(p) => p,
        Err(_) => {
            return axum::http::Response::builder()
                .status(403)
                .body(axum::body::Body::from("Not a player in this game"))
                .unwrap()
                .into_response();
        }
    };

    let nation_id = player.nation_id;

    ws.on_upgrade(move |socket| handle_ws(socket, state, game_id, nation_id))
        .into_response()
}

/// Handle a WebSocket connection
async fn handle_ws(
    socket: WebSocket,
    state: AppState,
    game_id: Uuid,
    nation_id: u8,
) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to game broadcasts
    let mut broadcast_rx = state.ws_manager.subscribe(game_id).await;

    // Heartbeat interval (T318)
    let heartbeat_secs = state.config.ws_heartbeat_secs;
    let mut heartbeat = interval(Duration::from_secs(heartbeat_secs));

    // Spawn broadcast forwarder
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Forward broadcasts to this client
                msg = broadcast_rx.recv() => {
                    match msg {
                        Ok(server_msg) => {
                            let json = match serde_json::to_string(&server_msg) {
                                Ok(j) => j,
                                Err(_) => continue,
                            };
                            if sender.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(_) => break,
                    }
                }
                // Send heartbeat ping
                _ = heartbeat.tick() => {
                    if sender.send(Message::Ping(vec![].into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Handle incoming messages from client
    let store = state.store.clone();
    let ws_mgr = state.ws_manager.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        match client_msg {
                            ClientMessage::Ping => {
                                // Pong is handled by broadcast forwarder
                                ws_mgr.broadcast(game_id, ServerMessage::Pong).await;
                            }
                            ClientMessage::Action { action } => {
                                // Submit action
                                let _ = store.submit_action(game_id, nation_id, action).await;
                            }
                            ClientMessage::ChatSend { channel, content } => {
                                // Store and broadcast chat
                                if let Ok(msg) = store.send_chat(
                                    game_id, Some(nation_id), &channel, &content,
                                ).await {
                                    ws_mgr.broadcast(game_id, ServerMessage::ChatMessage {
                                        sender_nation_id: Some(nation_id),
                                        channel: msg.channel,
                                        content: msg.content,
                                        timestamp: msg.created_at.to_rfc3339(),
                                    }).await;
                                }
                            }
                        }
                    }
                }
                Message::Pong(_) => {
                    // Client responded to our ping — connection alive
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}
