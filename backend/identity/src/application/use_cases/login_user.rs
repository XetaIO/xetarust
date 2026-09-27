use std::net::IpAddr;
use std::sync::Arc;

use validator::Validate;
use xetaravel_kernel::{AppError, AppResult};

use super::{authenticated_response, ensure_human};
use crate::application::dto::{AuthResponse, LoginRequest};
use crate::application::ports::{HumanVerifier, PasswordHasher, TokenService};
use crate::domain::{Email, UserRepository};

/// Message returned for every failed login, so attackers cannot tell
/// whether an email is registered.
const INVALID_CREDENTIALS: &str = "invalid credentials";

/// Exchanges an email/password pair for an access token.
pub struct LoginUser {
    users: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenService>,
    humans: Arc<dyn HumanVerifier>,
}

impl LoginUser {
    /// Builds the use case with its dependencies.
    pub fn new(
        users: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenService>,
        humans: Arc<dyn HumanVerifier>,
    ) -> Self {
        Self {
            users,
            hasher,
            tokens,
            humans,
        }
    }

    /// Checks the captcha, then the credentials, and returns an access token.
    ///
    /// The captcha is verified before any storage access or password hashing,
    /// so a bot never costs a database query nor an Argon2 computation.
    pub async fn execute(
        &self,
        input: LoginRequest,
        client_ip: Option<IpAddr>,
    ) -> AppResult<AuthResponse> {
        input.validate()?;
        ensure_human(self.humans.as_ref(), &input.captcha_token, client_ip).await?;
        let email = Email::parse(&input.email).map_err(|_| Self::invalid_credentials())?;

        let user = self
            .users
            .find_by_email(&email)
            .await?
            .ok_or_else(Self::invalid_credentials)?;

        if !self
            .hasher
            .verify(&input.password, &user.password_hash)
            .await?
        {
            return Err(Self::invalid_credentials());
        }

        authenticated_response(self.tokens.as_ref(), &user)
    }

    /// Builds the generic "invalid credentials" error.
    fn invalid_credentials() -> AppError {
        AppError::Unauthorized(INVALID_CREDENTIALS.into())
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use super::*;
    use crate::application::ports::{
        IssuedToken, MockHumanVerifier, MockPasswordHasher, MockTokenService,
    };
    use crate::application::test_support::{human, now, user};
    use crate::domain::{MockUserRepository, Role};

    /// Returns a login request for `email`.
    fn request(email: &str) -> LoginRequest {
        LoginRequest {
            email: email.into(),
            password: "super-secret".into(),
            captcha_token: "captcha".into(),
        }
    }

    /// Builds the use case under test with a captcha always solved.
    fn use_case(users: MockUserRepository, hasher: MockPasswordHasher) -> LoginUser {
        use_case_with(users, hasher, human(true))
    }

    /// Builds the use case under test with the given captcha verifier.
    fn use_case_with(
        users: MockUserRepository,
        hasher: MockPasswordHasher,
        humans: Arc<MockHumanVerifier>,
    ) -> LoginUser {
        let mut tokens = MockTokenService::new();
        tokens.expect_issue().returning(|_| {
            Ok(IssuedToken {
                token: "jwt".into(),
                expires_at: now(),
            })
        });
        LoginUser::new(Arc::new(users), Arc::new(hasher), Arc::new(tokens), humans)
    }

    /// Returns a repository mock knowing a single user, `john`.
    fn users_with_john() -> MockUserRepository {
        let mut users = MockUserRepository::new();
        users.expect_find_by_email().returning(|email| {
            Ok((email.as_str() == "john@example.com").then(|| user("john", Role::Member)))
        });
        users
    }

    /// Returns a hasher mock answering `valid` to every verification.
    fn hasher(valid: bool) -> MockPasswordHasher {
        let mut hasher = MockPasswordHasher::new();
        hasher.expect_verify().returning(move |_, _| Ok(valid));
        hasher
    }

    #[tokio::test]
    async fn logs_in_with_valid_credentials() {
        let response = use_case(users_with_john(), hasher(true))
            .execute(request("John@Example.com"), None)
            .await
            .unwrap();

        assert_eq!(response.token, "jwt");
        assert_eq!(response.user.username, "john");
    }

    #[tokio::test]
    async fn rejects_wrong_password() {
        let error = use_case(users_with_john(), hasher(false))
            .execute(request("john@example.com"), None)
            .await
            .unwrap_err();

        assert_eq!(error, AppError::Unauthorized(INVALID_CREDENTIALS.into()));
    }

    #[tokio::test]
    async fn rejects_unknown_email_with_the_same_error() {
        let error = use_case(users_with_john(), hasher(true))
            .execute(request("ghost@example.com"), None)
            .await
            .unwrap_err();

        assert_eq!(error, AppError::Unauthorized(INVALID_CREDENTIALS.into()));
    }

    #[tokio::test]
    async fn rejects_a_failed_captcha_without_touching_the_repository() {
        let mut users = MockUserRepository::new();
        users.expect_find_by_email().times(0);
        let mut hasher = MockPasswordHasher::new();
        hasher.expect_verify().times(0);

        let error = use_case_with(users, hasher, human(false))
            .execute(request("john@example.com"), None)
            .await
            .unwrap_err();

        assert_eq!(
            error,
            AppError::field("captcha_token", "verification failed, please retry")
        );
    }

    #[tokio::test]
    async fn forwards_the_client_ip_to_the_verifier() {
        let ip: IpAddr = "203.0.113.7".parse().unwrap();
        let mut humans = MockHumanVerifier::new();
        humans
            .expect_verify()
            .with(eq("captcha"), eq(Some(ip)))
            .times(1)
            .returning(|_, _| Ok(true));

        let response = use_case_with(users_with_john(), hasher(true), Arc::new(humans))
            .execute(request("john@example.com"), Some(ip))
            .await
            .unwrap();

        assert_eq!(response.user.username, "john");
    }
}
