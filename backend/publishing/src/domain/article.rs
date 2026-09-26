use chrono::{DateTime, Utc};
use xetaravel_kernel::DomainResult;
use xetaravel_kernel::text::{validate_optional_text, validate_text};

use super::{ArticleId, AuthorId, CategoryId, CoverImage, Slug};

/// Maximum length of an article title.
const TITLE_MAX: usize = 200;
/// Maximum length of an article excerpt.
const EXCERPT_MAX: usize = 500;
/// Maximum length of an article body (Markdown).
const CONTENT_MAX: usize = 200_000;
/// Average reading speed used to estimate the reading time.
const WORDS_PER_MINUTE: usize = 200;

/// The editable data of an article, as typed by an admin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticleDraft {
    pub category_id: CategoryId,
    pub title: String,
    /// Explicit slug; derived from the title when `None`.
    pub slug: Option<Slug>,
    pub excerpt: Option<String>,
    /// Markdown body.
    pub content: String,
    pub publish: bool,
}

/// A blog post written in Markdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Article {
    pub id: ArticleId,
    pub author_id: AuthorId,
    pub category_id: CategoryId,
    pub title: String,
    pub slug: Slug,
    pub excerpt: Option<String>,
    pub content: String,
    /// File name of the cover image, if any.
    pub cover: Option<CoverImage>,
    /// `Some` when the article is publicly visible.
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Article {
    /// Writes a new article from a validated draft.
    pub fn write(
        author_id: AuthorId,
        draft: ArticleDraft,
        now: DateTime<Utc>,
    ) -> DomainResult<Self> {
        let title = validate_text("title", &draft.title, 3, TITLE_MAX)?;
        let slug = match draft.slug {
            Some(slug) => slug,
            None => Slug::from_text(&title)?,
        };

        Ok(Self {
            id: ArticleId::generate(),
            author_id,
            category_id: draft.category_id,
            title,
            slug,
            excerpt: validate_optional_text("excerpt", draft.excerpt.as_deref(), EXCERPT_MAX)?,
            content: validate_text("content", &draft.content, 1, CONTENT_MAX)?,
            cover: None,
            published_at: draft.publish.then_some(now),
            created_at: now,
            updated_at: now,
        })
    }

    /// Applies a draft on an existing article. The slug is kept when the draft has none,
    /// so published URLs stay stable. The original publication date is preserved.
    pub fn revise(&mut self, draft: ArticleDraft, now: DateTime<Utc>) -> DomainResult<()> {
        let title = validate_text("title", &draft.title, 3, TITLE_MAX)?;
        let excerpt = validate_optional_text("excerpt", draft.excerpt.as_deref(), EXCERPT_MAX)?;
        let content = validate_text("content", &draft.content, 1, CONTENT_MAX)?;

        self.category_id = draft.category_id;
        self.title = title;
        if let Some(slug) = draft.slug {
            self.slug = slug;
        }
        self.excerpt = excerpt;
        self.content = content;
        if draft.publish {
            self.publish(now);
        } else {
            self.unpublish();
        }
        self.updated_at = now;
        Ok(())
    }

    /// Sets a new cover image and returns the previous one, whose file must
    /// then be deleted by the caller.
    pub fn replace_cover(&mut self, cover: CoverImage, now: DateTime<Utc>) -> Option<CoverImage> {
        self.updated_at = now;
        self.cover.replace(cover)
    }

    /// Removes the cover image and returns it (if any) so its file can be deleted.
    pub fn remove_cover(&mut self, now: DateTime<Utc>) -> Option<CoverImage> {
        let previous = self.cover.take();
        if previous.is_some() {
            self.updated_at = now;
        }
        previous
    }

    /// Makes the article public. Publishing twice keeps the first date.
    pub fn publish(&mut self, now: DateTime<Utc>) {
        self.published_at.get_or_insert(now);
    }

    /// Hides the article from the public blog.
    pub fn unpublish(&mut self) {
        self.published_at = None;
    }

    /// Tells whether the article is publicly visible.
    pub fn is_published(&self) -> bool {
        self.published_at.is_some()
    }

    /// Estimates the reading time in minutes (at least 1).
    pub fn reading_time_minutes(&self) -> u32 {
        let words = self.content.split_whitespace().count();
        u32::try_from(words.div_ceil(WORDS_PER_MINUTE).max(1)).unwrap_or(u32::MAX)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::domain::ImageFormat;

    /// Builds a valid draft for the tests.
    fn draft(publish: bool) -> ArticleDraft {
        ArticleDraft {
            category_id: CategoryId::generate(),
            title: "  Hello Rust  ".into(),
            slug: None,
            excerpt: Some("An intro".into()),
            content: "# Hello\n\nSome **markdown**.".into(),
            publish,
        }
    }

    #[test]
    fn write_derives_slug_and_trims_title() {
        let article = Article::write(AuthorId::generate(), draft(false), Utc::now()).unwrap();
        assert_eq!(article.title, "Hello Rust");
        assert_eq!(article.slug.as_str(), "hello-rust");
        assert!(!article.is_published());
    }

    #[test]
    fn write_publishes_when_requested() {
        let now = Utc::now();
        let article = Article::write(AuthorId::generate(), draft(true), now).unwrap();
        assert_eq!(article.published_at, Some(now));
    }

    #[test]
    fn write_rejects_empty_content_and_short_title() {
        let mut bad = draft(false);
        bad.content = "   ".into();
        assert!(Article::write(AuthorId::generate(), bad, Utc::now()).is_err());

        let mut bad = draft(false);
        bad.title = "ab".into();
        assert!(Article::write(AuthorId::generate(), bad, Utc::now()).is_err());
    }

    #[test]
    fn revise_keeps_slug_and_first_publication_date() {
        let created = Utc::now();
        let mut article = Article::write(AuthorId::generate(), draft(true), created).unwrap();

        let mut changes = draft(true);
        changes.title = "A brand new title".into();
        let later = created + Duration::days(1);
        article.revise(changes, later).unwrap();

        assert_eq!(article.title, "A brand new title");
        assert_eq!(article.slug.as_str(), "hello-rust");
        assert_eq!(article.published_at, Some(created));
        assert_eq!(article.updated_at, later);
    }

    #[test]
    fn revise_can_unpublish_and_change_slug() {
        let mut article = Article::write(AuthorId::generate(), draft(true), Utc::now()).unwrap();
        let mut changes = draft(false);
        changes.slug = Some(Slug::parse("new-slug").unwrap());

        article.revise(changes, Utc::now()).unwrap();

        assert!(!article.is_published());
        assert_eq!(article.slug.as_str(), "new-slug");
    }

    #[test]
    fn replace_cover_returns_the_previous_one() {
        let created = Utc::now();
        let mut article = Article::write(AuthorId::generate(), draft(false), created).unwrap();
        assert_eq!(article.cover, None);

        let first = CoverImage::new(ImageFormat::Png);
        let later = created + Duration::hours(1);
        assert_eq!(article.replace_cover(first.clone(), later), None);
        assert_eq!(article.cover.as_ref(), Some(&first));
        assert_eq!(article.updated_at, later);

        let second = CoverImage::new(ImageFormat::Jpeg);
        assert_eq!(article.replace_cover(second.clone(), later), Some(first));
        assert_eq!(article.cover, Some(second));
    }

    #[test]
    fn remove_cover_returns_the_removed_one() {
        let created = Utc::now();
        let mut article = Article::write(AuthorId::generate(), draft(false), created).unwrap();
        let later = created + Duration::hours(1);

        assert_eq!(article.remove_cover(later), None);
        assert_eq!(article.updated_at, created);

        let cover = CoverImage::new(ImageFormat::Webp);
        article.replace_cover(cover.clone(), created);
        assert_eq!(article.remove_cover(later), Some(cover));
        assert_eq!(article.cover, None);
        assert_eq!(article.updated_at, later);
    }

    #[test]
    fn revise_keeps_the_cover() {
        let mut article = Article::write(AuthorId::generate(), draft(false), Utc::now()).unwrap();
        let cover = CoverImage::new(ImageFormat::Png);
        article.replace_cover(cover.clone(), Utc::now());

        article.revise(draft(true), Utc::now()).unwrap();

        assert_eq!(article.cover, Some(cover));
    }

    #[test]
    fn reading_time_is_at_least_one_minute() {
        let mut article = Article::write(AuthorId::generate(), draft(false), Utc::now()).unwrap();
        assert_eq!(article.reading_time_minutes(), 1);

        article.content = "word ".repeat(401);
        assert_eq!(article.reading_time_minutes(), 3);
    }
}
