use argon2::Argon2;
use argon2::password_hash::phc::PasswordHash as PhcHash;
use argon2::password_hash::{PasswordHasher as _, PasswordVerifier as _};
use async_trait::async_trait;
use xetaravel_kernel::{AppError, AppResult};

use crate::application::ports::PasswordHasher;
use crate::domain::PasswordHash;

/// [`PasswordHasher`] using Argon2id with the recommended default parameters.
/// Hashing runs on the blocking thread pool so it never stalls the async runtime.
#[derive(Debug, Default, Clone, Copy)]
pub struct Argon2PasswordHasher;

#[async_trait]
impl PasswordHasher for Argon2PasswordHasher {
    /// Hashes the password into a PHC string with a random salt.
    async fn hash(&self, plain: &str) -> AppResult<PasswordHash> {
        let plain = plain.to_owned();
        let phc = tokio::task::spawn_blocking(move || {
            Argon2::default()
                .hash_password(plain.as_bytes())
                .map(|hash| hash.to_string())
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
        .map_err(|e| AppError::Internal(format!("password hashing failed: {e}")))?;

        Ok(PasswordHash::new(phc))
    }

    /// Verifies the password against the stored PHC string.
    async fn verify(&self, plain: &str, hash: &PasswordHash) -> AppResult<bool> {
        let plain = plain.to_owned();
        let stored = hash.as_str().to_owned();

        tokio::task::spawn_blocking(move || {
            let parsed = PhcHash::new(&stored)
                .map_err(|e| AppError::Internal(format!("stored hash is invalid: {e}")))?;
            Ok(Argon2::default()
                .verify_password(plain.as_bytes(), &parsed)
                .is_ok())
        })
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hashes_and_verifies_passwords() {
        let hasher = Argon2PasswordHasher;
        let hash = hasher.hash("super-secret").await.unwrap();

        assert!(hash.as_str().starts_with("$argon2id$"));
        assert!(hasher.verify("super-secret", &hash).await.unwrap());
        assert!(!hasher.verify("wrong", &hash).await.unwrap());
    }

    #[tokio::test]
    async fn uses_a_random_salt() {
        let hasher = Argon2PasswordHasher;
        let first = hasher.hash("same").await.unwrap();
        let second = hasher.hash("same").await.unwrap();
        assert_ne!(first, second);
    }
}
