//! [`HumanVerifier`] accepting every request, used when no captcha secret is
//! configured (development, tests, e2e).

use std::net::IpAddr;

use async_trait::async_trait;
use xetaravel_kernel::AppResult;

use crate::application::ports::HumanVerifier;

/// Captcha verifier that always answers "human".
pub struct DisabledHumanVerifier;

#[async_trait]
impl HumanVerifier for DisabledHumanVerifier {
    /// Accepts any token, from any client.
    async fn verify(&self, _token: &str, _remote_ip: Option<IpAddr>) -> AppResult<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn accepts_every_token() {
        assert!(
            DisabledHumanVerifier
                .verify("anything", None)
                .await
                .unwrap()
        );
    }
}
