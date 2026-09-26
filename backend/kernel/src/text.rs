//! Free-text validation helpers used by the entities of every context.

use crate::error::{DomainError, DomainResult};

/// Trims `value` and ensures its length (in characters) is within `min..=max`.
pub fn validate_text(
    field: &'static str,
    value: &str,
    min: usize,
    max: usize,
) -> DomainResult<String> {
    let trimmed = value.trim();
    let length = trimmed.chars().count();

    if (min..=max).contains(&length) {
        Ok(trimmed.to_owned())
    } else {
        Err(DomainError::validation(
            field,
            format!("must contain between {min} and {max} characters"),
        ))
    }
}

/// Trims an optional text, turning blank values into `None`, and checks its maximum length.
pub fn validate_optional_text(
    field: &'static str,
    value: Option<&str>,
    max: usize,
) -> DomainResult<Option<String>> {
    match non_blank(value) {
        Some(v) => validate_text(field, v, 1, max).map(Some),
        None => Ok(None),
    }
}

/// Turns an optional, possibly blank string into `None` when blank (trimmed otherwise).
pub fn non_blank(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_text_trims_and_checks_bounds() {
        assert_eq!(validate_text("f", "  abc ", 1, 3).unwrap(), "abc");
        assert!(validate_text("f", "   ", 1, 3).is_err());
        assert!(validate_text("f", "abcd", 1, 3).is_err());
    }

    #[test]
    fn validate_optional_text_turns_blank_into_none() {
        assert_eq!(validate_optional_text("f", Some("  "), 5).unwrap(), None);
        assert_eq!(validate_optional_text("f", None, 5).unwrap(), None);
        assert_eq!(
            validate_optional_text("f", Some(" ok "), 5).unwrap(),
            Some("ok".to_owned())
        );
        assert!(validate_optional_text("f", Some("toolong"), 5).is_err());
    }

    #[test]
    fn non_blank_filters_blank_values() {
        assert_eq!(non_blank(Some("  ")), None);
        assert_eq!(non_blank(Some(" a ")), Some("a"));
        assert_eq!(non_blank(None), None);
    }
}
