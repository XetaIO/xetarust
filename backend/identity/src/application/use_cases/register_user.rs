use std::sync::Arc;

use validator::Validate;
use xetaravel_kernel::{AppError, AppResult, Clock};

use super::authenticated_response;
use crate::application::dto::{AuthResponse, RegisterRequest};
use crate::application::ports::{PasswordHasher, TokenService};
use crate::domain::{Email, User, UserRepository, Username};

/// Creates a member account and logs it in.
pub struct RegisterUser {
    users: Arc<dyn UserRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenService>,
    clock: Arc<dyn Clock>,
}

impl RegisterUser {
    /// Builds the use case with its dependencies.
    pub fn new(
        users: Arc<dyn UserRepository>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenService>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            users,
            hasher,
            tokens,
            clock,
        }
    }

    /// Validates the input, ensures email and username are free, stores the
    /// new member and returns an access token.
    pub async fn execute(&self, input: RegisterRequest) -> AppResult<AuthResponse> {
        input.validate()?;
        let username = Username::parse(&input.username)?;
        let email = Email::parse(&input.email)?;

        if self.users.email_exists(&email).await? {
            return Err(AppError::field("email", "is already taken"));
        }
        if self.users.username_exists(&username).await? {
            return Err(AppError::field("username", "is already taken"));
        }

        let password_hash = self.hasher.hash(&input.password).await?;
        let user = User::register(username, email, password_hash, self.clock.now());
        self.users.create(&user).await?;

        authenticated_response(self.tokens.as_ref(), &user)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use super::*;
    use crate::application::dto::RoleDto;
    use crate::application::ports::{IssuedToken, MockPasswordHasher, MockTokenService};
    use crate::application::test_support::{clock, now};
    use crate::domain::{MockUserRepository, PasswordHash, Role};

    /// Returns a valid registration request.
    fn request() -> RegisterRequest {
        RegisterRequest {
            username: "Xety".into(),
            email: "Emeric@Xetaravel.com".into(),
            password: "super-secret".into(),
        }
    }

    /// Returns a token service mock issuing a fixed token.
    fn tokens() -> MockTokenService {
        let mut tokens = MockTokenService::new();
        tokens.expect_issue().returning(|_| {
            Ok(IssuedToken {
                token: "jwt".into(),
                expires_at: now(),
            })
        });
        tokens
    }

    /// Builds the use case under test.
    fn use_case(users: MockUserRepository, hasher: MockPasswordHasher) -> RegisterUser {
        RegisterUser::new(
            Arc::new(users),
            Arc::new(hasher),
            Arc::new(tokens()),
            clock(),
        )
    }

    #[tokio::test]
    async fn registers_a_member_and_returns_a_token() {
        let mut users = MockUserRepository::new();
        users.expect_email_exists().returning(|_| Ok(false));
        users.expect_username_exists().returning(|_| Ok(false));
        users
            .expect_create()
            .withf(|user| {
                user.role == Role::Member
                    && user.email.as_str() == "emeric@xetaravel.com"
                    && user.password_hash.as_str() == "hashed:super-secret"
            })
            .times(1)
            .returning(|_| Ok(()));
        let mut hasher = MockPasswordHasher::new();
        hasher
            .expect_hash()
            .with(eq("super-secret"))
            .returning(|plain| Ok(PasswordHash::new(format!("hashed:{plain}"))));

        let response = use_case(users, hasher).execute(request()).await.unwrap();

        assert_eq!(response.token, "jwt");
        assert_eq!(response.user.username, "Xety");
        assert_eq!(response.user.role, RoleDto::Member);
    }

    #[tokio::test]
    async fn rejects_invalid_input_before_touching_storage() {
        let input = RegisterRequest {
            username: "x".into(),
            email: "nope".into(),
            password: "short".into(),
        };

        let error = use_case(MockUserRepository::new(), MockPasswordHasher::new())
            .execute(input)
            .await
            .unwrap_err();

        let AppError::Validation(fields) = error else {
            panic!("expected a validation error");
        };
        assert!(fields.contains_key("username"));
        assert!(fields.contains_key("email"));
        assert!(fields.contains_key("password"));
    }

    #[tokio::test]
    async fn rejects_taken_email() {
        let mut users = MockUserRepository::new();
        users.expect_email_exists().returning(|_| Ok(true));

        let error = use_case(users, MockPasswordHasher::new())
            .execute(request())
            .await
            .unwrap_err();

        assert_eq!(error, AppError::field("email", "is already taken"));
    }

    #[tokio::test]
    async fn rejects_taken_username() {
        let mut users = MockUserRepository::new();
        users.expect_email_exists().returning(|_| Ok(false));
        users.expect_username_exists().returning(|_| Ok(true));

        let error = use_case(users, MockPasswordHasher::new())
            .execute(request())
            .await
            .unwrap_err();

        assert_eq!(error, AppError::field("username", "is already taken"));
    }
}
