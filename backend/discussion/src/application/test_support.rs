//! Fixtures shared by the use case tests.

use std::sync::Arc;

use chrono::{DateTime, TimeZone, Utc};
use uuid::Uuid;
use xetaravel_kernel::{FixedClock, Principal};

use crate::application::ports::{MockArticleCatalog, MockAuthorDirectory};
use crate::domain::{ArticleId, AuthorId, Comment};

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

/// Builds a comment written by `principal`.
pub fn comment_by(principal: Principal) -> Comment {
    Comment::post(
        ArticleId::generate(),
        AuthorId::from(principal.user_id),
        "Great post",
        now(),
    )
    .unwrap()
}

/// Returns a catalog where every slug is a published article (or none is).
pub fn catalog(published: bool) -> MockArticleCatalog {
    let mut catalog = MockArticleCatalog::new();
    catalog
        .expect_published_article_id()
        .returning(move |_| Ok(published.then(ArticleId::generate)));
    catalog
}

/// Returns an author directory naming every author "john".
pub fn authors() -> MockAuthorDirectory {
    let mut authors = MockAuthorDirectory::new();
    authors
        .expect_names()
        .returning(|ids| Ok(ids.iter().map(|id| (*id, "john".to_owned())).collect()));
    authors
}
