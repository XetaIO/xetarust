//! Public contract of the Identity context for the other contexts.
//!
//! Other contexts never see [`crate::domain::User`]: they ask this driving
//! port for what they need (public names), through an anti-corruption layer
//! wired by the composition root.

use std::collections::HashMap;

use async_trait::async_trait;
use uuid::Uuid;
use xetaravel_kernel::AppResult;

/// Read-only directory of public user profiles.
#[async_trait]
pub trait IdentityDirectory: Send + Sync {
    /// Returns the public name of every known user among `ids`, keyed by id.
    /// Unknown ids are simply absent from the map.
    async fn public_names(&self, ids: &[Uuid]) -> AppResult<HashMap<Uuid, String>>;
}
