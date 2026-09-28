//! # Discussion bounded context
//!
//! Interactions around the published content: reader comments.
//!
//! The crate keeps the hexagonal layering inside the context:
//! `domain` ← `application` ← `infrastructure` / `http`, assembled by
//! [`DiscussionModule`]. It depends on the shared kernel only: articles and
//! authors are reached through the [`ArticleCatalog`] and [`AuthorDirectory`]
//! ports, implemented by the composition root.

pub mod application;
pub mod domain;
pub mod http;
pub mod infrastructure;
mod module;

pub use application::ports::{ArticleCatalog, AuthorDirectory};
pub use domain::CommentThrottle;
pub use http::router;
pub use infrastructure::migrations::migrations;
pub use module::DiscussionModule;
