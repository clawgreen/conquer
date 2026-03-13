// conquer-server: Axum HTTP/WebSocket server (Phase 3 + Phase 7 production)
//
// Modules:
// - config: Server configuration (env var driven)
// - jwt: JWT token management
// - routes: All HTTP route handlers
// - ws: WebSocket connection manager and message protocol
// - errors: Error types and HTTP error responses
// - app: Application state and router construction
// - metrics: Server metrics collection (T453)

pub mod app;
pub mod config;
pub mod errors;
pub mod jwt;
pub mod metrics;
pub mod routes;
pub mod ws;
