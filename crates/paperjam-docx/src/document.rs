use crate::error::DocxError;

/// A parsed DOCX document backed by `docx-rs`.
pub struct DocxDocument {
    pub(crate) inner: docx_rs::Docx,
    pub(crate) raw_bytes: Vec<u8>,
}

impl DocxDocument {
    /// Open a DOCX document from raw bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DocxError> {
        let inner = docx_rs::read_docx(bytes).map_err(|e| DocxError::Parse(format!("{}", e)))?;
        Ok(Self {
            inner,
            raw_bytes: bytes.to_vec(),
        })
    }
}
