//! Conversions between SeaORM rows and the Publishing aggregates.
//!
//! Loading a row re-runs the value object validation; a failure means the
//! database holds corrupted data and is reported as a repository error.

use sea_orm::ActiveValue::Set;
use xetaravel_kernel::DomainResult;
use xetaravel_kernel::persistence::corrupted;

use super::entities::{article, category};
use crate::domain::{Article, Category, CoverImage, Slug};

/// Converts a `categories` row into a domain category.
pub(super) fn to_category(model: category::Model) -> DomainResult<Category> {
    Ok(Category {
        id: model.id.into(),
        name: model.name,
        slug: Slug::parse(&model.slug).map_err(|e| corrupted("categories", e))?,
        description: model.description,
        created_at: model.created_at,
        updated_at: model.updated_at,
    })
}

/// Converts a domain category into a fully set active model.
pub(super) fn from_category(category: &Category) -> category::ActiveModel {
    category::ActiveModel {
        id: Set(category.id.as_uuid()),
        name: Set(category.name.clone()),
        slug: Set(category.slug.to_string()),
        description: Set(category.description.clone()),
        created_at: Set(category.created_at),
        updated_at: Set(category.updated_at),
    }
}

/// Converts an `articles` row into a domain article.
pub(super) fn to_article(model: article::Model) -> DomainResult<Article> {
    Ok(Article {
        id: model.id.into(),
        author_id: model.author_id.into(),
        category_id: model.category_id.into(),
        title: model.title,
        slug: Slug::parse(&model.slug).map_err(|e| corrupted("articles", e))?,
        excerpt: model.excerpt,
        content: model.content,
        cover: model
            .cover_image
            .as_deref()
            .map(CoverImage::parse)
            .transpose()
            .map_err(|e| corrupted("articles", e))?,
        published_at: model.published_at,
        comments_enabled: model.comments_enabled,
        created_at: model.created_at,
        updated_at: model.updated_at,
    })
}

/// Converts a domain article into a fully set active model.
pub(super) fn from_article(article: &Article) -> article::ActiveModel {
    article::ActiveModel {
        id: Set(article.id.as_uuid()),
        author_id: Set(article.author_id.as_uuid()),
        category_id: Set(article.category_id.as_uuid()),
        title: Set(article.title.clone()),
        slug: Set(article.slug.to_string()),
        excerpt: Set(article.excerpt.clone()),
        content: Set(article.content.clone()),
        cover_image: Set(article.cover.as_ref().map(|cover| cover.to_string())),
        published_at: Set(article.published_at),
        comments_enabled: Set(article.comments_enabled),
        created_at: Set(article.created_at),
        updated_at: Set(article.updated_at),
    }
}
