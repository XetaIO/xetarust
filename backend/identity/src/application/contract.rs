//! Public contract of the Identity context for the other contexts.
//!
//! Other contexts never see [`crate::domain::User`]: they ask these driving
//! ports for what they need (public names, human checks), through an
//! anti-corruption layer wired by the composition root.

use std::collections::HashMap;
use std::net::IpAddr;

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

/// Tells humans from bots, for the public routes of the other contexts.
#[async_trait]
pub trait HumanCheck: Send + Sync {
    /// Returns whether the captcha `token`, sent from `remote_ip` when known,
    /// proves a human is behind the request. Errors are reserved for
    /// technical failures.
    async fn is_human(&self, token: &str, remote_ip: Option<IpAddr>) -> AppResult<bool>;
}
