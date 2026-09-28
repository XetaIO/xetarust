use std::net::IpAddr;
use std::sync::Arc;

use validator::Validate;
use xetaravel_kernel::{AppError, AppResult};

use crate::application::dto::DownloadResumeRequest;
use crate::application::ports::HumanVerifier;
use crate::domain::{ResumeFile, ResumeStore};

/// Message returned when the captcha challenge fails.
const CAPTCHA_FAILED: &str = "the captcha verification failed, please try again";

/// Hands the resume over to humans only.
pub struct DownloadResume {
    store: Arc<dyn ResumeStore>,
    humans: Arc<dyn HumanVerifier>,
}

impl DownloadResume {
    /// Builds the use case with its dependencies.
    pub fn new(store: Arc<dyn ResumeStore>, humans: Arc<dyn HumanVerifier>) -> Self {
        Self { store, humans }
    }

    /// Checks the captcha sent from `client_ip`, then returns the resume.
    /// A missing token is rejected without calling the captcha verifier.
    pub async fn execute(
        &self,
        input: DownloadResumeRequest,
        client_ip: Option<IpAddr>,
    ) -> AppResult<ResumeFile> {
        input.validate()?;
        if !self.humans.verify(&input.captcha_token, client_ip).await? {
            return Err(AppError::field("captcha_token", CAPTCHA_FAILED));
        }
        Ok(self.store.current())
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use mockall::predicate::eq;

    use super::*;
    use crate::application::ports::MockHumanVerifier;
    use crate::domain::MockResumeStore;

    /// Resume returned by the store in the tests.
    const RESUME: ResumeFile = ResumeFile {
        filename: "resume.pdf",
        bytes: b"%PDF-1.7",
    };

    /// Returns a download request carrying `token`.
    fn request(token: &str) -> DownloadResumeRequest {
        DownloadResumeRequest {
            captcha_token: token.into(),
        }
    }

    /// Returns a store serving [`RESUME`].
    fn store() -> Arc<MockResumeStore> {
        let mut store = MockResumeStore::new();
        store.expect_current().returning(|| RESUME);
        Arc::new(store)
    }

    #[tokio::test]
    async fn serves_the_resume_to_humans() {
        let ip = Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)));
        let mut humans = MockHumanVerifier::new();
        humans
            .expect_verify()
            .with(eq("captcha"), eq(ip))
            .times(1)
            .returning(|_, _| Ok(true));

        let file = DownloadResume::new(store(), Arc::new(humans))
            .execute(request("captcha"), ip)
            .await
            .unwrap();

        assert_eq!(file, RESUME);
    }

    #[tokio::test]
    async fn rejects_a_failed_captcha() {
        let mut humans = MockHumanVerifier::new();
        humans.expect_verify().returning(|_, _| Ok(false));

        let error = DownloadResume::new(Arc::new(MockResumeStore::new()), Arc::new(humans))
            .execute(request("forged"), None)
            .await
            .unwrap_err();

        assert_eq!(error, AppError::field("captcha_token", CAPTCHA_FAILED));
    }

    #[tokio::test]
    async fn requires_a_token_before_asking_the_verifier() {
        // No expectation: any call to the verifier fails the test.
        let humans = MockHumanVerifier::new();

        let error = DownloadResume::new(Arc::new(MockResumeStore::new()), Arc::new(humans))
            .execute(request(""), None)
            .await
            .unwrap_err();

        assert!(
            matches!(error, AppError::Validation(fields) if fields.contains_key("captcha_token"))
        );
    }
}
