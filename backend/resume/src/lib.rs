//! # Resume bounded context
//!
//! Download of Emeric Fevre's resume (PDF), reserved to humans: every
//! download is guarded by a captcha challenge.
//!
//! The crate keeps the hexagonal layering inside the context:
//! `domain` ← `application` ← `infrastructure` / `http`, assembled by
//! [`ResumeModule`]. It depends on the shared kernel only: the captcha is
//! checked through the [`HumanVerifier`] port, implemented by the composition
//! root. It owns no table, hence no migrations.

pub mod application;
pub mod domain;
pub mod http;
pub mod infrastructure;
mod module;

pub use application::ports::HumanVerifier;
pub use http::router;
pub use module::ResumeModule;
