pub mod error;
pub mod session;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::schemars;
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::McpError;
use crate::session::SessionManager;

/// Configuration for the MCP server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Root directory for relative paths. Also acts as the sandbox root unless
    /// `allow_absolute_paths` is set.
    pub working_dir: PathBuf,
    /// If true, disable sandbox containment and allow paths that point outside
    /// `working_dir`. Default: false.
    pub allow_absolute_paths: bool,
}

/// The paperjam MCP server.
pub struct PaperjamServer {
    sessions: Arc<Mutex<SessionManager>>,
    working_dir: PathBuf,
    working_dir_canonical: Option<PathBuf>,
    allow_absolute_paths: bool,
    tool_router: ToolRouter<Self>,
}

impl PaperjamServer {
    /// Construct a server rooted at `working_dir` with sandboxing enabled.
    pub fn new(working_dir: PathBuf) -> Self {
        Self::with_config(ServerConfig {
            working_dir,
            allow_absolute_paths: false,
        })
    }

    pub fn with_config(config: ServerConfig) -> Self {
        let working_dir_canonical = config.working_dir.canonicalize().ok();
        Self {
            sessions: Arc::new(Mutex::new(SessionManager::new())),
            working_dir: config.working_dir,
            working_dir_canonical,
            allow_absolute_paths: config.allow_absolute_paths,
            tool_router: Self::tool_router(),
        }
    }

    /// Resolve a user-supplied path against the server's working directory.
    ///
    /// Unless `allow_absolute_paths` is set, the result must be contained
    /// within `working_dir`. A non-existent target (common for save
    /// operations) is accepted if its nearest existing ancestor is within
    /// the sandbox.
    fn resolve_path(&self, path: &str) -> Result<PathBuf, McpError> {
        let candidate = {
            let p = PathBuf::from(path);
            if p.is_absolute() {
                p
            } else {
                self.working_dir.join(p)
            }
        };

        if self.allow_absolute_paths {
            return Ok(candidate);
        }

        let root = self
            .working_dir_canonical
            .as_deref()
            .unwrap_or(self.working_dir.as_path());

        // Canonicalize as much of the candidate as exists on disk, then
        // append any non-existent tail so that `..` components are resolved
        // before the containment check.
        let checked = canonicalize_with_fallback(&candidate)
            .ok_or_else(|| McpError::PathEscapesSandbox(path.to_string()))?;

        if !checked.starts_with(root) {
            return Err(McpError::PathEscapesSandbox(path.to_string()));
        }

        Ok(checked)
    }
}

/// Walk up `path` until an existing ancestor is found, canonicalize it, then
/// re-append the non-existent tail. Returns `None` if no ancestor exists.
fn canonicalize_with_fallback(path: &Path) -> Option<PathBuf> {
    if let Ok(c) = path.canonicalize() {
        return Some(c);
    }
    let mut tail: Vec<&std::ffi::OsStr> = Vec::new();
    let mut cur = path;
    loop {
        if let Some(name) = cur.file_name() {
            tail.push(name);
        }
        match cur.parent() {
            Some(parent) => {
                if let Ok(canon) = parent.canonicalize() {
                    let mut out = canon;
                    for name in tail.iter().rev() {
                        out.push(name);
                    }
                    return Some(out);
                }
                cur = parent;
                if cur.as_os_str().is_empty() {
                    return None;
                }
            }
            None => return None,
        }
    }
}

// --- Tool parameter types ---

#[derive(Deserialize, JsonSchema)]
struct OpenDocumentParams {
    /// File path to open (relative to working directory or absolute).
    path: String,
}

#[derive(Deserialize, JsonSchema)]
struct SessionIdParams {
    /// Session ID of the open document.
    session_id: String,
}

#[derive(Deserialize, JsonSchema)]
struct ConvertDocumentParams {
    /// Session ID of the source document.
    session_id: String,
    /// Target format (e.g., "pdf", "docx", "html", "epub", "xlsx", "pptx", "markdown").
    target_format: String,
}

#[derive(Deserialize, JsonSchema)]
struct RedactTextParams {
    /// Session ID of the document.
    session_id: String,
    /// Text pattern to redact.
    pattern: String,
    /// Whether the match is case-sensitive. Default: false.
    #[serde(default)]
    case_sensitive: bool,
}

#[derive(Deserialize, JsonSchema)]
struct WatermarkParams {
    /// Session ID of the document.
    session_id: String,
    /// Watermark text.
    text: String,
    /// Font size. Default: 60.
    #[serde(default)]
    font_size: Option<f64>,
    /// Opacity (0.0-1.0). Default: 0.3.
    #[serde(default)]
    opacity: Option<f64>,
}

