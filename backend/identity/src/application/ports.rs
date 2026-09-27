//! Technical driven ports of the Identity context. The infrastructure
//! provides the implementations (Argon2, JWT, Turnstile); tests use mocks.

use std::net::IpAddr;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use xetaravel_kernel::AppResult;

use crate::domain::{PasswordHash, User, UserId};

/// Hashes and verifies passwords.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PasswordHasher: Send + Sync {
    /// Hashes a plain password with a random salt.
    async fn hash(&self, plain: &str) -> AppResult<PasswordHash>;

    /// Checks a plain password against a stored hash.
    async fn verify(&self, plain: &str, hash: &PasswordHash) -> AppResult<bool>;
}

/// A freshly issued access token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Issues and verifies stateless access tokens.
#[cfg_attr(test, mockall::automock)]
pub trait TokenService: Send + Sync {
    /// Issues a token identifying `user`.
    fn issue(&self, user: &User) -> AppResult<IssuedToken>;

    /// Verifies a token (signature + expiration) and returns the user id it carries.
    fn verify(&self, token: &str) -> AppResult<UserId>;
}

/// Tells humans from bots (captcha challenge).
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait HumanVerifier: Send + Sync {
    /// Checks that a request comes from a human (captcha challenge).
    ///
    /// `token` is the response of the captcha widget and `remote_ip` the
    /// address of the client, when known. Returns `false` when the challenge
    /// failed; errors are reserved for technical failures.
    async fn verify(&self, token: &str, remote_ip: Option<IpAddr>) -> AppResult<bool>;
}
