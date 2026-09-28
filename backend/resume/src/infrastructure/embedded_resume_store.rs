use crate::domain::{ResumeFile, ResumeStore};

/// Serves the PDF embedded in the binary at compile time: no file is read at
/// runtime, so the deployment needs neither a volume nor a copy step.
/// Updating the resume means committing the new file and redeploying.
pub struct EmbeddedResumeStore;

impl ResumeStore for EmbeddedResumeStore {
    /// Returns `assets/CV_Emeric_Fevre.pdf`.
    fn current(&self) -> ResumeFile {
        ResumeFile {
            filename: "CV_Emeric_Fevre.pdf",
            bytes: include_bytes!("../../assets/CV_Emeric_Fevre.pdf"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embeds_a_pdf_document() {
        let file = EmbeddedResumeStore.current();
        assert_eq!(file.filename, "CV_Emeric_Fevre.pdf");
        assert!(file.bytes.starts_with(b"%PDF"));
    }
}
