//! Anti-corruption layer between the bounded contexts.
//!
//! Contexts never depend on each other: each one declares the ports it needs
//! (`AuthorDirectory`, `ArticleCatalog`, `HumanVerifier`) and exposes a small
//! public contract (`IdentityDirectory`, `PublishedArticles`, `HumanCheck`). The adapters below plug the
//! contracts into the ports, translating identifiers at the boundary.

mod articles;
mod authors;
mod humans;

pub use articles::PublishingArticleCatalog;
pub use authors::IdentityAuthorDirectory;
pub use humans::IdentityHumanVerifier;
