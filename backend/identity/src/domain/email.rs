use std::fmt;

use xetaravel_kernel::{DomainError, DomainResult};

/// Maximum length of an email address (RFC 5321).
const MAX_LENGTH: usize = 254;

/// A syntactically valid, lower-cased email address.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(String);

impl Email {
    /// Parses and normalizes (trim + lowercase) an email address.
    pub fn parse(raw: &str) -> DomainResult<Self> {
        let email = raw.trim().to_lowercase();

        if email.is_empty() || email.len() > MAX_LENGTH {
            return Err(DomainError::validation(
                "email",
                "must contain between 1 and 254 characters",
            ));
        }
        if !Self::has_valid_shape(&email) {
            return Err(DomainError::validation("email", "is not a valid address"));
        }

        Ok(Self(email))
    }

    /// Returns the address as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Checks the `local@domain.tld` shape without whitespace.
    fn has_valid_shape(email: &str) -> bool {
        let Some((local, domain)) = email.split_once('@') else {
            return false;
        };

        !local.is_empty()
            && !domain.contains('@')
            && !email.chars().any(char::is_whitespace)
            && domain
                .split_once('.')
                .is_some_and(|(name, tld)| !name.is_empty() && !tld.is_empty())
            && !domain.ends_with('.')
    }
}

impl fmt::Display for Email {
    /// Formats the address as-is.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_valid_addresses() {
        let email = Email::parse("  Emeric@Xetaravel.COM ").unwrap();
        assert_eq!(email.as_str(), "emeric@xetaravel.com");
    }

    #[test]
    fn rejects_invalid_addresses() {
        for raw in [
            "",
            "   ",
            "no-at-sign",
            "@domain.com",
            "user@",
            "user@domain",
            "user@domain.",
            "user@.com",
            "us er@domain.com",
            "a@b@c.com",
        ] {
            assert!(Email::parse(raw).is_err(), "{raw:?} should be rejected");
        }
    }

    #[test]
    fn rejects_too_long_addresses() {
        let raw = format!("{}@example.com", "a".repeat(250));
        assert!(Email::parse(&raw).is_err());
    }
}
