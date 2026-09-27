//! # Identity bounded context
//!
//! Identity, authentication and authorization of Xetaravel: users, roles,
//! passwords and access tokens.
//!
//! The crate keeps the hexagonal layering inside the context:
//! `domain` ← `application` ← `infrastructure` / `http`, assembled by
//! [`IdentityModule`]. It depends on the shared kernel only; other contexts
//! reach it through [`IdentityDirectory`] and the kernel `PrincipalResolver`
//! port, wired by the composition root.

pub mod application;
pub mod domain;
pub mod http;
pub mod infrastructure;
mod module;

pub use application::contract::IdentityDirectory;
pub use http::{account_router, auth_router, router};
pub use infrastructure::migrations::migrations;
pub use infrastructure::security::{CaptchaSettings, JwtSettings, TURNSTILE_SITEVERIFY_URL};
pub use module::IdentityModule;
