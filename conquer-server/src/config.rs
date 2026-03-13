use std::net::SocketAddr;

/// Server configuration (T438 — environment variable driven)
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Address to bind
    pub bind_addr: SocketAddr,
    /// JWT secret key
    pub jwt_secret: String,
    /// JWT token expiry in hours
    pub jwt_expiry_hours: u64,
    /// CORS allowed origins
    pub cors_origins: Vec<String>,
    /// Static files directory (for frontend assets)
    pub static_dir: Option<String>,
    /// WebSocket heartbeat interval in seconds
    pub ws_heartbeat_secs: u64,
    /// WebSocket timeout in seconds
    pub ws_timeout_secs: u64,
    /// Database URL for Postgres
    pub database_url: Option<String>,
    /// Log level (RUST_LOG)
    pub log_level: String,
    /// Rate limit: max requests per window per IP
    pub rate_limit_max: u64,
    /// Rate limit window in seconds
    pub rate_limit_window_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            bind_addr: "0.0.0.0:3000".parse().unwrap(),
            jwt_secret: "conquer-dev-secret-change-in-production".to_string(),
            jwt_expiry_hours: 24,
            cors_origins: vec!["http://localhost:5173".to_string()],
            static_dir: None,
            ws_heartbeat_secs: 30,
            ws_timeout_secs: 60,
            database_url: None,
            log_level: "info,conquer_server=debug".to_string(),
            rate_limit_max: 100,
            rate_limit_window_secs: 60,
        }
    }
}

impl ServerConfig {
    /// Build configuration from environment variables (T438)
    pub fn from_env() -> Self {
        let port: u16 = std::env::var("PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3000);

        let bind_addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();

        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "conquer-dev-secret-change-in-production".to_string());

        let jwt_expiry_hours: u64 = std::env::var("JWT_EXPIRY_HOURS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(24);

        let cors_origins: Vec<String> = std::env::var("CORS_ORIGIN")
            .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|_| vec!["http://localhost:5173".to_string()]);

        let static_dir = std::env::var("STATIC_DIR").ok().or_else(|| {
            // Default: look for web/dist relative to binary
            let candidates = ["./web/dist", "./dist", "/app/dist"];
            candidates
                .iter()
                .find(|p| std::path::Path::new(p).is_dir())
                .map(|p| p.to_string())
        });

        let database_url = std::env::var("DATABASE_URL").ok();

        let log_level =
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,conquer_server=debug".to_string());

        let rate_limit_max: u64 = std::env::var("RATE_LIMIT_MAX")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);

        let rate_limit_window_secs: u64 = std::env::var("RATE_LIMIT_WINDOW_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        ServerConfig {
            bind_addr,
            jwt_secret,
            jwt_expiry_hours,
            cors_origins,
            static_dir,
            ws_heartbeat_secs: 30,
            ws_timeout_secs: 60,
            database_url,
            log_level,
            rate_limit_max,
            rate_limit_window_secs,
        }
    }
}
