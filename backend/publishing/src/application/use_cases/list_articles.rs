use std::sync::Arc;

use xetaravel_kernel::dto::{PageQuery, Paginated};
use xetaravel_kernel::{AppResult, Principal};

use crate::application::dto::ArticleSummaryDto;
use crate::application::ports::AuthorDirectory;
use crate::application::views::page_with_authors;
use crate::domain::{ArticleFilter, ArticleRepository};

/// Lists every article, drafts included, for the administration.
pub struct ListArticles {
    articles: Arc<dyn ArticleRepository>,
    authors: Arc<dyn AuthorDirectory>,
}

impl ListArticles {
    /// Builds the use case with its dependencies.
    pub fn new(articles: Arc<dyn ArticleRepository>, authors: Arc<dyn AuthorDirectory>) -> Self {
        Self { articles, authors }
    }

    /// Returns one page of articles, newest first.
    pub async fn execute(
        &self,
        principal: Principal,
        query: PageQuery,
    ) -> AppResult<Paginated<ArticleSummaryDto>> {
        principal.require_admin()?;
        let page = self
            .articles
            .list_categorized(ArticleFilter::all(), query.to_page_request())
            .await?;
        let page = page_with_authors(self.authors.as_ref(), page).await?;

        Ok(Paginated::from_page(page, |view| (&view).into()))
    }
}

#[cfg(test)]
mod tests {
    use xetaravel_kernel::AppError;
    use xetaravel_kernel::pagination::Page;

    use super::*;
    use crate::application::ports::MockAuthorDirectory;
    use crate::application::test_support::{
        admin_principal, authors, categorized, member_principal,
    };
    use crate::domain::MockArticleRepository;

    #[tokio::test]
    async fn lists_drafts_too() {
        let mut articles = MockArticleRepository::new();
        articles
            .expect_list_categorized()
            .withf(|filter, _| !filter.published_only)
            .returning(|_, request| {
                Ok(Page {
                    items: vec![categorized(false)],
                    total: 1,
                    request,
                })
            });

        let result = ListArticles::new(Arc::new(articles), Arc::new(authors()))
            .execute(admin_principal(), PageQuery::default())
            .await
            .unwrap();

        assert!(!result.items[0].is_published);
    }

    #[tokio::test]
    async fn is_reserved_to_admins() {
        let error = ListArticles::new(
            Arc::new(MockArticleRepository::new()),
            Arc::new(MockAuthorDirectory::new()),
        )
        .execute(member_principal(), PageQuery::default())
        .await
        .unwrap_err();
        assert!(matches!(error, AppError::Forbidden(_)));
    }
}
