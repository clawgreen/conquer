use axum::Json;
use chrono::Utc;
use serde_json::{json, Value};

/// GET /api/health — health check (T281)
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": Utc::now().to_rfc3339(),
    }))
}
