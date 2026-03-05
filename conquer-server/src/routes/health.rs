use axum::Json;
use serde_json::{json, Value};
use chrono::Utc;

/// GET /api/health — health check (T281)
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": Utc::now().to_rfc3339(),
    }))
}
