use std::net::SocketAddr;

/// Server configuration
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
        }
    }
}
