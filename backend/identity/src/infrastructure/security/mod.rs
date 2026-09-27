//! Security adapters: password hashing, access tokens and captcha.

mod argon2_password_hasher;
mod jwt_token_service;
mod turnstile_human_verifier;

pub use argon2_password_hasher::Argon2PasswordHasher;
pub use jwt_token_service::JwtTokenService;
pub use turnstile_human_verifier::{TURNSTILE_SITEVERIFY_URL, TurnstileHumanVerifier};

use chrono::Duration;

/// Settings of the access tokens, read from the environment by the composition root.
#[derive(Debug, Clone)]
pub struct JwtSettings {
    /// HS256 shared secret.
    pub secret: String,
    /// Lifetime of an issued token.
    pub ttl: Duration,
}

/// Settings of the captcha, read from the environment by the composition root.
#[derive(Debug, Clone)]
pub struct CaptchaSettings {
    /// Cloudflare Turnstile secret key.
    pub turnstile_secret: String,
    /// `siteverify` endpoint ([`TURNSTILE_SITEVERIFY_URL`] in production, a fake server in tests).
    pub siteverify_url: String,
}
