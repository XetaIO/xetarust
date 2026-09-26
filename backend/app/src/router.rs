//! HTTP entry point: merges the routers of every context.

use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::state::AppState;

/// Builds the full API router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .merge(xetaravel_identity::router())
        .merge(xetaravel_publishing::router())
        .merge(xetaravel_discussion::router())
        .with_state(state)
}

/// Liveness probe.
async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
