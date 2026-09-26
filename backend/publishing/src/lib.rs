//! # Publishing bounded context
//!
//! Creation and publication of the blog content: articles written in
//! Markdown and their categories.
//!
//! The crate keeps the hexagonal layering inside the context:
//! `domain` ← `application` ← `infrastructure` / `http`, assembled by
//! [`PublishingModule`]. It depends on the shared kernel only: author names
//! come through the [`AuthorDirectory`] port, and other contexts reach it
//! through the [`PublishedArticles`] contract.

pub mod application;
pub mod domain;
pub mod http;
pub mod infrastructure;
mod module;

pub use application::contract::{PublishedArticleRef, PublishedArticles};
pub use application::ports::AuthorDirectory;
pub use http::router;
pub use infrastructure::migrations::migrations;
pub use module::PublishingModule;
