// conquer-server — Axum HTTP/WebSocket server for Conquer (Phase 3)

use conquer_db::GameStore;
use conquer_server::app::{build_router, AppState};
use conquer_server::config::ServerConfig;
use conquer_server::jwt::JwtManager;
use conquer_server::ws::ConnectionManager;
use tokio::signal;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    // Initialize tracing (T279)
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,conquer_server=debug")),
        )
        .json()
        .init();

    let config = ServerConfig::default();
    let bind_addr = config.bind_addr;

    let state = AppState {
        store: GameStore::new(),
        jwt: JwtManager::new(&config.jwt_secret, config.jwt_expiry_hours),
        ws_manager: ConnectionManager::new(),
        config,
    };

    let app = build_router(state);

    tracing::info!("Conquer server starting on {}", bind_addr);

    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("Failed to bind");

    // Graceful shutdown (T277)
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");

    tracing::info!("Server shut down gracefully");
}

async fn shutdown_signal() {
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

    tracing::info!("Shutdown signal received");
}
