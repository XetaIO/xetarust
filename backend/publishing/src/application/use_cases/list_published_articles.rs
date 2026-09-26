use std::sync::Arc;

use xetaravel_kernel::AppResult;
use xetaravel_kernel::dto::Paginated;
use xetaravel_kernel::pagination::PageRequest;

use super::parse_optional_slug;
use crate::application::dto::{ArticleSummaryDto, ArticlesQuery};
use crate::application::ports::AuthorDirectory;
use crate::application::views::page_with_authors;
use crate::domain::{ArticleFilter, ArticleRepository};

/// Lists the published articles, optionally filtered by category.
pub struct ListPublishedArticles {
    articles: Arc<dyn ArticleRepository>,
    authors: Arc<dyn AuthorDirectory>,
}

impl ListPublishedArticles {
    /// Builds the use case with its dependencies.
    pub fn new(articles: Arc<dyn ArticleRepository>, authors: Arc<dyn AuthorDirectory>) -> Self {
        Self { articles, authors }
    }

    /// Returns one page of published articles, newest first.
    pub async fn execute(&self, query: ArticlesQuery) -> AppResult<Paginated<ArticleSummaryDto>> {
        let category = parse_optional_slug(query.category.as_deref())?;
        let page = self
            .articles
            .list_categorized(
                ArticleFilter::published(category),
                PageRequest::new(query.page, query.per_page),
            )
            .await?;
        let page = page_with_authors(self.authors.as_ref(), page).await?;

        Ok(Paginated::from_page(page, |view| (&view).into()))
    }
}

#[cfg(test)]
mod tests {
    use xetaravel_kernel::pagination::Page;

    use super::*;
    use crate::application::ports::MockAuthorDirectory;
    use crate::application::test_support::{authors, categorized};
    use crate::domain::MockArticleRepository;

    #[tokio::test]
    async fn lists_published_articles_of_a_category() {
        let mut articles = MockArticleRepository::new();
        articles
            .expect_list_categorized()
            .withf(|filter, page| {
                filter.published_only
                    && filter.category.as_ref().map(|s| s.as_str()) == Some("rust")
                    && page.page() == 2
            })
            .returning(|_, request| {
                Ok(Page {
                    items: vec![categorized(true)],
                    total: 11,
                    request,
                })
            });

        let result = ListPublishedArticles::new(Arc::new(articles), Arc::new(authors()))
            .execute(ArticlesQuery {
                page: Some(2),
                per_page: None,
                category: Some("rust".into()),
            })
            .await
            .unwrap();

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].slug, "hello-rust");
        assert_eq!(result.items[0].author.username, "xety");
        assert_eq!(result.total_pages, 2);
    }

    #[tokio::test]
    async fn rejects_malformed_category_slug() {
        let result = ListPublishedArticles::new(
            Arc::new(MockArticleRepository::new()),
            Arc::new(MockAuthorDirectory::new()),
        )
        .execute(ArticlesQuery {
            category: Some("Not a slug".into()),
            ..Default::default()
        })
        .await;

        assert!(result.is_err());
    }
}
