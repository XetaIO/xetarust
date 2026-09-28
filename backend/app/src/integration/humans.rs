//! Human checks for Resume, served by Identity.

use std::net::IpAddr;
use std::sync::Arc;

use async_trait::async_trait;
use xetaravel_identity::HumanCheck;
use xetaravel_kernel::AppResult;

/// Implements the `HumanVerifier` port of Resume on top of the Identity
/// captcha check.
pub struct IdentityHumanVerifier {
    identity: Arc<dyn HumanCheck>,
}

impl IdentityHumanVerifier {
    /// Wraps the Identity captcha check.
    pub fn new(identity: Arc<dyn HumanCheck>) -> Self {
        Self { identity }
    }
}

#[async_trait]
impl xetaravel_resume::HumanVerifier for IdentityHumanVerifier {
    /// Asks Identity whether `token` was solved by a human.
    async fn verify(&self, token: &str, remote_ip: Option<IpAddr>) -> AppResult<bool> {
        self.identity.is_human(token, remote_ip).await
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use std::sync::Mutex;

    use xetaravel_resume::HumanVerifier;

    use super::*;

    /// Identity captcha check accepting a single token, recording the calls.
    struct FakeIdentity {
        calls: Mutex<Vec<(String, Option<IpAddr>)>>,
    }

    #[async_trait]
    impl HumanCheck for FakeIdentity {
        /// Accepts the "valid" token only.
        async fn is_human(&self, token: &str, remote_ip: Option<IpAddr>) -> AppResult<bool> {
            self.calls
                .lock()
                .unwrap()
                .push((token.to_owned(), remote_ip));
            Ok(token == "valid")
        }
    }

    /// Builds the adapter over a fresh fake.
    fn verifier() -> (IdentityHumanVerifier, Arc<FakeIdentity>) {
        let fake = Arc::new(FakeIdentity {
            calls: Mutex::new(vec![]),
        });
        (IdentityHumanVerifier::new(fake.clone()), fake)
    }

    #[tokio::test]
    async fn forwards_the_token_and_the_ip_to_identity() {
        let (verifier, fake) = verifier();
        let ip = Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)));

        assert!(verifier.verify("valid", ip).await.unwrap());
        assert_eq!(*fake.calls.lock().unwrap(), [("valid".to_owned(), ip)]);
    }

    #[tokio::test]
    async fn reports_bots() {
        let (verifier, _) = verifier();
        assert!(!verifier.verify("forged", None).await.unwrap());
    }
}
