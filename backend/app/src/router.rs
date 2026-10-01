//! HTTP entry point: merges the routers of every context.

use std::time::Duration;

use axum::http::{HeaderValue, Method, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::SmartIpKeyExtractor;
use tower_governor::{GovernorError, GovernorLayer};
use tower_http::cors::{AllowOrigin, CorsLayer};
use xetaravel_kernel::AppError;
use xetaravel_kernel::http::ApiError;

use crate::config::RateLimitSettings;
use crate::state::AppState;

/// How long a browser may cache a CORS preflight answer.
const CORS_MAX_AGE: Duration = Duration::from_secs(10 * 60);

/// Builds the full API router. The credential routes of Identity (login,
/// register) and the resume download are rate limited per client IP; the
/// other routes are not. Every route answers CORS for the configured origin.
pub fn router(state: AppState) -> Router {
    let cors = cors(state.cors_origin.clone());
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
        .layer(cors)
}

/// CORS policy: only the exact `origin` of the Next.js site may read the
/// API from a browser, without credentials (the JWT travels in the
/// `Authorization` header, never in a cookie of the API).
///
/// A one-item list rather than an exact origin: the header is then echoed
/// only to the matching `Origin` and the responses vary on `Origin` (safe
/// behind a cache).
fn cors(origin: HeaderValue) -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::list([origin]))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .max_age(CORS_MAX_AGE)
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
