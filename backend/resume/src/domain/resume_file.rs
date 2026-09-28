/// The resume document offered for download: its file name and its bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResumeFile {
    /// Name suggested to the browser when saving the file.
    pub filename: &'static str,
    /// Content of the PDF document.
    pub bytes: &'static [u8],
}
