//! Use cases of the Publishing context. Each use case is a struct holding its
//! dependencies as `Arc<dyn Port>` and exposing a single `execute` method.
//! Administration use cases start with `principal.require_admin()`.

mod create_article;
mod create_category;
mod delete_article;
mod delete_category;
mod find_published_article;
mod get_article;
mod get_cover;
mod get_published_article;
mod list_articles;
mod list_categories;
mod list_published_articles;
mod remove_cover;
mod update_article;
mod update_category;
mod upload_cover;

pub use create_article::CreateArticle;
pub use create_category::CreateCategory;
pub use delete_article::DeleteArticle;
pub use delete_category::DeleteCategory;
pub use find_published_article::FindPublishedArticle;
pub use get_article::GetArticle;
pub use get_cover::GetCover;
pub use get_published_article::GetPublishedArticle;
pub use list_articles::ListArticles;
pub use list_categories::ListCategories;
pub use list_published_articles::ListPublishedArticles;
pub use remove_cover::RemoveCover;
pub use update_article::UpdateArticle;
pub use update_category::UpdateCategory;
pub use upload_cover::UploadCover;

use validator::Validate;
use xetaravel_kernel::text::non_blank;
use xetaravel_kernel::{AppError, AppResult};

use crate::application::dto::{ArticleDto, UpsertArticleRequest};
use crate::application::ports::{AuthorDirectory, CoverStorage};
use crate::application::views::with_author;
use crate::domain::{
    ArticleDraft, ArticleId, ArticleRepository, CategorizedArticle, CategoryId, CategoryRepository,
    CoverImage, Slug,
};

/// Parses an optional slug typed by a user; blank values mean "no slug".
fn parse_optional_slug(raw: Option<&str>) -> AppResult<Option<Slug>> {
    Ok(non_blank(raw).map(Slug::parse).transpose()?)
}

/// Validates an article form and turns it into a domain draft,
/// checking that the chosen category exists.
async fn build_article_draft(
    categories: &dyn CategoryRepository,
    input: UpsertArticleRequest,
) -> AppResult<ArticleDraft> {
    input.validate()?;
    let category_id = CategoryId::from(input.category_id);
    if categories.find_by_id(category_id).await?.is_none() {
        return Err(AppError::field("category_id", "does not exist"));
    }

    Ok(ArticleDraft {
        category_id,
        slug: parse_optional_slug(input.slug.as_deref())?,
        title: input.title,
        excerpt: input.excerpt,
        content: input.content,
        publish: input.publish,
        comments_enabled: input.comments_enabled,
    })
}

/// Loads the full view of an article (drafts included) as a DTO.
async fn load_article(
    articles: &dyn ArticleRepository,
    authors: &dyn AuthorDirectory,
    id: ArticleId,
) -> AppResult<ArticleDto> {
    let entry = articles
        .find_categorized_by_id(id)
        .await?
        .ok_or_else(article_not_found)?;
    Ok(ArticleDto::from(&with_author(authors, entry).await?))
}

/// Loads a publicly visible article by slug. Drafts and malformed slugs are
/// reported as `None` so nothing is leaked.
async fn find_published(
    articles: &dyn ArticleRepository,
    slug: &str,
) -> AppResult<Option<CategorizedArticle>> {
    let Ok(slug) = Slug::parse(slug) else {
        return Ok(None);
    };
    Ok(articles
        .find_categorized_by_slug(&slug)
        .await?
        .filter(|entry| entry.article.is_published()))
}

/// Deletes the file of a cover that is no longer referenced. Best effort: a
/// failure only leaves an orphan file behind, so it is logged, not returned.
async fn discard_cover(covers: &dyn CoverStorage, cover: &CoverImage) {
    if let Err(error) = covers.delete(cover).await {
        tracing::warn!(%cover, %error, "could not delete a cover image file");
    }
}

/// Error returned when an article does not exist (or must stay hidden).
fn article_not_found() -> AppError {
    AppError::NotFound("article not found".into())
}

/// Error returned when a slug is already used by another record.
fn slug_taken() -> AppError {
    AppError::field("slug", "is already taken")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_optional_slug_handles_blank_and_invalid_values() {
        assert_eq!(parse_optional_slug(None).unwrap(), None);
        assert_eq!(parse_optional_slug(Some("  ")).unwrap(), None);
        assert_eq!(
            parse_optional_slug(Some("rust")).unwrap(),
            Some(Slug::parse("rust").unwrap())
        );
        assert!(parse_optional_slug(Some("Not A Slug")).is_err());
    }
}
