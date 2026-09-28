use std::net::IpAddr;
use std::sync::Arc;

use async_trait::async_trait;
use xetaravel_kernel::AppResult;

use crate::application::contract::HumanCheck;
use crate::application::ports::HumanVerifier;

/// Tells whether a captcha token was solved by a human (used by other
/// contexts to protect their public routes). This use case implements the
/// [`HumanCheck`] contract.
pub struct CheckHuman {
    humans: Arc<dyn HumanVerifier>,
}

impl CheckHuman {
    /// Builds the use case with its dependencies.
    pub fn new(humans: Arc<dyn HumanVerifier>) -> Self {
        Self { humans }
    }

    /// Asks the captcha verifier whether `token` proves a human is behind
    /// the request sent from `remote_ip`.
    pub async fn execute(&self, token: &str, remote_ip: Option<IpAddr>) -> AppResult<bool> {
        self.humans.verify(token, remote_ip).await
    }
}

#[async_trait]
impl HumanCheck for CheckHuman {
    /// Delegates to [`CheckHuman::execute`].
    async fn is_human(&self, token: &str, remote_ip: Option<IpAddr>) -> AppResult<bool> {
        self.execute(token, remote_ip).await
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use mockall::predicate::eq;
    use xetaravel_kernel::AppError;

    use super::*;
    use crate::application::ports::MockHumanVerifier;

    #[tokio::test]
    async fn forwards_the_token_and_the_ip_to_the_verifier() {
        let ip = Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)));
        let mut humans = MockHumanVerifier::new();
        humans
            .expect_verify()
            .with(eq("token"), eq(ip))
            .times(1)
            .returning(|_, _| Ok(true));

        let human = CheckHuman::new(Arc::new(humans))
            .is_human("token", ip)
            .await
            .unwrap();

        assert!(human);
    }

    #[tokio::test]
    async fn reports_a_failed_challenge() {
        let mut humans = MockHumanVerifier::new();
        humans.expect_verify().returning(|_, _| Ok(false));

        let human = CheckHuman::new(Arc::new(humans))
            .is_human("forged", None)
            .await
            .unwrap();

        assert!(!human);
    }

    #[tokio::test]
    async fn propagates_technical_failures() {
        let mut humans = MockHumanVerifier::new();
        humans
            .expect_verify()
            .returning(|_, _| Err(AppError::Internal("siteverify is down".into())));

        let result = CheckHuman::new(Arc::new(humans))
            .is_human("token", None)
            .await;

        assert!(matches!(result, Err(AppError::Internal(_))));
    }
}
