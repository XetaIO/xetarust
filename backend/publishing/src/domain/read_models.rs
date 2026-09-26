//! Read-only projections returned by the article repository.

use super::{Article, Category, Slug};

/// An article with its category (both owned by Publishing, so the repository
/// can join them). Authors are attached later by the application layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategorizedArticle {
    pub article: Article,
    pub category: Category,
}

/// Criteria used to list articles.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArticleFilter {
    /// When `true`, drafts are excluded.
    pub published_only: bool,
    /// Restricts the results to one category.
    pub category: Option<Slug>,
}

impl ArticleFilter {
    /// Filter used by the public blog: published articles only.
    pub fn published(category: Option<Slug>) -> Self {
        Self {
            published_only: true,
            category,
        }
    }

    /// Filter used by the administration: every article, drafts included.
    pub fn all() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn published_filter_excludes_drafts() {
        let slug = Slug::parse("rust").unwrap();
        let filter = ArticleFilter::published(Some(slug.clone()));
        assert!(filter.published_only);
        assert_eq!(filter.category, Some(slug));
        assert!(!ArticleFilter::all().published_only);
    }
}
