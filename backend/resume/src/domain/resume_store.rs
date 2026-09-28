use crate::domain::ResumeFile;

/// Provides the current version of the resume.
#[cfg_attr(test, mockall::automock)]
pub trait ResumeStore: Send + Sync {
    /// Returns the resume currently offered for download.
    fn current(&self) -> ResumeFile;
}
