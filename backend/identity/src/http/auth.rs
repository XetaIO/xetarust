use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use xetaravel_kernel::http::{ApiResult, CurrentPrincipal, JsonBody};

use crate::IdentityModule;
use crate::application::dto::{AuthResponse, LoginRequest, RegisterRequest, UserDto};

/// `POST /api/auth/register` — creates a member account and returns a token.
pub async fn register(
    State(identity): State<Arc<IdentityModule>>,
    JsonBody(input): JsonBody<RegisterRequest>,
) -> ApiResult<(StatusCode, Json<AuthResponse>)> {
    let response = identity.register.execute(input).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

/// `POST /api/auth/login` — exchanges credentials for a token.
pub async fn login(
    State(identity): State<Arc<IdentityModule>>,
    JsonBody(input): JsonBody<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    Ok(Json(identity.login.execute(input).await?))
}

/// `GET /api/auth/me` — returns the authenticated user.
pub async fn me(
    State(identity): State<Arc<IdentityModule>>,
    CurrentPrincipal(principal): CurrentPrincipal,
) -> ApiResult<Json<UserDto>> {
    Ok(Json(identity.current_user.execute(principal).await?))
}
