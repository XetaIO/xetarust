//! Data Transfer Objects of the Publishing context (public API contract).
//!
//! TypeScript bindings are generated in `frontend/src/types/api/publishing/`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;
use validator::Validate;
use xetaravel_kernel::dto::AuthorDto;

use crate::application::views::{ArticleView, AuthorSummary};
use crate::domain::Category;

/// A blog category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "publishing/")]
pub struct CategoryDto {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

impl From<&Category> for CategoryDto {
    /// Builds the DTO from a domain category.
    fn from(category: &Category) -> Self {
        Self {
            id: category.id.as_uuid(),
            name: category.name.clone(),
            slug: category.slug.to_string(),
            description: category.description.clone(),
        }
    }
}

/// Body used to create or update a category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate, TS)]
#[ts(export, export_to = "publishing/")]
pub struct UpsertCategoryRequest {
    #[validate(length(
        min = 2,
        max = 100,
        message = "must contain between 2 and 100 characters"
    ))]
    pub name: String,
    /// Derived from the name when empty.
    #[serde(default)]
    #[ts(optional = nullable)]
    #[validate(length(max = 200, message = "must contain at most 200 characters"))]
    pub slug: Option<String>,
    #[serde(default)]
    #[ts(optional = nullable)]
    #[validate(length(max = 500, message = "must contain at most 500 characters"))]
    pub description: Option<String>,
}

impl From<&AuthorSummary> for AuthorDto {
    /// Builds the shared author DTO from an article author.
    fn from(author: &AuthorSummary) -> Self {
        Self {
            id: author.id.as_uuid(),
            username: author.username.clone(),
        }
    }
}

/// An article without its body, used in listings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "publishing/")]
pub struct ArticleSummaryDto {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub excerpt: Option<String>,
    /// File name of the cover image, served by `GET /api/covers/{name}`.
    pub cover_image: Option<String>,
    pub author: AuthorDto,
    pub category: CategoryDto,
    pub reading_time_minutes: u32,
    pub is_published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&ArticleView> for ArticleSummaryDto {
    /// Builds the summary from an article view.
    fn from(view: &ArticleView) -> Self {
        let article = &view.article;
        Self {
            id: article.id.as_uuid(),
            title: article.title.clone(),
            slug: article.slug.to_string(),
            excerpt: article.excerpt.clone(),
            cover_image: article.cover.as_ref().map(ToString::to_string),
            author: (&view.author).into(),
            category: (&view.category).into(),
            reading_time_minutes: article.reading_time_minutes(),
            is_published: article.is_published(),
            published_at: article.published_at,
            created_at: article.created_at,
            updated_at: article.updated_at,
        }
    }
}

/// A full article, including its Markdown body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "publishing/")]
pub struct ArticleDto {
    #[serde(flatten)]
    pub summary: ArticleSummaryDto,
    /// Markdown body.
    pub content: String,
}

impl From<&ArticleView> for ArticleDto {
    /// Builds the full DTO from an article view.
    fn from(view: &ArticleView) -> Self {
        Self {
            summary: view.into(),
            content: view.article.content.clone(),
        }
    }
}

/// Body used to create or update an article.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate, TS)]
#[ts(export, export_to = "publishing/")]
pub struct UpsertArticleRequest {
    pub category_id: Uuid,
    #[validate(length(
        min = 3,
        max = 200,
        message = "must contain between 3 and 200 characters"
    ))]
    pub title: String,
    /// Derived from the title on creation when empty; kept as-is on update when empty.
    #[serde(default)]
    #[ts(optional = nullable)]
    #[validate(length(max = 200, message = "must contain at most 200 characters"))]
    pub slug: Option<String>,
    #[serde(default)]
    #[ts(optional = nullable)]
    #[validate(length(max = 500, message = "must contain at most 500 characters"))]
    pub excerpt: Option<String>,
    /// Markdown body.
    #[validate(length(min = 1, message = "is required"))]
    pub content: String,
    /// `true` to make the article public.
    #[serde(default)]
    pub publish: bool,
}

/// Query string of the public article listing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = "publishing/", optional_fields = nullable)]
pub struct ArticlesQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    /// Category slug to filter on.
    pub category: Option<String>,
}
