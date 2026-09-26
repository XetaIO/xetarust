//! HTTP adapter of the Identity context: authentication and user administration.
//! Handlers stay thin: extract, call one use case, serialize.

mod admin;
mod auth;

use std::sync::Arc;

use axum::Router;
use axum::extract::FromRef;
use axum::routing::{get, patch, post};
use xetaravel_kernel::PrincipalResolver;

use crate::IdentityModule;

/// Routes of the Identity context. The router state must expose the
/// [`IdentityModule`] and the principal resolver through [`FromRef`].
pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Arc<IdentityModule>: FromRef<S>,
    Arc<dyn PrincipalResolver>: FromRef<S>,
{
    Router::new()
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/me", get(auth::me))
        .route("/api/admin/users", get(admin::list_users))
        .route("/api/admin/users/{id}/role", patch(admin::change_user_role))
}
