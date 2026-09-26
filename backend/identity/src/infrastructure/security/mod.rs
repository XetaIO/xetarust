//! Security adapters: password hashing and access tokens.

mod argon2_password_hasher;
mod jwt_token_service;

pub use argon2_password_hasher::Argon2PasswordHasher;
pub use jwt_token_service::JwtTokenService;

use chrono::Duration;

/// Settings of the access tokens, read from the environment by the composition root.
#[derive(Debug, Clone)]
pub struct JwtSettings {
    /// HS256 shared secret.
    pub secret: String,
    /// Lifetime of an issued token.
    pub ttl: Duration,
}
