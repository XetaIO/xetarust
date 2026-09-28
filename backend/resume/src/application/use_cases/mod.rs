//! Use cases of the Resume context. Each use case is a struct holding its
//! dependencies as `Arc<dyn Port>` and exposing a single `execute` method.

mod download_resume;

pub use download_resume::DownloadResume;
