use std::sync::Arc;

use uuid::Uuid;
use xetaravel_kernel::{AppResult, Principal};

use super::load_article;
use crate::application::dto::ArticleDto;
use crate::application::ports::AuthorDirectory;
use crate::domain::{ArticleId, ArticleRepository};

/// Loads any article (drafts included) for edition.
pub struct GetArticle {
    articles: Arc<dyn ArticleRepository>,
    authors: Arc<dyn AuthorDirectory>,
}

impl GetArticle {
    /// Builds the use case with its dependencies.
    pub fn new(articles: Arc<dyn ArticleRepository>, authors: Arc<dyn AuthorDirectory>) -> Self {
        Self { articles, authors }
    }

    /// Returns the article with its content.
    pub async fn execute(&self, principal: Principal, id: Uuid) -> AppResult<ArticleDto> {
        principal.require_admin()?;
        load_article(
            self.articles.as_ref(),
            self.authors.as_ref(),
            ArticleId::from(id),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use xetaravel_kernel::AppError;

    use super::*;
    use crate::application::ports::MockAuthorDirectory;
    use crate::application::test_support::{
        admin_principal, authors, categorized, member_principal,
    };
    use crate::domain::MockArticleRepository;

    #[tokio::test]
    async fn returns_drafts() {
        let mut articles = MockArticleRepository::new();
        articles
            .expect_find_categorized_by_id()
            .returning(|_| Ok(Some(categorized(false))));

        let article = GetArticle::new(Arc::new(articles), Arc::new(authors()))
            .execute(admin_principal(), Uuid::now_v7())
            .await
            .unwrap();

        assert!(!article.summary.is_published);
    }

    #[tokio::test]
    async fn fails_when_missing_or_not_admin() {
        let mut articles = MockArticleRepository::new();
        articles
            .expect_find_categorized_by_id()
            .returning(|_| Ok(None));
        let use_case = GetArticle::new(Arc::new(articles), Arc::new(MockAuthorDirectory::new()));

        assert!(matches!(
            use_case.execute(admin_principal(), Uuid::now_v7()).await,
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            use_case.execute(member_principal(), Uuid::now_v7()).await,
            Err(AppError::Forbidden(_))
        ));
    }
}
