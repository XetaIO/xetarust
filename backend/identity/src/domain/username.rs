use std::fmt;

use xetaravel_kernel::{DomainError, DomainResult};

/// Minimum number of characters of a username.
pub const MIN_LENGTH: usize = 3;
/// Maximum number of characters of a username.
pub const MAX_LENGTH: usize = 30;

/// A public user name made of ASCII letters, digits, `_`, `-` and `.`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Username(String);

impl Username {
    /// Parses a username, trimming surrounding whitespace.
    pub fn parse(raw: &str) -> DomainResult<Self> {
        let username = raw.trim();
        let length = username.chars().count();

        if !(MIN_LENGTH..=MAX_LENGTH).contains(&length) {
            return Err(DomainError::validation(
                "username",
                format!("must contain between {MIN_LENGTH} and {MAX_LENGTH} characters"),
            ));
        }
        if !username.chars().all(Self::is_allowed_char) {
            return Err(DomainError::validation(
                "username",
                "may only contain letters, digits, '_', '-' and '.'",
            ));
        }

        Ok(Self(username.to_owned()))
    }

    /// Returns the username as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Tells whether a character is allowed inside a username.
    fn is_allowed_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')
    }
}

impl fmt::Display for Username {
    /// Formats the username as-is.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_and_trims_valid_usernames() {
        assert_eq!(Username::parse(" Xety ").unwrap().as_str(), "Xety");
        assert!(Username::parse("john.doe-42_x").is_ok());
    }

    #[test]
    fn rejects_bad_lengths() {
        assert!(Username::parse("ab").is_err());
        assert!(Username::parse(&"a".repeat(31)).is_err());
    }

    #[test]
    fn rejects_forbidden_characters() {
        assert!(Username::parse("john doe").is_err());
        assert!(Username::parse("john@doe").is_err());
        assert!(Username::parse("jöhn").is_err());
    }
}
