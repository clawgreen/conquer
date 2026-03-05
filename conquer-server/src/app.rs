// conquer-server/src/app.rs — Application state, router, JWT extraction

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode, header},
    middleware,
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use conquer_db::GameStore;
use crate::config::ServerConfig;
use crate::errors::ApiError;
use crate::jwt::{Claims, JwtManager};
use crate::routes;
use crate::ws::ConnectionManager;

// ============================================================
// Application State
// ============================================================

#[derive(Clone)]
pub struct AppState {
    pub store: GameStore,
    pub jwt: JwtManager,
    pub ws_manager: ConnectionManager,
    pub config: ServerConfig,
}

// ============================================================
// JWT Claims extractor for Axum
// ============================================================

impl FromRequestParts<AppState> for Claims {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header_val = parts.headers.get(header::AUTHORIZATION)
            .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

        let header_str = header_val.to_str()
            .map_err(|_| ApiError::Unauthorized("Invalid Authorization header".to_string()))?;

        let token = header_str
            .strip_prefix("Bearer ")
            .ok_or_else(|| ApiError::Unauthorized("Expected Bearer token".to_string()))?;

        state.jwt.validate_token(token)
            .map_err(|e| ApiError::Unauthorized(format!("Invalid token: {}", e)))
    }
}

// ============================================================
// Router construction
// ============================================================

pub fn build_router(state: AppState) -> Router {
    // CORS (T278)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // API routes
    let api = Router::new()
        // Health (T281)
        .route("/health", get(routes::health::health_check))
        // Auth (T283-T284)
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        // Games (T288-T293)
        .route("/games", post(routes::games::create_game))
        .route("/games", get(routes::games::list_games))
        .route("/games/{id}", get(routes::games::get_game))
        .route("/games/{id}", delete(routes::games::delete_game))
        .route("/games/{id}/join", post(routes::games::join_game))
        // Game state (T296-T304)
        .route("/games/{id}/map", get(routes::state::get_map))
        .route("/games/{id}/nation", get(routes::state::get_nation))
        .route("/games/{id}/nations", get(routes::state::get_nations))
        .route("/games/{id}/armies", get(routes::state::get_armies))
        .route("/games/{id}/navies", get(routes::state::get_navies))
        .route("/games/{id}/sector/{x}/{y}", get(routes::state::get_sector))
        .route("/games/{id}/news", get(routes::state::get_news))
        .route("/games/{id}/scores", get(routes::state::get_scores))
        .route("/games/{id}/budget", get(routes::state::get_budget))
        // Actions (T305-T311)
        .route("/games/{id}/actions", post(routes::actions::submit_actions))
        .route("/games/{id}/actions", get(routes::actions::get_actions))
        .route("/games/{id}/actions/{action_id}", delete(routes::actions::retract_action))
        .route("/games/{id}/end-turn", post(routes::actions::end_turn))
        .route("/games/{id}/run-turn", post(routes::actions::run_turn))
        // WebSocket (T312)
        .route("/games/{id}/ws", get(routes::websocket::ws_upgrade))
        // Invites (T321-T323)
        .route("/games/{id}/invites", post(routes::invites::create_invite))
        .route("/invites/{code}", get(routes::invites::get_invite))
        .route("/invites/{code}/accept", post(routes::invites::accept_invite));

    Router::new()
        .nest("/api", api)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    use serde_json::json;

    fn test_state() -> AppState {
        let config = ServerConfig::default();
        AppState {
            store: GameStore::new(),
            jwt: JwtManager::new(&config.jwt_secret, config.jwt_expiry_hours),
            ws_manager: ConnectionManager::new(),
            config,
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = build_router(test_state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_register_and_login() {
        let state = test_state();
        let app = build_router(state.clone());

        // Register
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/register")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&json!({
                        "username": "testplayer",
                        "email": "test@example.com",
                        "password": "password123",
                    })).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Login
        let app = build_router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/auth/login")
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_string(&json!({
                        "username": "testplayer",
                        "password": "password123",
                    })).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_game_requires_auth() {
        let app = build_router(test_state());

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/games")
                    .header("Content-Type", "application/json")
                    .body(Body::from(r#"{"name":"Test"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_full_game_flow() {
        let state = test_state();

        // 1. Register
        let app = build_router(state.clone());
        let resp = app.oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&json!({
                    "username": "player1",
                    "email": "p1@test.com",
                    "password": "password123",
                })).unwrap()))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
        let auth: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let token = auth["token"].as_str().unwrap();

        // 2. Create game
        let app = build_router(state.clone());
        let resp = app.oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/games")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::from(serde_json::to_string(&json!({
                    "name": "Test Game",
                })).unwrap()))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
        let game: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let game_id = game["id"].as_str().unwrap();

        // 3. Join game
        let app = build_router(state.clone());
        let resp = app.oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/games/{}/join", game_id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::from(serde_json::to_string(&json!({
                    "nation_name": "Gondor",
                    "leader_name": "Aragorn",
                    "race": "H",
                    "class": 1,
                    "mark": "G",
                })).unwrap()))
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // 4. Get nation
        let app = build_router(state.clone());
        let resp = app.oneshot(
            Request::builder()
                .uri(&format!("/api/games/{}/nation", game_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
        let nation: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(nation["name"], "Gondor");

        // 5. Get map
        let app = build_router(state.clone());
        let resp = app.oneshot(
            Request::builder()
                .uri(&format!("/api/games/{}/map", game_id))
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // 6. List games
        let app = build_router(state.clone());
        let resp = app.oneshot(
            Request::builder()
                .uri("/api/games")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        ).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
