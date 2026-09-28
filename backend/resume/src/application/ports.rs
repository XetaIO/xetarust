//! Outgoing ports of the Resume context, implemented outside of it.

use std::net::IpAddr;

use async_trait::async_trait;
use xetaravel_kernel::AppResult;

/// Tells humans from bots (captcha challenge). Implemented by the
/// composition root on top of the Identity context.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait HumanVerifier: Send + Sync {
    /// Checks that a request comes from a human.
    ///
    /// `token` is the response of the captcha widget and `remote_ip` the
    /// address of the client, when known. Returns `false` when the challenge
    /// failed; errors are reserved for technical failures.
    async fn verify(&self, token: &str, remote_ip: Option<IpAddr>) -> AppResult<bool>;
}
