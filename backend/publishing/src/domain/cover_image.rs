//! Cover image of an article: accepted formats and the value object naming
//! the stored file.

use std::fmt;

use uuid::Uuid;
use xetaravel_kernel::{DomainError, DomainResult};

/// Maximum size of a cover image, in bytes (5 MiB).
pub const COVER_MAX_BYTES: usize = 5 * 1024 * 1024;

/// Name of the field reported in validation errors.
const FIELD: &str = "cover";

/// Image formats accepted as article covers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Webp,
}

impl ImageFormat {
    /// Detects the format of an uploaded file from its magic bytes (the
    /// client-provided name or MIME type are never trusted). Empty, oversized
    /// or non-image files are rejected.
    pub fn sniff(bytes: &[u8]) -> DomainResult<Self> {
        if bytes.is_empty() {
            return Err(DomainError::validation(FIELD, "is required"));
        }
        if bytes.len() > COVER_MAX_BYTES {
            return Err(DomainError::validation(
                FIELD,
                format!("must not exceed {} MB", COVER_MAX_BYTES / (1024 * 1024)),
            ));
        }

        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            Ok(Self::Jpeg)
        } else if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
            Ok(Self::Png)
        } else if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
            Ok(Self::Webp)
        } else {
            Err(DomainError::validation(
                FIELD,
                "must be a JPEG, PNG or WebP image",
            ))
        }
    }

    /// Returns the file extension of the format (without the dot).
    pub fn extension(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Webp => "webp",
        }
    }

    /// Returns the MIME type of the format.
    pub fn mime_type(self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Webp => "image/webp",
        }
    }

    /// Finds the format matching a file extension.
    fn from_extension(extension: &str) -> Option<Self> {
        [Self::Jpeg, Self::Png, Self::Webp]
            .into_iter()
            .find(|format| format.extension() == extension)
    }
}

/// File name of a stored cover image: `<uuid v7>.<extension>`.
///
/// A new name is generated on every upload, so a given name always refers to
/// the same bytes (safe to cache forever). Parsing only accepts this exact
/// shape, which rules out any path traversal.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CoverImage(String);

impl CoverImage {
    /// Generates a fresh, unique file name for an image of `format`.
    pub fn new(format: ImageFormat) -> Self {
        Self(format!("{}.{}", Uuid::now_v7(), format.extension()))
    }

    /// Parses a stored or requested file name, rejecting anything that is not
    /// a lowercase hyphenated UUID followed by a supported extension.
    pub fn parse(raw: &str) -> DomainResult<Self> {
        let invalid = || DomainError::validation(FIELD, "is not a valid cover image name");
        let (stem, extension) = raw.split_once('.').ok_or_else(invalid)?;
        let uuid = Uuid::try_parse(stem).map_err(|_| invalid())?;
        if uuid.hyphenated().to_string() != stem || ImageFormat::from_extension(extension).is_none()
        {
            return Err(invalid());
        }
        Ok(Self(raw.to_owned()))
    }

    /// Returns the image format, deduced from the extension.
    pub fn format(&self) -> ImageFormat {
        self.0
            .rsplit_once('.')
            .and_then(|(_, extension)| ImageFormat::from_extension(extension))
            .unwrap_or(ImageFormat::Jpeg)
    }

    /// Returns the file name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CoverImage {
    /// Formats the file name as-is.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal headers of each supported format.
    const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0, 0x10];
    const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0];
    const WEBP: &[u8] = b"RIFF\x24\0\0\0WEBPVP8 ";

    #[test]
    fn sniffs_supported_formats() {
        assert_eq!(ImageFormat::sniff(JPEG).unwrap(), ImageFormat::Jpeg);
        assert_eq!(ImageFormat::sniff(PNG).unwrap(), ImageFormat::Png);
        assert_eq!(ImageFormat::sniff(WEBP).unwrap(), ImageFormat::Webp);
    }

    #[test]
    fn rejects_empty_unknown_and_oversized_files() {
        assert!(ImageFormat::sniff(b"").is_err());
        assert!(ImageFormat::sniff(b"hello, I am a text file").is_err());
        assert!(ImageFormat::sniff(b"RIFF\0\0\0\0WAVE").is_err());

        let mut huge = JPEG.to_vec();
        huge.resize(COVER_MAX_BYTES + 1, 0);
        assert_eq!(
            ImageFormat::sniff(&huge).unwrap_err(),
            DomainError::validation("cover", "must not exceed 5 MB")
        );
    }

    #[test]
    fn exposes_extension_and_mime_type() {
        assert_eq!(ImageFormat::Webp.extension(), "webp");
        assert_eq!(ImageFormat::Png.mime_type(), "image/png");
    }

    #[test]
    fn generates_unique_parsable_names() {
        let first = CoverImage::new(ImageFormat::Png);
        let second = CoverImage::new(ImageFormat::Png);

        assert_ne!(first, second);
        assert!(first.as_str().ends_with(".png"));
        assert_eq!(CoverImage::parse(first.as_str()).unwrap(), first);
        assert_eq!(first.format(), ImageFormat::Png);
    }

    #[test]
    fn parses_canonical_names_only() {
        let uuid = Uuid::now_v7();
        assert!(CoverImage::parse(&format!("{uuid}.webp")).is_ok());
        for raw in [
            String::new(),
            "../etc/passwd".into(),
            format!("../{uuid}.png"),
            format!("{uuid}.png/.."),
            format!("{uuid}.gif"),
            format!("{uuid}.PNG"),
            format!("{uuid}"),
            format!("{}.png", uuid.simple()),
            format!("{}.png", uuid.hyphenated().to_string().to_uppercase()),
        ] {
            assert!(
                CoverImage::parse(&raw).is_err(),
                "{raw:?} should be rejected"
            );
        }
    }
}
