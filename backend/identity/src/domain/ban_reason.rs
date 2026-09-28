use std::fmt;

use xetaravel_kernel::DomainResult;
use xetaravel_kernel::text::validate_text;

/// Maximum number of characters of a ban reason.
pub const MAX_LENGTH: usize = 255;

/// The reason an admin gave when banning an account (trimmed, never blank).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BanReason(String);

impl BanReason {
    /// Parses a ban reason, trimming surrounding whitespace.
    pub fn parse(raw: &str) -> DomainResult<Self> {
        validate_text("reason", raw, 1, MAX_LENGTH).map(Self)
    }

    /// Returns the reason as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BanReason {
    /// Formats the reason as-is.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_and_trims_a_reason() {
        assert_eq!(BanReason::parse("  spam ").unwrap().as_str(), "spam");
        assert!(BanReason::parse(&"a".repeat(MAX_LENGTH)).is_ok());
    }

    #[test]
    fn rejects_blank_or_too_long_reasons() {
        assert!(BanReason::parse("   ").is_err());
        assert!(BanReason::parse(&"a".repeat(MAX_LENGTH + 1)).is_err());
    }
}
