//! Business concepts of the Resume context: the [`ResumeFile`] offered to
//! visitors and the [`ResumeStore`] port that provides it.
//!
//! Framework-free: no Axum, no serde.

mod resume_file;
mod resume_store;

pub use resume_file::ResumeFile;
pub use resume_store::ResumeStore;

#[cfg(test)]
pub use resume_store::MockResumeStore;
