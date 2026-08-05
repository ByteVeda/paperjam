use std::collections::HashMap;
use std::path::PathBuf;

use paperjam_model::format::DocumentFormat;
use paperjam_model::metadata::Metadata;

/// A document session — an open document with its raw bytes and metadata.
#[derive(Debug)]
pub struct DocumentSession {
    pub id: String,
    pub bytes: Vec<u8>,
    pub format: DocumentFormat,
    pub path: Option<PathBuf>,
    pub metadata: Option<Metadata>,
}

/// Manages open document sessions.
#[derive(Debug, Default)]
pub struct SessionManager {
    sessions: HashMap<String, DocumentSession>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Open a document from a file path and return its session ID.
    pub fn open_from_path(
        &mut self,
        path: &std::path::Path,
    ) -> Result<String, crate::error::McpError> {
        let bytes = std::fs::read(path)?;
        let format = paperjam_convert::detect_format(path);
        let id = uuid::Uuid::new_v4().to_string();

        // Extract metadata.
        let metadata = paperjam_convert::extract::extract(&bytes, format)
            .ok()
            .map(|doc| doc.metadata);

        self.sessions.insert(
            id.clone(),
            DocumentSession {
                id: id.clone(),
                bytes,
                format,
                path: Some(path.to_path_buf()),
                metadata,
            },
        );

        Ok(id)
    }

    /// Open a document from bytes with a known format.
    pub fn open_from_bytes(&mut self, bytes: Vec<u8>, format: DocumentFormat) -> String {
        let id = uuid::Uuid::new_v4().to_string();

        let metadata = paperjam_convert::extract::extract(&bytes, format)
            .ok()
            .map(|doc| doc.metadata);

        self.sessions.insert(
            id.clone(),
            DocumentSession {
                id: id.clone(),
                bytes,
                format,
                path: None,
                metadata,
            },
        );

        id
    }

    /// Get a session by ID.
    pub fn get(&self, id: &str) -> Option<&DocumentSession> {
        self.sessions.get(id)
    }

    /// Get a mutable session by ID.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut DocumentSession> {
        self.sessions.get_mut(id)
    }

    /// Close (remove) a session.
    pub fn close(&mut self, id: &str) -> bool {
        self.sessions.remove(id).is_some()
    }

    /// List all open session IDs.
    pub fn list_sessions(&self) -> Vec<&str> {
        self.sessions.keys().map(|s| s.as_str()).collect()
    }
}
