use chrono::{DateTime, Utc};
use xetaravel_kernel::DomainResult;
use xetaravel_kernel::text::{validate_optional_text, validate_text};

use super::{CategoryId, Slug};

/// Maximum length of a category name.
const NAME_MAX: usize = 100;
/// Maximum length of a category description.
const DESCRIPTION_MAX: usize = 500;

/// A blog category grouping articles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub id: CategoryId,
    pub name: String,
    pub slug: Slug,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Category {
    /// Creates a category. When `slug` is `None` it is derived from the name.
    pub fn create(
        name: &str,
        slug: Option<Slug>,
        description: Option<&str>,
        now: DateTime<Utc>,
    ) -> DomainResult<Self> {
        let name = validate_text("name", name, 2, NAME_MAX)?;
        let slug = match slug {
            Some(slug) => slug,
            None => Slug::from_text(&name)?,
        };

        Ok(Self {
            id: CategoryId::generate(),
            name,
            slug,
            description: validate_optional_text("description", description, DESCRIPTION_MAX)?,
            created_at: now,
            updated_at: now,
        })
    }

    /// Updates the editable fields. When `slug` is `None` the current slug is kept.
    pub fn update(
        &mut self,
        name: &str,
        slug: Option<Slug>,
        description: Option<&str>,
        now: DateTime<Utc>,
    ) -> DomainResult<()> {
        let name = validate_text("name", name, 2, NAME_MAX)?;
        let description = validate_optional_text("description", description, DESCRIPTION_MAX)?;

        self.name = name;
        if let Some(slug) = slug {
            self.slug = slug;
        }
        self.description = description;
        self.updated_at = now;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_derives_slug_from_name() {
        let category = Category::create(" Rust & Web ", None, Some("  "), Utc::now()).unwrap();
        assert_eq!(category.name, "Rust & Web");
        assert_eq!(category.slug.as_str(), "rust-web");
        assert_eq!(category.description, None);
    }

    #[test]
    fn create_keeps_explicit_slug() {
        let slug = Slug::parse("custom").unwrap();
        let category = Category::create("Rust", Some(slug.clone()), None, Utc::now()).unwrap();
        assert_eq!(category.slug, slug);
    }

    #[test]
    fn create_rejects_too_short_name() {
        assert!(Category::create("a", None, None, Utc::now()).is_err());
    }

    #[test]
    fn update_keeps_slug_when_none() {
        let mut category = Category::create("Rust", None, None, Utc::now()).unwrap();
        category
            .update("Rust lang", None, Some("All about Rust"), Utc::now())
            .unwrap();
        assert_eq!(category.name, "Rust lang");
        assert_eq!(category.slug.as_str(), "rust");
        assert_eq!(category.description.as_deref(), Some("All about Rust"));
    }

    #[test]
    fn failed_update_leaves_category_untouched() {
        let mut category = Category::create("Rust", None, None, Utc::now()).unwrap();
        let before = category.clone();
        assert!(category.update("", None, None, Utc::now()).is_err());
        assert_eq!(category, before);
    }
}