#[derive(Deserialize, JsonSchema)]
struct EncryptParams {
    /// Session ID of the document.
    session_id: String,
    /// User password.
    password: String,
    /// Encryption algorithm ("aes128", "aes256", "rc4"). Default: "aes128".
    #[serde(default)]
    algorithm: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
struct SaveDocumentParams {
    /// Session ID of the document.
    session_id: String,
    /// Output file path.
    output_path: String,
}

#[derive(Deserialize, JsonSchema)]
struct RunPipelineParams {
    /// Pipeline definition as YAML string.
    yaml: String,
}

fn text_content(text: impl Into<String>) -> String {
    text.into()
}

fn json_content(value: &impl Serialize) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|e| format!("Error: {}", e))
}

// --- Tool implementations ---

#[tool_router]
impl PaperjamServer {
    #[tool(
        description = "Open a document from a file path. Supports PDF, DOCX, XLSX, PPTX, HTML, EPUB. Returns a session ID for subsequent operations."
    )]
    async fn open_document(&self, params: Parameters<OpenDocumentParams>) -> String {
        let path = match self.resolve_path(&params.0.path) {
            Ok(p) => p,
            Err(e) => return format!("Error: {}", e),
        };
        match self.sessions.lock().unwrap().open_from_path(&path) {
            Ok(session_id) => {
                let sessions = self.sessions.lock().unwrap();
                let session = sessions.get(&session_id).unwrap();
                let info = serde_json::json!({
                    "session_id": session_id,
                    "format": session.format.display_name(),
                    "path": path.display().to_string(),
                });
                json_content(&info)
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Get document info including format and metadata.")]
    async fn get_document_info(&self, params: Parameters<SessionIdParams>) -> String {
        let sessions = self.sessions.lock().unwrap();
        match sessions.get(&params.0.session_id) {
            Some(session) => {
                let meta = session.metadata.as_ref();
                let info = serde_json::json!({
                    "session_id": session.id,
                    "format": session.format.display_name(),
                    "title": meta.and_then(|m| m.title.as_deref()),
                    "author": meta.and_then(|m| m.author.as_deref()),
                    "page_count": meta.map(|m| m.page_count),
                });
                json_content(&info)
            }
            None => format!("Error: session not found: {}", params.0.session_id),
        }
    }

    #[tool(description = "Close an open document session and free resources.")]
    async fn close_document(&self, params: Parameters<SessionIdParams>) -> String {
        let closed = self.sessions.lock().unwrap().close(&params.0.session_id);
        json_content(&serde_json::json!({ "success": closed }))
    }

    #[tool(description = "Extract plain text from a document.")]
    async fn extract_text(&self, params: Parameters<SessionIdParams>) -> String {
        let sessions = self.sessions.lock().unwrap();
        let session = match sessions.get(&params.0.session_id) {
            Some(s) => s,
            None => return format!("Error: session not found: {}", params.0.session_id),
        };

        match paperjam_convert::extract::extract(&session.bytes, session.format) {
            Ok(intermediate) => intermediate
                .blocks
                .iter()
                .map(|b| b.text().to_string())
                .filter(|t| !t.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n"),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Extract tables from a document as structured data.")]
    async fn extract_tables(&self, params: Parameters<SessionIdParams>) -> String {
        let sessions = self.sessions.lock().unwrap();
        let session = match sessions.get(&params.0.session_id) {
            Some(s) => s,
            None => return format!("Error: session not found: {}", params.0.session_id),
        };

        match paperjam_convert::extract::extract(&session.bytes, session.format) {
            Ok(intermediate) => {
                let tables: Vec<serde_json::Value> = intermediate
                    .tables
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "rows": t.to_vec(),
                            "row_count": t.row_count(),
                            "col_count": t.col_count,
                        })
                    })
                    .collect();
                json_content(&tables)
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Extract document structure (headings, paragraphs, lists).")]
    async fn extract_structure(&self, params: Parameters<SessionIdParams>) -> String {
        let sessions = self.sessions.lock().unwrap();
        let session = match sessions.get(&params.0.session_id) {
            Some(s) => s,
            None => return format!("Error: session not found: {}", params.0.session_id),
        };

        match paperjam_convert::extract::extract(&session.bytes, session.format) {
            Ok(intermediate) => {
                let blocks: Vec<serde_json::Value> = intermediate
                    .blocks
                    .iter()
                    .map(|b| {
                        serde_json::json!({
                            "type": b.block_type(),
                            "text": b.text(),
                            "page": b.page(),
                        })
                    })
                    .collect();
                json_content(&blocks)
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Convert a document to Markdown.")]
    async fn to_markdown(&self, params: Parameters<SessionIdParams>) -> String {
        let sessions = self.sessions.lock().unwrap();
        let session = match sessions.get(&params.0.session_id) {
            Some(s) => s,
            None => return format!("Error: session not found: {}", params.0.session_id),
        };

        match paperjam_convert::extract::extract(&session.bytes, session.format) {
            Ok(intermediate) => {
                match paperjam_convert::generate::generate(
                    &intermediate,
                    paperjam_model::format::DocumentFormat::Markdown,
                ) {
                    Ok(bytes) => {
                        String::from_utf8(bytes).unwrap_or_else(|e| format!("Error: {}", e))
                    }
                    Err(e) => format!("Error: {}", e),
                }
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        description = "Convert a document to another format. Creates a new session with the converted document."
    )]
    async fn convert_document(&self, params: Parameters<ConvertDocumentParams>) -> String {
        let (bytes, format) = {
            let sessions = self.sessions.lock().unwrap();
            match sessions.get(&params.0.session_id) {
                Some(s) => (s.bytes.clone(), s.format),
                None => return format!("Error: session not found: {}", params.0.session_id),
            }
        };

        let target =
            paperjam_model::format::DocumentFormat::from_extension(&params.0.target_format);
        match paperjam_convert::convert_bytes(&bytes, format, target) {
            Ok(output) => {
                let new_id = self
                    .sessions
                    .lock()
                    .unwrap()
                    .open_from_bytes(output, target);
                json_content(&serde_json::json!({
                    "session_id": new_id,
                    "format": target.display_name(),
                }))
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Redact text matching a pattern in a PDF document.")]
    async fn redact_text(&self, params: Parameters<RedactTextParams>) -> String {
        let bytes = {
            let sessions = self.sessions.lock().unwrap();
            match sessions.get(&params.0.session_id) {
                Some(s) => s.bytes.clone(),
                None => return format!("Error: session not found: {}", params.0.session_id),
            }
        };

        let doc = match paperjam_core::document::Document::open_bytes(&bytes) {
            Ok(d) => d,
            Err(e) => return format!("Error: {}", e),
        };

        match paperjam_core::redact::redact_text(
            &doc,
            &params.0.pattern,
            params.0.case_sensitive,
            false,
            None,
        ) {
            Ok((new_doc, result)) => {
                let mut inner = new_doc.into_inner();
                let mut buf = Vec::new();
                if let Err(e) = inner.save_to(&mut buf) {
                    return format!("Error: {}", e);
                }

                let mut sessions = self.sessions.lock().unwrap();
                if let Some(session) = sessions.get_mut(&params.0.session_id) {
                    session.bytes = buf;
                }

                json_content(&serde_json::json!({
                    "pages_modified": result.pages_modified,
                    "items_redacted": result.items_redacted,
                }))
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Add a text watermark to a PDF document.")]
    async fn add_watermark(&self, params: Parameters<WatermarkParams>) -> String {
        let bytes = {
            let sessions = self.sessions.lock().unwrap();
            match sessions.get(&params.0.session_id) {
                Some(s) => s.bytes.clone(),
                None => return format!("Error: session not found: {}", params.0.session_id),
            }
        };

        let mut doc = match paperjam_core::document::Document::open_bytes(&bytes) {
            Ok(d) => d,
            Err(e) => return format!("Error: {}", e),
        };

        let options = paperjam_core::watermark::WatermarkOptions {
            text: params.0.text.clone(),
            font_size: params.0.font_size.unwrap_or(60.0),
            opacity: params.0.opacity.unwrap_or(0.3),
            ..paperjam_core::watermark::WatermarkOptions::default()
        };

        if let Err(e) = paperjam_core::watermark::add_watermark(&mut doc, &options) {
            return format!("Error: {}", e);
        }

        let mut inner = doc.into_inner();
        let mut buf = Vec::new();
        if let Err(e) = inner.save_to(&mut buf) {
            return format!("Error: {}", e);
        }

        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(&params.0.session_id) {
            session.bytes = buf;
        }

        text_content("Watermark added successfully")
    }

    #[tool(description = "Encrypt a PDF document with a password.")]
    async fn encrypt_document(&self, params: Parameters<EncryptParams>) -> String {
        let bytes = {
            let sessions = self.sessions.lock().unwrap();
            match sessions.get(&params.0.session_id) {
                Some(s) => s.bytes.clone(),
                None => return format!("Error: session not found: {}", params.0.session_id),
            }
        };

        let doc = match paperjam_core::document::Document::open_bytes(&bytes) {
            Ok(d) => d,
            Err(e) => return format!("Error: {}", e),
        };

        let algo = match params.0.algorithm.as_deref() {
            Some("aes256") => paperjam_core::encryption::EncryptionAlgorithm::Aes256,
            Some("rc4") => paperjam_core::encryption::EncryptionAlgorithm::Rc4,
            _ => paperjam_core::encryption::EncryptionAlgorithm::Aes128,
        };
        let options = paperjam_core::encryption::EncryptionOptions {
            user_password: params.0.password.clone(),
            owner_password: String::new(),
            permissions: paperjam_core::encryption::Permissions::default(),
            algorithm: algo,
        };

        match paperjam_core::encryption::encrypt(&doc, &options) {
            Ok(encrypted_bytes) => {
                let mut sessions = self.sessions.lock().unwrap();
                if let Some(session) = sessions.get_mut(&params.0.session_id) {
                    session.bytes = encrypted_bytes;
                }
                text_content("Document encrypted successfully")
            }
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Save an open document to a file.")]
    async fn save_document(&self, params: Parameters<SaveDocumentParams>) -> String {
        let bytes = {
            let sessions = self.sessions.lock().unwrap();
            match sessions.get(&params.0.session_id) {
                Some(s) => s.bytes.clone(),
                None => return format!("Error: session not found: {}", params.0.session_id),
            }
        };

        let path = match self.resolve_path(&params.0.output_path) {
            Ok(p) => p,
            Err(e) => return format!("Error: {}", e),
        };
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return format!("Error: {}", e);
            }
        }

        match std::fs::write(&path, &bytes) {
            Ok(()) => json_content(&serde_json::json!({
                "success": true,
                "path": path.display().to_string(),
            })),
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(description = "Run a document processing pipeline from a YAML definition.")]
    async fn run_pipeline(&self, params: Parameters<RunPipelineParams>) -> String {
        let definition = match paperjam_pipeline::PipelineDefinition::from_yaml(&params.0.yaml) {
            Ok(d) => d,
            Err(e) => return format!("Error: {}", e),
        };

        let engine = paperjam_pipeline::PipelineEngine::new(definition);
        match engine.run() {
            Ok(result) => {
                let result_json = serde_json::json!({
                    "total_files": result.total_files,
                    "succeeded": result.succeeded,
                    "failed": result.failed,
                    "skipped": result.skipped,
                });
                json_content(&result_json)
            }
            Err(e) => format!("Error: {}", e),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for PaperjamServer {}

#[cfg(test)]
mod tests {
    use super::*;

    fn server(tmp: &Path, allow_absolute: bool) -> PaperjamServer {
        PaperjamServer::with_config(ServerConfig {
            working_dir: tmp.to_path_buf(),
            allow_absolute_paths: allow_absolute,
        })
    }

    #[test]
    fn relative_path_inside_sandbox_is_allowed() {
        let tmp = std::env::temp_dir().canonicalize().unwrap();
        let s = server(&tmp, false);
        let resolved = s.resolve_path("some_file.pdf").unwrap();
        assert!(resolved.starts_with(&tmp));
    }

    #[test]
    fn absolute_path_outside_sandbox_is_rejected() {
        let tmp = std::env::temp_dir().canonicalize().unwrap();
        let s = server(&tmp, false);
        let err = s.resolve_path("/etc/passwd").unwrap_err();
        assert!(matches!(err, McpError::PathEscapesSandbox(_)));
    }

    #[test]
    fn parent_traversal_is_rejected() {
        let tmp = std::env::temp_dir().canonicalize().unwrap();
        let s = server(&tmp, false);
        let err = s.resolve_path("../../../etc/passwd").unwrap_err();
        assert!(matches!(err, McpError::PathEscapesSandbox(_)));
    }

    #[test]
    fn nonexistent_child_path_is_accepted() {
        let tmp = std::env::temp_dir().canonicalize().unwrap();
        let s = server(&tmp, false);
        let resolved = s.resolve_path("does_not_exist_yet.pdf").unwrap();
        assert!(resolved.starts_with(&tmp));
    }

    #[test]
    fn allow_absolute_flag_bypasses_containment_check() {
        let tmp = std::env::temp_dir().canonicalize().unwrap();
        let s = server(&tmp, true);
        let resolved = s.resolve_path("/etc/passwd").unwrap();
        assert_eq!(resolved, PathBuf::from("/etc/passwd"));
    }
}
