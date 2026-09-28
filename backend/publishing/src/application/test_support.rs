//! Fixtures shared by the use case tests.

use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use uuid::Uuid;
use xetaravel_kernel::{FixedClock, Principal};

use crate::application::ports::MockAuthorDirectory;
use crate::domain::{Article, ArticleDraft, AuthorId, CategorizedArticle, Category};

/// Returns the instant every test clock is frozen at.
pub fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap()
}

/// Returns a fixed clock frozen at [`now`].
pub fn clock() -> Arc<FixedClock> {
    Arc::new(FixedClock(now()))
}

/// Builds an admin principal.
pub fn admin_principal() -> Principal {
    Principal::admin(Uuid::now_v7())
}

/// Builds a member principal.
pub fn member_principal() -> Principal {
    Principal::member(Uuid::now_v7())
}

/// Builds a category named `name`.
pub fn category(name: &str) -> Category {
    Category::create(name, None, None, now()).unwrap()
}

/// Builds an article in `category` written by `author`.
pub fn article(author: AuthorId, category: &Category, publish: bool) -> Article {
    Article::write(
        author,
        ArticleDraft {
            category_id: category.id,
            title: "Hello Rust".into(),
            slug: None,
            excerpt: None,
            content: "Some content".into(),
            publish,
            comments_enabled: true,
        },
        now(),
    )
    .unwrap()
}

/// Builds an article with its category.
pub fn categorized(publish: bool) -> CategorizedArticle {
    let category = category("Rust");
    CategorizedArticle {
        article: article(AuthorId::generate(), &category, publish),
        category,
    }
}

/// Returns an author directory naming every author "xety".
pub fn authors() -> MockAuthorDirectory {
    let mut authors = MockAuthorDirectory::new();
    authors
        .expect_names()
        .returning(|ids| Ok(ids.iter().map(|id| (*id, "xety".to_owned())).collect()));
    authors
}
