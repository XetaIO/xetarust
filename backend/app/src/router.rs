//! HTTP entry point: merges the routers of every context.

use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_governor::{GovernorError, GovernorLayer};
use xetaravel_kernel::AppError;
use xetaravel_kernel::http::ApiError;

use crate::config::RateLimitSettings;
use crate::state::AppState;

/// Builds the full API router. The credential routes of Identity (login,
/// register) and the resume download are rate limited per client IP; the
/// other routes are not.
pub fn router(state: AppState) -> Router {
    let auth = rate_limited(xetaravel_identity::auth_router(), state.auth_rate_limit);
    let resume = rate_limited(xetaravel_resume::router(), state.auth_rate_limit);

    Router::new()
        .route("/api/health", get(health))
        .merge(auth)
        .merge(xetaravel_identity::account_router())
        .merge(xetaravel_publishing::router())
        .merge(xetaravel_discussion::router())
        .merge(resume)
        .with_state(state)
}

/// Wraps `routes` in a per-IP rate limiter. The client IP is read from
/// `X-Forwarded-For` (set by the Next.js server), then from the peer address.
fn rate_limited(routes: Router<AppState>, settings: RateLimitSettings) -> Router<AppState> {
    let config = GovernorConfigBuilder::default()
        .per_second(settings.period_seconds)
        .burst_size(settings.burst)
        .key_extractor(SmartIpKeyExtractor)
        .finish()
        .expect("the rate limit settings are validated as positive by the config");

    routes.layer(GovernorLayer::new(config).error_handler(rate_limit_error))
}

/// Converts a rate limiter failure into the project's JSON error format.
fn rate_limit_error(error: GovernorError) -> Response {
    let error = match error {
        GovernorError::TooManyRequests { .. } => {
            AppError::TooManyRequests("too many attempts, try again later".into())
        }
        other => AppError::Internal(format!("rate limiter failure: {other}")),
    };
    ApiError(error).into_response()
}

/// Liveness probe.
async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
