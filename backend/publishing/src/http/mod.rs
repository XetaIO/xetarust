//! HTTP adapter of the Publishing context: public blog and content
//! administration. Handlers stay thin: extract, call one use case, serialize.

mod admin;
mod public;

use std::sync::Arc;

use axum::Router;
use axum::extract::{DefaultBodyLimit, FromRef};
use axum::routing::{get, post, put};
use xetaravel_kernel::PrincipalResolver;

use crate::domain::COVER_MAX_BYTES;

use crate::PublishingModule;

/// Extra bytes accepted over [`COVER_MAX_BYTES`] by the upload route.
const COVER_BODY_MARGIN: usize = 64 * 1024;

/// Routes of the Publishing context. The router state must expose the
/// [`PublishingModule`] and the principal resolver through [`FromRef`].
pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Arc<PublishingModule>: FromRef<S>,
    Arc<dyn PrincipalResolver>: FromRef<S>,
{
    Router::new()
        .route("/api/articles", get(public::list_articles))
        .route("/api/articles/{slug}", get(public::get_article))
        .route("/api/categories", get(public::list_categories))
        .route("/api/covers/{name}", get(public::get_cover))
        .route(
            "/api/admin/articles",
            get(admin::list_articles).post(admin::create_article),
        )
        .route(
            "/api/admin/articles/{id}",
            get(admin::get_article)
                .put(admin::update_article)
                .delete(admin::delete_article),
        )
        .route(
            "/api/admin/articles/{id}/cover",
            put(admin::upload_cover)
                .delete(admin::remove_cover)
                // The domain rejects files above the limit with a clear
                // message; the margin lets such files reach it.
                .layer(DefaultBodyLimit::max(COVER_MAX_BYTES + COVER_BODY_MARGIN)),
        )
        .route("/api/admin/categories", post(admin::create_category))
        .route(
            "/api/admin/categories/{id}",
            put(admin::update_category).delete(admin::delete_category),
        )
}
