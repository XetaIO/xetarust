//! # Xetaravel application
//!
//! Composition root of the domain-first architecture: it reads the
//! configuration, builds the three bounded contexts (Identity, Publishing,
//! Discussion), wires the anti-corruption layer between them and exposes
//! their merged HTTP routers. It is the only crate depending on every context.

pub mod config;
pub mod integration;
pub mod router;
pub mod state;

pub use config::{Config, ConfigError};
pub use router::router;
pub use state::AppState;
