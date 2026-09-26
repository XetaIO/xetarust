use std::sync::Arc;

use chrono::Duration;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xetaravel_kernel::{AppError, AppResult, Clock};

use super::JwtSettings;
use crate::application::ports::{IssuedToken, TokenService};
use crate::domain::{User, UserId};

/// Payload of the access tokens.
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// User id.
    sub: Uuid,
    /// Role at issue time (informative only: the API reloads the user).
    role: String,
    /// Issued at (UNIX timestamp).
    iat: i64,
    /// Expiration (UNIX timestamp).
    exp: i64,
}

/// [`TokenService`] issuing HS256-signed JWTs.
pub struct JwtTokenService {
    encoding: EncodingKey,
    decoding: DecodingKey,
    ttl: Duration,
    clock: Arc<dyn Clock>,
}

impl JwtTokenService {
    /// Builds the service from the shared secret and the token lifetime.
    pub fn new(settings: &JwtSettings, clock: Arc<dyn Clock>) -> Self {
        Self {
            encoding: EncodingKey::from_secret(settings.secret.as_bytes()),
            decoding: DecodingKey::from_secret(settings.secret.as_bytes()),
            ttl: settings.ttl,
            clock,
        }
    }
}

impl TokenService for JwtTokenService {
    /// Signs a token carrying the user id and role.
    fn issue(&self, user: &User) -> AppResult<IssuedToken> {
        let now = self.clock.now();
        let expires_at = now + self.ttl;
        let claims = Claims {
            sub: user.id.as_uuid(),
            role: user.role.to_string(),
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
        };

        let token = encode(&Header::new(Algorithm::HS256), &claims, &self.encoding)
            .map_err(|e| AppError::Internal(format!("cannot sign token: {e}")))?;

        Ok(IssuedToken { token, expires_at })
    }

    /// Checks the signature and expiration, then returns the user id.
    fn verify(&self, token: &str) -> AppResult<UserId> {
        let data = decode::<Claims>(token, &self.decoding, &Validation::new(Algorithm::HS256))
            .map_err(|_| AppError::Unauthorized("invalid or expired token".into()))?;

        Ok(UserId::from(data.claims.sub))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use xetaravel_kernel::FixedClock;

    use super::*;
    use crate::domain::{Email, PasswordHash, Username};

    const SECRET: &str = "0123456789abcdef0123456789abcdef";

    /// Builds a service whose clock is shifted by `offset` from now.
    fn service(secret: &str, offset: Duration) -> JwtTokenService {
        let settings = JwtSettings {
            secret: secret.into(),
            ttl: Duration::hours(1),
        };
        JwtTokenService::new(&settings, Arc::new(FixedClock(Utc::now() + offset)))
    }

    /// Builds a user to issue tokens for.
    fn user() -> User {
        User::register(
            Username::parse("john").unwrap(),
            Email::parse("john@example.com").unwrap(),
            PasswordHash::new("hash"),
            Utc::now(),
        )
    }

    #[test]
    fn issued_tokens_can_be_verified() {
        let tokens = service(SECRET, Duration::zero());
        let user = user();

        let issued = tokens.issue(&user).unwrap();

        assert_eq!(tokens.verify(&issued.token).unwrap(), user.id);
    }

    #[test]
    fn rejects_tokens_signed_with_another_secret() {
        let issued = service("another-secret-another-secret-xx", Duration::zero())
            .issue(&user())
            .unwrap();
        assert!(matches!(
            service(SECRET, Duration::zero()).verify(&issued.token),
            Err(AppError::Unauthorized(_))
        ));
    }

    #[test]
    fn rejects_expired_tokens() {
        let issued = service(SECRET, Duration::hours(-3)).issue(&user()).unwrap();
        assert!(
            service(SECRET, Duration::zero())
                .verify(&issued.token)
                .is_err()
        );
    }

    #[test]
    fn rejects_garbage() {
        assert!(
            service(SECRET, Duration::zero())
                .verify("not.a.jwt")
                .is_err()
        );
    }
}
