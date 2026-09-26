use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::header;
use axum::response::IntoResponse;
use xetaravel_kernel::dto::Paginated;
use xetaravel_kernel::http::{ApiResult, PathParam, QueryParams};

use crate::PublishingModule;
use crate::application::dto::{ArticleDto, ArticleSummaryDto, ArticlesQuery, CategoryDto};

/// `GET /api/articles?page&per_page&category` — published articles.
pub async fn list_articles(
    State(publishing): State<Arc<PublishingModule>>,
    QueryParams(query): QueryParams<ArticlesQuery>,
) -> ApiResult<Json<Paginated<ArticleSummaryDto>>> {
    Ok(Json(
        publishing.list_published_articles.execute(query).await?,
    ))
}

/// `GET /api/articles/{slug}` — one published article.
pub async fn get_article(
    State(publishing): State<Arc<PublishingModule>>,
    PathParam(slug): PathParam<String>,
) -> ApiResult<Json<ArticleDto>> {
    Ok(Json(publishing.get_published_article.execute(&slug).await?))
}

/// `GET /api/categories` — every category.
pub async fn list_categories(
    State(publishing): State<Arc<PublishingModule>>,
) -> ApiResult<Json<Vec<CategoryDto>>> {
    Ok(Json(publishing.list_categories.execute().await?))
}

/// `GET /api/covers/{name}` — the bytes of a cover image. Names are unique
/// per upload, so the response can be cached forever.
pub async fn get_cover(
    State(publishing): State<Arc<PublishingModule>>,
    PathParam(name): PathParam<String>,
) -> ApiResult<impl IntoResponse> {
    let (bytes, mime_type) = publishing.get_cover.execute(&name).await?;
    Ok((
        [
            (header::CONTENT_TYPE, mime_type),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        bytes,
    ))
}
