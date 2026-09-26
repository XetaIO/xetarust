//! Article views: an article with its category (from the repository) and
//! its author name (from the [`AuthorDirectory`] port).

use xetaravel_kernel::AppResult;
use xetaravel_kernel::pagination::Page;

use crate::application::ports::AuthorDirectory;
use crate::domain::{Article, AuthorId, CategorizedArticle, Category};

/// Name displayed when the author is unknown to the directory.
pub const UNKNOWN_AUTHOR: &str = "unknown";

/// Public identity of the author of an article.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorSummary {
    pub id: AuthorId,
    pub username: String,
}

/// An article with its author and category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticleView {
    pub article: Article,
    pub author: AuthorSummary,
    pub category: Category,
}

/// Attaches the author names to articles with a single directory lookup.
pub async fn with_authors(
    authors: &dyn AuthorDirectory,
    entries: Vec<CategorizedArticle>,
) -> AppResult<Vec<ArticleView>> {
    let mut ids: Vec<AuthorId> = entries.iter().map(|e| e.article.author_id).collect();
    ids.sort_unstable();
    ids.dedup();
    let names = if ids.is_empty() {
        Default::default()
    } else {
        authors.names(&ids).await?
    };

    Ok(entries
        .into_iter()
        .map(|entry| {
            let id = entry.article.author_id;
            let username = names
                .get(&id)
                .cloned()
                .unwrap_or_else(|| UNKNOWN_AUTHOR.to_owned());
            ArticleView {
                article: entry.article,
                author: AuthorSummary { id, username },
                category: entry.category,
            }
        })
        .collect())
}

/// Attaches the author name to one article.
pub async fn with_author(
    authors: &dyn AuthorDirectory,
    entry: CategorizedArticle,
) -> AppResult<ArticleView> {
    let mut views = with_authors(authors, vec![entry]).await?;
    Ok(views.remove(0))
}

/// Attaches the author names to every article of a page.
pub async fn page_with_authors(
    authors: &dyn AuthorDirectory,
    page: Page<CategorizedArticle>,
) -> AppResult<Page<ArticleView>> {
    Ok(Page {
        items: with_authors(authors, page.items).await?,
        total: page.total,
        request: page.request,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::application::ports::MockAuthorDirectory;
    use crate::application::test_support::{authors, categorized};

    #[tokio::test]
    async fn attaches_names_with_one_lookup() {
        let first = categorized(true);
        let second = categorized(false);
        let views = with_authors(&authors(), vec![first.clone(), second])
            .await
            .unwrap();

        assert_eq!(views.len(), 2);
        assert_eq!(views[0].article, first.article);
        assert_eq!(views[0].author.username, "xety");
        assert_eq!(views[0].category, first.category);
    }

    #[tokio::test]
    async fn falls_back_for_unknown_authors() {
        let mut directory = MockAuthorDirectory::new();
        directory.expect_names().returning(|_| Ok(HashMap::new()));

        let view = with_author(&directory, categorized(true)).await.unwrap();

        assert_eq!(view.author.username, UNKNOWN_AUTHOR);
    }

    #[tokio::test]
    async fn skips_the_lookup_when_empty() {
        let views = with_authors(&MockAuthorDirectory::new(), vec![])
            .await
            .unwrap();
        assert!(views.is_empty());
    }
}
