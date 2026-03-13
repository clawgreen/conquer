// conquer-server — Axum HTTP/WebSocket server for Conquer
// Phase 7: Deploy & CI — production-ready with static file serving, metrics, graceful shutdown

use conquer_db::GameStore;
use conquer_server::app::{build_router, AppState};
use conquer_server::config::ServerConfig;
use conquer_server::jwt::JwtManager;
use conquer_server::metrics::Metrics;
use conquer_server::ws::ConnectionManager;
use std::sync::Arc;
use tokio::signal;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    let config = ServerConfig::from_env();

    // Initialize tracing with structured logging (T452)
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level)),
        )
        .json()
        .with_target(true)
        .with_thread_ids(true)
        .init();

    let bind_addr = config.bind_addr;

    // Create store — with Postgres if DATABASE_URL is set
    let store = if let Some(ref db_url) = config.database_url {
        let safe_url = if let Some(at) = db_url.find('@') {
            format!(
                "{}...{}",
                &db_url[..db_url[..at].rfind(':').unwrap_or(0).max(15).min(at)],
                &db_url[at..]
            )
        } else {
            db_url.clone()
        };
        tracing::info!("Connecting to Postgres: {}", safe_url);

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(10)
            .connect(db_url)
            .await
            .expect("Failed to connect to Postgres");

        let store = GameStore::with_pool(pool);
        store
            .hydrate()
            .await
            .expect("Failed to hydrate from Postgres");
        tracing::info!("Postgres persistence active — data will survive restarts");
        store
    } else {
        tracing::warn!("No DATABASE_URL set — running in-memory only (data lost on restart)");
        GameStore::new()
    };

    let ws_manager = ConnectionManager::new();
    let metrics = Arc::new(Metrics::new());

    let state = AppState {
        store,
        jwt: JwtManager::new(&config.jwt_secret, config.jwt_expiry_hours),
        ws_manager,
        config,
        metrics,
    };

    let app = build_router(state.clone());

    tracing::info!("Conquer server starting on {}", bind_addr);
    if let Some(ref dir) = state.config.static_dir {
        tracing::info!("Serving static files from {}", dir);
    }

    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("Failed to bind");

    // Graceful shutdown (T455)
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(state))
        .await
        .expect("Server error");

    tracing::info!("Server shut down gracefully");
}

/// Graceful shutdown handler (T455)
/// Saves state, closes WebSocket connections, and cleans up resources
async fn shutdown_signal(state: AppState) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, cleaning up...");

    // Close all WebSocket connections gracefully
    let game_count = state.ws_manager.game_count().await;
    tracing::info!(game_count, "Closing WebSocket connections");
    state.ws_manager.shutdown().await;

    // Log final metrics
    let snapshot = state.metrics.snapshot();
    tracing::info!(
        total_requests = snapshot.total_requests,
        active_connections = snapshot.active_connections,
        "Final metrics before shutdown"
    );

    tracing::info!("Cleanup complete");
}
