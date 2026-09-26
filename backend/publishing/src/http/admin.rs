use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use uuid::Uuid;
use xetaravel_kernel::dto::{PageQuery, Paginated};
use xetaravel_kernel::http::{AdminPrincipal, ApiResult, JsonBody, PathParam, QueryParams};

use crate::PublishingModule;
use crate::application::dto::{
    ArticleDto, ArticleSummaryDto, CategoryDto, UpsertArticleRequest, UpsertCategoryRequest,
};

/// `GET /api/admin/articles` — every article, drafts included.
pub async fn list_articles(
    State(publishing): State<Arc<PublishingModule>>,
    AdminPrincipal(principal): AdminPrincipal,
    QueryParams(query): QueryParams<PageQuery>,
) -> ApiResult<Json<Paginated<ArticleSummaryDto>>> {
    Ok(Json(
        publishing.list_articles.execute(principal, query).await?,
    ))
}

/// `GET /api/admin/articles/{id}` — one article for edition.
pub async fn get_article(
    State(publishing): State<Arc<PublishingModule>>,
    AdminPrincipal(principal): AdminPrincipal,
    PathParam(id): PathParam<Uuid>,
) -> ApiResult<Json<ArticleDto>> {
    Ok(Json(publishing.get_article.execute(principal, id).await?))
}

/// `POST /api/admin/articles` — writes an article.
pub async fn create_article(
    State(publishing): State<Arc<PublishingModule>>,
    AdminPrincipal(principal): AdminPrincipal,
    JsonBody(input): JsonBody<UpsertArticleRequest>,
) -> ApiResult<(StatusCode, Json<ArticleDto>)> {
    let article = publishing.create_article.execute(principal, input).await?;
    Ok((StatusCode::CREATED, Json(article)))
}

/// `PUT /api/admin/articles/{id}` — edits an article.
pub async fn update_article(
    State(publishing): State<Arc<PublishingModule>>,
    AdminPrincipal(principal): AdminPrincipal,
    PathParam(id): PathParam<Uuid>,
    JsonBody(input): JsonBody<UpsertArticleRequest>,
) -> ApiResult<Json<ArticleDto>> {
    Ok(Json(
        publishing
            .update_article
            .execute(principal, id, input)
            .await?,
    ))
}

/// `DELETE /api/admin/articles/{id}` — deletes an article and its comments.
pub async fn delete_article(
    State(publishing): State<Arc<PublishingModule>>,
    AdminPrincipal(principal): AdminPrincipal,
    PathParam(id): PathParam<Uuid>,
) -> ApiResult<StatusCode> {
    publishing.delete_article.execute(principal, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/admin/categories` — creates a category.
pub async fn create_category(
    State(publishing): State<Arc<PublishingModule>>,
    AdminPrincipal(principal): AdminPrincipal,
    JsonBody(input): JsonBody<UpsertCategoryRequest>,
) -> ApiResult<(StatusCode, Json<CategoryDto>)> {
    let category = publishing.create_category.execute(principal, input).await?;
    Ok((StatusCode::CREATED, Json(category)))
}

/// `PUT /api/admin/categories/{id}` — edits a category.
pub async fn update_category(
    State(publishing): State<Arc<PublishingModule>>,
    AdminPrincipal(principal): AdminPrincipal,
    PathParam(id): PathParam<Uuid>,
    JsonBody(input): JsonBody<UpsertCategoryRequest>,
) -> ApiResult<Json<CategoryDto>> {
    Ok(Json(
        publishing
            .update_category
            .execute(principal, id, input)
            .await?,
    ))
}

/// `DELETE /api/admin/categories/{id}` — deletes an empty category.
pub async fn delete_category(
    State(publishing): State<Arc<PublishingModule>>,
    AdminPrincipal(principal): AdminPrincipal,
    PathParam(id): PathParam<Uuid>,
) -> ApiResult<StatusCode> {
    publishing.delete_category.execute(principal, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
