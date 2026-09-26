//! HTTP adapter of the Discussion context: comments of the blog articles.
//! Handlers stay thin: extract, call one use case, serialize.

use std::sync::Arc;

use axum::extract::{FromRef, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use axum::{Json, Router};
use uuid::Uuid;
use xetaravel_kernel::PrincipalResolver;
use xetaravel_kernel::http::{ApiResult, CurrentPrincipal, JsonBody, PathParam};

use crate::DiscussionModule;
use crate::application::dto::{CommentDto, CreateCommentRequest};

/// Routes of the Discussion context. The router state must expose the
/// [`DiscussionModule`] and the principal resolver through [`FromRef`].
pub fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Arc<DiscussionModule>: FromRef<S>,
    Arc<dyn PrincipalResolver>: FromRef<S>,
{
    Router::new()
        .route(
            "/api/articles/{slug}/comments",
            get(list_comments).post(post_comment),
        )
        .route("/api/comments/{id}", delete(delete_comment))
}

/// `GET /api/articles/{slug}/comments` — comments of a published article.
async fn list_comments(
    State(discussion): State<Arc<DiscussionModule>>,
    PathParam(slug): PathParam<String>,
) -> ApiResult<Json<Vec<CommentDto>>> {
    Ok(Json(discussion.list_comments.execute(&slug).await?))
}

/// `POST /api/articles/{slug}/comments` — comments as the current user.
async fn post_comment(
    State(discussion): State<Arc<DiscussionModule>>,
    CurrentPrincipal(principal): CurrentPrincipal,
    PathParam(slug): PathParam<String>,
    JsonBody(input): JsonBody<CreateCommentRequest>,
) -> ApiResult<(StatusCode, Json<CommentDto>)> {
    let comment = discussion
        .post_comment
        .execute(principal, &slug, input)
        .await?;
    Ok((StatusCode::CREATED, Json(comment)))
}

/// `DELETE /api/comments/{id}` — author or admin only.
async fn delete_comment(
    State(discussion): State<Arc<DiscussionModule>>,
    CurrentPrincipal(principal): CurrentPrincipal,
    PathParam(id): PathParam<Uuid>,
) -> ApiResult<StatusCode> {
    discussion.delete_comment.execute(principal, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
