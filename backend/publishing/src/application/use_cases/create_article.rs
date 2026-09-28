use std::sync::Arc;

use xetaravel_kernel::{AppResult, Clock, Principal};

use super::{build_article_draft, load_article, slug_taken};
use crate::application::dto::{ArticleDto, UpsertArticleRequest};
use crate::application::ports::AuthorDirectory;
use crate::domain::{Article, ArticleRepository, AuthorId, CategoryRepository};

/// Writes a new article.
pub struct CreateArticle {
    articles: Arc<dyn ArticleRepository>,
    categories: Arc<dyn CategoryRepository>,
    authors: Arc<dyn AuthorDirectory>,
    clock: Arc<dyn Clock>,
}

impl CreateArticle {
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

    /// Validates the form, ensures the slug is unique and stores the article
    /// on behalf of `principal`, who becomes its author.
    pub async fn execute(
        &self,
        principal: Principal,
        input: UpsertArticleRequest,
    ) -> AppResult<ArticleDto> {
        principal.require_admin()?;
        let draft = build_article_draft(self.categories.as_ref(), input).await?;
        let author = AuthorId::from(principal.user_id);
        let article = Article::write(author, draft, self.clock.now())?;

        if self.articles.slug_exists(&article.slug, None).await? {
            return Err(slug_taken());
        }
        self.articles.create(&article).await?;

        load_article(self.articles.as_ref(), self.authors.as_ref(), article.id).await
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use xetaravel_kernel::AppError;

    use super::*;
    use crate::application::ports::MockAuthorDirectory;
    use crate::application::test_support::{
        admin_principal, authors, categorized, category, clock, member_principal,
    };
    use crate::domain::{MockArticleRepository, MockCategoryRepository};

    /// Returns a valid article form.
    fn request() -> UpsertArticleRequest {
        UpsertArticleRequest {
            category_id: Uuid::now_v7(),
            title: "Hello Rust".into(),
            slug: None,
            excerpt: None,
            content: "Body".into(),
            publish: true,
            comments_enabled: false,
        }
    }

    /// Returns a category repository mock where the category exists (or not).
    fn categories(exists: bool) -> MockCategoryRepository {
        let mut categories = MockCategoryRepository::new();
        categories
            .expect_find_by_id()
            .returning(move |_| Ok(exists.then(|| category("Rust"))));
        categories
    }

    /// Builds the use case under test.
    fn use_case(
        articles: MockArticleRepository,
        categories: MockCategoryRepository,
        authors: MockAuthorDirectory,
    ) -> CreateArticle {
        CreateArticle::new(
            Arc::new(articles),
            Arc::new(categories),
            Arc::new(authors),
            clock(),
        )
    }

    #[tokio::test]
    async fn creates_a_published_article() {
        let admin = admin_principal();
        let mut articles = MockArticleRepository::new();
        articles.expect_slug_exists().returning(|_, _| Ok(false));
        articles
            .expect_create()
            .withf(move |a| {
                a.author_id.as_uuid() == admin.user_id
                    && a.slug.as_str() == "hello-rust"
                    && a.is_published()
                    && !a.comments_enabled
            })
            .times(1)
            .returning(|_| Ok(()));
        articles
            .expect_find_categorized_by_id()
            .returning(|_| Ok(Some(categorized(true))));

        let dto = use_case(articles, categories(true), authors())
            .execute(admin, request())
            .await
            .unwrap();

        assert_eq!(dto.summary.slug, "hello-rust");
        assert_eq!(dto.summary.author.username, "xety");
    }

    #[tokio::test]
    async fn rejects_unknown_category() {
        let error = use_case(
            MockArticleRepository::new(),
            categories(false),
            MockAuthorDirectory::new(),
        )
        .execute(admin_principal(), request())
        .await
        .unwrap_err();

        assert_eq!(error, AppError::field("category_id", "does not exist"));
    }

    #[tokio::test]
    async fn rejects_duplicated_slug() {
        let mut articles = MockArticleRepository::new();
        articles.expect_slug_exists().returning(|_, _| Ok(true));

        let error = use_case(articles, categories(true), MockAuthorDirectory::new())
            .execute(admin_principal(), request())
            .await
            .unwrap_err();

        assert_eq!(error, AppError::field("slug", "is already taken"));
    }

    #[tokio::test]
    async fn is_reserved_to_admins() {
        let error = use_case(
            MockArticleRepository::new(),
            MockCategoryRepository::new(),
            MockAuthorDirectory::new(),
        )
        .execute(member_principal(), request())
        .await
        .unwrap_err();

        assert!(matches!(error, AppError::Forbidden(_)));
    }
}
