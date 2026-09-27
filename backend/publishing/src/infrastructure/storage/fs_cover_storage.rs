//! [`CoverStorage`] adapter keeping the cover images on the local disk.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use xetaravel_kernel::{AppError, AppResult};

use crate::application::ports::CoverStorage;
use crate::domain::CoverImage;

/// Stores every cover image as a file named after the [`CoverImage`] in a
/// single directory. The name is validated by the value object, so it can
/// never escape `root`.
pub struct FsCoverStorage {
    root: PathBuf,
}

impl FsCoverStorage {
    /// Builds the adapter over `root` (created on the first write).
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the path of the file holding `cover`.
    fn path(&self, cover: &CoverImage) -> PathBuf {
        self.root.join(cover.as_str())
    }
}

/// Translates an I/O failure on `path` into an internal error.
fn io_error(path: &Path, error: std::io::Error) -> AppError {
    AppError::Internal(format!("cover storage {}: {error}", path.display()))
}

#[async_trait]
impl CoverStorage for FsCoverStorage {
    /// Writes the file, creating the directory when needed.
    async fn store(&self, cover: &CoverImage, bytes: &[u8]) -> AppResult<()> {
        tokio::fs::create_dir_all(&self.root)
            .await
            .map_err(|e| io_error(&self.root, e))?;
        let path = self.path(cover);
        tokio::fs::write(&path, bytes)
            .await
            .map_err(|e| io_error(&path, e))
    }

    /// Reads the file; a missing file gives `None`.
    async fn load(&self, cover: &CoverImage) -> AppResult<Option<Vec<u8>>> {
        let path = self.path(cover);
        match tokio::fs::read(&path).await {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(io_error(&path, error)),
        }
    }

    /// Removes the file; a missing file is ignored.
    async fn delete(&self, cover: &CoverImage) -> AppResult<()> {
        let path = self.path(cover);
        match tokio::fs::remove_file(&path).await {
            Err(error) if error.kind() != ErrorKind::NotFound => Err(io_error(&path, error)),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ImageFormat;

    #[tokio::test]
    async fn stores_loads_and_deletes_files() {
        let dir = tempfile::tempdir().unwrap();
        let storage = FsCoverStorage::new(dir.path().join("covers"));
        let cover = CoverImage::new(ImageFormat::Png);

        assert_eq!(storage.load(&cover).await.unwrap(), None);

        storage.store(&cover, b"bytes").await.unwrap();
        assert!(dir.path().join("covers").join(cover.as_str()).is_file());
        assert_eq!(storage.load(&cover).await.unwrap(), Some(b"bytes".to_vec()));

        storage.delete(&cover).await.unwrap();
        assert_eq!(storage.load(&cover).await.unwrap(), None);
        storage.delete(&cover).await.unwrap();
    }
}
