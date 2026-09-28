//! Assembly of the Resume context: builds its adapter once and injects it
//! into its use case.

use std::sync::Arc;

use crate::application::ports::HumanVerifier;
use crate::application::use_cases::DownloadResume;
use crate::infrastructure::EmbeddedResumeStore;

/// Every use case of the Resume context, ready to be shared by the HTTP adapter.
pub struct ResumeModule {
    pub download: DownloadResume,
}

impl ResumeModule {
    /// Wires the embedded resume and the captcha verifier provided by the
    /// composition root into the use case.
    pub fn new(humans: Arc<dyn HumanVerifier>) -> Self {
        Self {
            download: DownloadResume::new(Arc::new(EmbeddedResumeStore), humans),
        }
    }
}
