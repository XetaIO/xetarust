use std::sync::Arc;

use xetaravel_kernel::{AppError, AppResult};

use crate::application::ports::CoverStorage;
use crate::domain::CoverImage;

/// Serves the bytes of a cover image (public).
pub struct GetCover {
    covers: Arc<dyn CoverStorage>,
}

impl GetCover {
    /// Builds the use case with its dependencies.
    pub fn new(covers: Arc<dyn CoverStorage>) -> Self {
        Self { covers }
    }

    /// Returns the bytes and the MIME type of the cover named `name`.
    /// Malformed names and missing files are both reported as `NotFound`.
    pub async fn execute(&self, name: &str) -> AppResult<(Vec<u8>, &'static str)> {
        let not_found = || AppError::NotFound("cover image not found".into());
        let cover = CoverImage::parse(name).map_err(|_| not_found())?;
        let bytes = self.covers.load(&cover).await?.ok_or_else(not_found)?;
        Ok((bytes, cover.format().mime_type()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::MockCoverStorage;
    use crate::domain::ImageFormat;

    /// Builds the use case over a storage answering `stored`.
    fn use_case(stored: Option<Vec<u8>>) -> GetCover {
        let mut covers = MockCoverStorage::new();
        covers.expect_load().returning(move |_| Ok(stored.clone()));
        GetCover::new(Arc::new(covers))
    }

    #[tokio::test]
    async fn returns_the_bytes_and_mime_type() {
        let cover = CoverImage::new(ImageFormat::Webp);

        let (bytes, mime) = use_case(Some(vec![1, 2, 3]))
            .execute(cover.as_str())
            .await
            .unwrap();

        assert_eq!(bytes, vec![1, 2, 3]);
        assert_eq!(mime, "image/webp");
    }

    #[tokio::test]
    async fn missing_file_is_not_found() {
        let cover = CoverImage::new(ImageFormat::Png);
        assert!(matches!(
            use_case(None).execute(cover.as_str()).await,
            Err(AppError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn malformed_name_is_not_found_without_touching_the_storage() {
        let mut covers = MockCoverStorage::new();
        covers.expect_load().never();

        assert!(matches!(
            GetCover::new(Arc::new(covers))
                .execute("../../etc/passwd")
                .await,
            Err(AppError::NotFound(_))
        ));
    }
}
