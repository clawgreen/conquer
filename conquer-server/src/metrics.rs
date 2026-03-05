// conquer-server/src/metrics.rs — Server metrics (T453)
//
// Tracks active games, connected players, request counts, and uptime.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use serde::Serialize;

/// Server metrics collector (T453)
pub struct Metrics {
    start_time: Instant,
    pub total_requests: AtomicU64,
    pub active_connections: AtomicU64,
    pub ws_messages_sent: AtomicU64,
    pub ws_messages_received: AtomicU64,
    pub actions_processed: AtomicU64,
    pub turns_advanced: AtomicU64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshot {
    pub uptime_secs: u64,
    pub total_requests: u64,
    pub active_connections: u64,
    pub ws_messages_sent: u64,
    pub ws_messages_received: u64,
    pub actions_processed: u64,
    pub turns_advanced: u64,
    pub requests_per_minute: f64,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            start_time: Instant::now(),
            total_requests: AtomicU64::new(0),
            active_connections: AtomicU64::new(0),
            ws_messages_sent: AtomicU64::new(0),
            ws_messages_received: AtomicU64::new(0),
            actions_processed: AtomicU64::new(0),
            turns_advanced: AtomicU64::new(0),
        }
    }

    pub fn record_request(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn connection_opened(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn connection_closed(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let uptime_secs = self.start_time.elapsed().as_secs();
        let total_requests = self.total_requests.load(Ordering::Relaxed);
        let rpm = if uptime_secs > 0 {
            (total_requests as f64 / uptime_secs as f64) * 60.0
        } else {
            0.0
        };

        MetricsSnapshot {
            uptime_secs,
            total_requests,
            active_connections: self.active_connections.load(Ordering::Relaxed),
            ws_messages_sent: self.ws_messages_sent.load(Ordering::Relaxed),
            ws_messages_received: self.ws_messages_received.load(Ordering::Relaxed),
            actions_processed: self.actions_processed.load(Ordering::Relaxed),
            turns_advanced: self.turns_advanced.load(Ordering::Relaxed),
            requests_per_minute: rpm,
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}
