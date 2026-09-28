use std::sync::Arc;

use uuid::Uuid;
use xetaravel_kernel::{AppResult, Clock, Principal};

use super::{article_not_found, build_article_draft, load_article, slug_taken};
use crate::application::dto::{ArticleDto, UpsertArticleRequest};
use crate::application::ports::AuthorDirectory;
use crate::domain::{ArticleId, ArticleRepository, CategoryRepository};

/// Edits an existing article.
pub struct UpdateArticle {
    articles: Arc<dyn ArticleRepository>,
    categories: Arc<dyn CategoryRepository>,
    authors: Arc<dyn AuthorDirectory>,
    clock: Arc<dyn Clock>,
}

impl UpdateArticle {
    /// Builds the use case with its dependencies.
    pub fn new(
        articles: Arc<dyn ArticleRepository>,
        categories: Arc<dyn CategoryRepository>,
        authors: Arc<dyn AuthorDirectory>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            articles,
            categories,
            authors,
            clock,
        }
    }

    /// Applies the form to the article, keeping its slug unique.
    pub async fn execute(
        &self,
        principal: Principal,
        id: Uuid,
        input: UpsertArticleRequest,
    ) -> AppResult<ArticleDto> {
        principal.require_admin()?;
        let mut article = self
            .articles
            .find_by_id(ArticleId::from(id))
            .await?
            .ok_or_else(article_not_found)?;

        let draft = build_article_draft(self.categories.as_ref(), input).await?;
        article.revise(draft, self.clock.now())?;

        if self
            .articles
            .slug_exists(&article.slug, Some(article.id))
            .await?
        {
            return Err(slug_taken());
        }
        self.articles.update(&article).await?;

        load_article(self.articles.as_ref(), self.authors.as_ref(), article.id).await
    }
}

#[cfg(test)]
mod tests {
    use xetaravel_kernel::AppError;

    use super::*;
    use crate::application::ports::MockAuthorDirectory;
    use crate::application::test_support::{
        admin_principal, authors, categorized, category, clock,
    };
    use crate::domain::{MockArticleRepository, MockCategoryRepository};

    /// Returns an article form renaming the article.
    fn request() -> UpsertArticleRequest {
        UpsertArticleRequest {
            category_id: Uuid::now_v7(),
            title: "Renamed".into(),
            slug: Some("renamed".into()),
            excerpt: None,
            content: "New body".into(),
            publish: false,
            comments_enabled: false,
        }
    }

    /// Returns a category repository mock where every category exists.
    fn categories() -> MockCategoryRepository {
        let mut categories = MockCategoryRepository::new();
        categories
            .expect_find_by_id()
            .returning(|_| Ok(Some(category("Rust"))));
        categories
    }

    /// Builds the use case under test.
    fn use_case(articles: MockArticleRepository, authors: MockAuthorDirectory) -> UpdateArticle {
        UpdateArticle::new(
            Arc::new(articles),
            Arc::new(categories()),
            Arc::new(authors),
            clock(),
        )
    }

    #[tokio::test]
    async fn updates_the_article() {
        let existing = categorized(true).article;
        let id = existing.id;
        let mut articles = MockArticleRepository::new();
        articles
            .expect_find_by_id()
            .returning(move |_| Ok(Some(existing.clone())));
        articles
            .expect_slug_exists()
            .withf(move |slug, excluding| slug.as_str() == "renamed" && *excluding == Some(id))
            .returning(|_, _| Ok(false));
        articles
            .expect_update()
            .withf(|a| a.title == "Renamed" && !a.is_published() && !a.comments_enabled)
            .times(1)
            .returning(|_| Ok(()));
        articles
            .expect_find_categorized_by_id()
            .returning(|_| Ok(Some(categorized(false))));

        use_case(articles, authors())
            .execute(admin_principal(), id.as_uuid(), request())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn missing_article_is_not_found() {
        let mut articles = MockArticleRepository::new();
        articles.expect_find_by_id().returning(|_| Ok(None));

        let error = use_case(articles, MockAuthorDirectory::new())
            .execute(admin_principal(), Uuid::now_v7(), request())
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn rejects_slug_of_another_article() {
        let existing = categorized(true).article;
        let mut articles = MockArticleRepository::new();
        articles
            .expect_find_by_id()
            .returning(move |_| Ok(Some(existing.clone())));
        articles.expect_slug_exists().returning(|_, _| Ok(true));

        let error = use_case(articles, MockAuthorDirectory::new())
            .execute(admin_principal(), Uuid::now_v7(), request())
            .await
            .unwrap_err();

        assert_eq!(error, AppError::field("slug", "is already taken"));
    }
}
