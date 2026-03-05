// conquer-server: Axum HTTP/WebSocket server (Phase 3)
//
// Modules:
// - config: Server configuration
// - jwt: JWT token management
// - routes: All HTTP route handlers
// - ws: WebSocket connection manager and message protocol
// - errors: Error types and HTTP error responses
// - middleware: Request ID, auth extraction
// - app: Application state and router construction

pub mod config;
pub mod jwt;
pub mod routes;
pub mod ws;
pub mod errors;
pub mod app;
