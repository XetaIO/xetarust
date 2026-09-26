use std::fmt;

/// An already hashed password (PHC string). The domain never sees plain
/// passwords: hashing is delegated to the `PasswordHasher` port.
#[derive(Clone, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    /// Wraps a hash produced by a password hasher or loaded from storage.
    pub fn new(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    /// Returns the PHC string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PasswordHash {
    /// Redacts the hash so it never leaks into logs.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PasswordHash(***)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_is_redacted() {
        let hash = PasswordHash::new("$argon2id$secret");
        assert_eq!(format!("{hash:?}"), "PasswordHash(***)");
        assert_eq!(hash.as_str(), "$argon2id$secret");
    }
}
