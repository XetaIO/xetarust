//! Anti-corruption layer between the bounded contexts.
//!
//! Contexts never depend on each other: each one declares the ports it needs
//! (`AuthorDirectory`, `ArticleCatalog`) and exposes a small public contract
//! (`IdentityDirectory`, `PublishedArticles`). The adapters below plug the
//! contracts into the ports, translating identifiers at the boundary.

mod articles;
mod authors;

pub use articles::PublishingArticleCatalog;
pub use authors::IdentityAuthorDirectory;
