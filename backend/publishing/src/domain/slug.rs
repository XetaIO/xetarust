use std::fmt;

use xetaravel_kernel::{DomainError, DomainResult};

/// Maximum length of a slug.
const MAX_LENGTH: usize = 200;

/// A URL-friendly identifier: lowercase ASCII letters, digits and single dashes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slug(String);

impl Slug {
    /// Builds a slug from any human text (e.g. "Hello Rust!" -> "hello-rust").
    pub fn from_text(text: &str) -> DomainResult<Self> {
        Self::parse(&slug::slugify(text))
    }

    /// Parses an already slugified value, rejecting anything not canonical.
    pub fn parse(raw: &str) -> DomainResult<Self> {
        let is_canonical = !raw.is_empty()
            && raw.len() <= MAX_LENGTH
            && raw
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !raw.starts_with('-')
            && !raw.ends_with('-')
            && !raw.contains("--");

        if !is_canonical {
            return Err(DomainError::validation(
                "slug",
                "must only contain lowercase letters, digits and single dashes",
            ));
        }

        Ok(Self(raw.to_owned()))
    }

    /// Returns the slug as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Slug {
    /// Formats the slug as-is.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugifies_human_text() {
        assert_eq!(
            Slug::from_text("  Écrire une API en Rust !")
                .unwrap()
                .as_str(),
            "ecrire-une-api-en-rust"
        );
    }

    #[test]
    fn rejects_text_without_slug_characters() {
        assert!(Slug::from_text("!!!").is_err());
    }

    #[test]
    fn parses_canonical_slugs_only() {
        assert!(Slug::parse("rust-2024").is_ok());
        for raw in [
            "",
            "Rust",
            "rust 2024",
            "-rust",
            "rust-",
            "rust--2024",
            "rüst",
        ] {
            assert!(Slug::parse(raw).is_err(), "{raw:?} should be rejected");
        }
    }
}
