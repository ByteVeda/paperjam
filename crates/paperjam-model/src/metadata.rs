/// PDF document metadata.
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    pub creation_date: Option<String>,
    pub modification_date: Option<String>,
    pub pdf_version: String,
    pub page_count: usize,
    pub is_encrypted: bool,
    pub xmp_metadata: Option<String>,
}

/// Fields to update in the PDF /Info dictionary.
///
/// Each field uses `Option<Option<String>>`:
/// - `None` — leave unchanged
/// - `Some(None)` — remove the field
/// - `Some(Some(value))` — set the field to `value`
#[derive(Debug, Clone, Default)]
pub struct MetadataUpdate {
    pub title: Option<Option<String>>,
    pub author: Option<Option<String>>,
    pub subject: Option<Option<String>>,
    pub keywords: Option<Option<String>>,
    pub creator: Option<Option<String>>,
    pub producer: Option<Option<String>>,
}
