use std::sync::Arc;

use validator::Validate;
use xetaravel_kernel::{AppError, AppResult};

use super::authenticated_response;
use crate::application::dto::{AuthResponse, LoginRequest};
use crate::application::ports::{PasswordHasher, TokenService};
use crate::domain::{Email, UserRepository};

/// Message returned for every failed login, so attackers cannot tell
/// whether an email is registered.
const INVALID_CREDENTIALS: &str = "invalid credentials";

/// Exchanges an email/password pair for an access token.
pub struct LoginUser {
    users: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenService>,
}

impl LoginUser {
    /// Builds the use case with its dependencies.
    pub fn new(
        users: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenService>,
    ) -> Self {
        Self {
            users,
            hasher,
            tokens,
        }
    }

    /// Checks the credentials and returns an access token.
    pub async fn execute(&self, input: LoginRequest) -> AppResult<AuthResponse> {
        input.validate()?;
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
    use super::*;
    use crate::application::ports::{IssuedToken, MockPasswordHasher, MockTokenService};
    use crate::application::test_support::{now, user};
    use crate::domain::{MockUserRepository, Role};

    /// Returns a login request for `email`.
    fn request(email: &str) -> LoginRequest {
        LoginRequest {
            email: email.into(),
            password: "super-secret".into(),
        }
    }

    /// Builds the use case under test.
    fn use_case(users: MockUserRepository, hasher: MockPasswordHasher) -> LoginUser {
        let mut tokens = MockTokenService::new();
        tokens.expect_issue().returning(|_| {
            Ok(IssuedToken {
                token: "jwt".into(),
                expires_at: now(),
            })
        });
        LoginUser::new(Arc::new(users), Arc::new(hasher), Arc::new(tokens))
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
            .execute(request("John@Example.com"))
            .await
            .unwrap();

        assert_eq!(response.token, "jwt");
        assert_eq!(response.user.username, "john");
    }

    #[tokio::test]
    async fn rejects_wrong_password() {
        let error = use_case(users_with_john(), hasher(false))
            .execute(request("john@example.com"))
            .await
            .unwrap_err();

        assert_eq!(error, AppError::Unauthorized(INVALID_CREDENTIALS.into()));
    }

    #[tokio::test]
    async fn rejects_unknown_email_with_the_same_error() {
        let error = use_case(users_with_john(), hasher(true))
            .execute(request("ghost@example.com"))
            .await
            .unwrap_err();

        assert_eq!(error, AppError::Unauthorized(INVALID_CREDENTIALS.into()));
    }
}
