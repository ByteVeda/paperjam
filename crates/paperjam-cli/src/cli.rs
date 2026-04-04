use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "pj",
    version,
    about = "paperjam — fast document processing from the command line",
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Output format
    #[arg(long, global = true, default_value = "text")]
    pub format: OutputFormat,

    /// Password for encrypted PDFs
    #[arg(long, global = true, env = "PAPERJAM_PASSWORD")]
    pub password: Option<String>,

    /// Suppress non-essential output
    #[arg(long, short, global = true)]
    pub quiet: bool,
}

#[derive(ValueEnum, Clone, Copy, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}

#[derive(Subcommand)]
pub enum Command {
    /// Show document info (metadata, pages, encryption status)
    Info(InfoArgs),
    /// Extract content from a document
    Extract(ExtractArgs),
    /// Convert a document to another format
    Convert(ConvertArgs),
    /// Merge multiple PDFs into one
    Merge(MergeArgs),
    /// Split a PDF into multiple files
    Split(SplitArgs),
    /// Rotate pages
    Rotate(RotateArgs),
    /// Reorder pages
    Reorder(ReorderArgs),
    /// Delete pages
    Delete(DeleteArgs),
    /// Add a text watermark
    Watermark(WatermarkArgs),
    /// Stamp/overlay one PDF onto another
    Stamp(StampArgs),
    /// Redact text by pattern
    Redact(RedactArgs),
    /// Remove potentially dangerous content
    Sanitize(SanitizeArgs),
    /// Optimize file size
    Optimize(OptimizeArgs),
    /// Encrypt with password
    Encrypt(EncryptArgs),
    /// Digitally sign a PDF
    Sign(SignArgs),
    /// Verify digital signatures
    Verify(VerifyArgs),
    /// Validate PDF/A or PDF/UA conformance
    Validate(ValidateArgs),
    /// Compare two PDFs
    Diff(DiffArgs),
    /// Inspect or fill form fields
    Form(FormArgs),
    /// Generate table of contents from headings
    Toc(TocArgs),
    /// Render pages to images
    #[cfg(feature = "render")]
    Render(RenderArgs),
    /// Run a document processing pipeline
    Pipeline(PipelineArgs),
}

// --- Info ---

#[derive(Args)]
pub struct InfoArgs {
    /// Input file (PDF, DOCX, XLSX, PPTX)
    pub file: PathBuf,
}

// --- Extract ---

#[derive(Args)]
pub struct ExtractArgs {
    #[command(subcommand)]
    pub what: ExtractWhat,
}

#[derive(Subcommand)]
pub enum ExtractWhat {
    /// Extract plain text
    Text(ExtractTextArgs),
    /// Extract tables
    Tables(ExtractTablesArgs),
    /// Extract images
    Images(ExtractImagesArgs),
    /// Extract document structure (headings, paragraphs, lists)
    Structure(ExtractStructureArgs),
    /// Extract bookmarks/outlines
    Bookmarks(ExtractBookmarksArgs),
    /// Extract annotations
    Annotations(ExtractAnnotationsArgs),
    /// Extract metadata
    Metadata(ExtractMetadataArgs),
}

#[derive(Args)]
pub struct ExtractTextArgs {
    /// Input file (PDF, DOCX, XLSX, PPTX)
    pub file: PathBuf,
    /// Page numbers or ranges (e.g., "1-5,8,10-12") — PDF only
    #[arg(long)]
    pub pages: Option<String>,
}

#[derive(Args)]
pub struct ExtractTablesArgs {
    /// Input file (PDF, DOCX, XLSX, PPTX)
    pub file: PathBuf,
    /// Page numbers or ranges
    #[arg(long)]
    pub pages: Option<String>,
    /// Table detection strategy
    #[arg(long, default_value = "auto")]
    pub strategy: String,
}

#[derive(Args)]
pub struct ExtractImagesArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// Output directory for extracted images
    #[arg(long, default_value = ".")]
    pub output_dir: PathBuf,
    /// Page numbers or ranges
    #[arg(long)]
    pub pages: Option<String>,
}

#[derive(Args)]
pub struct ExtractStructureArgs {
    /// Input file (PDF, DOCX, XLSX, PPTX)
    pub file: PathBuf,
}

#[derive(Args)]
pub struct ExtractBookmarksArgs {
    /// Input PDF file
    pub file: PathBuf,
}

#[derive(Args)]
pub struct ExtractAnnotationsArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// Page numbers or ranges
    #[arg(long)]
    pub pages: Option<String>,
}

#[derive(Args)]
pub struct ExtractMetadataArgs {
    /// Input file (PDF, DOCX, XLSX, PPTX)
    pub file: PathBuf,
}

// --- Convert ---

#[derive(Args)]
pub struct ConvertArgs {
    #[command(subcommand)]
    pub target: ConvertTarget,
}

#[derive(Subcommand)]
pub enum ConvertTarget {
    /// Convert to Markdown
    Markdown(ConvertMarkdownArgs),
    /// Convert to PDF/A
    PdfA(ConvertPdfAArgs),
    /// Convert to PDF
    ToPdf(ConvertToPdfArgs),
    /// Convert to DOCX
    ToDocx(ConvertToDocxArgs),
    /// Convert to XLSX
    ToXlsx(ConvertToXlsxArgs),
    /// Convert to HTML
    ToHtml(ConvertToHtmlArgs),
    /// Convert to EPUB
    ToEpub(ConvertToEpubArgs),
    /// Auto-detect target format from output extension
    Auto(ConvertAutoArgs),
}

#[derive(Args)]
pub struct ConvertMarkdownArgs {
    /// Input file (PDF, DOCX, XLSX, PPTX)
    pub file: PathBuf,
    /// Use layout-aware extraction (PDF only)
    #[arg(long)]
    pub layout_aware: bool,
    /// Include page number annotations (PDF only)
    #[arg(long)]
    pub page_numbers: bool,
    /// Use HTML tables instead of pipe tables (PDF only)
    #[arg(long)]
    pub html_tables: bool,
}

#[derive(Args)]
pub struct ConvertPdfAArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// PDF/A level (1b, 1a, 2b)
    #[arg(long, default_value = "1b")]
    pub level: String,
    /// Output file
    #[arg(short, long)]
    pub output: PathBuf,
}

#[derive(Args)]
pub struct ConvertToPdfArgs {
    /// Input file (DOCX, XLSX, PPTX)
    pub file: PathBuf,
    /// Output PDF file
    #[arg(short, long)]
    pub output: PathBuf,
}

#[derive(Args)]
pub struct ConvertToDocxArgs {
    /// Input file (PDF, XLSX, PPTX)
    pub file: PathBuf,
    /// Output DOCX file
    #[arg(short, long)]
    pub output: PathBuf,
}

#[derive(Args)]
pub struct ConvertToXlsxArgs {
    /// Input file (PDF, DOCX, PPTX)
    pub file: PathBuf,
    /// Output XLSX file
    #[arg(short, long)]
    pub output: PathBuf,
}

#[derive(Args)]
pub struct ConvertToHtmlArgs {
    /// Input file (PDF, DOCX, XLSX, PPTX, EPUB)
    pub file: PathBuf,
    /// Output HTML file
    #[arg(short, long)]
    pub output: PathBuf,
}

#[derive(Args)]
pub struct ConvertToEpubArgs {
    /// Input file (PDF, DOCX, XLSX, PPTX, HTML)
    pub file: PathBuf,
    /// Output EPUB file
    #[arg(short, long)]
    pub output: PathBuf,
}

#[derive(Args)]
pub struct ConvertAutoArgs {
    /// Input file
    pub file: PathBuf,
    /// Output file (format detected from extension)
    #[arg(short, long)]
    pub output: PathBuf,
}

// --- Merge ---

#[derive(Args)]
pub struct MergeArgs {
    /// Input PDF files (2 or more)
    pub files: Vec<PathBuf>,
    /// Output file
    #[arg(short, long)]
    pub output: PathBuf,
}

// --- Split ---

#[derive(Args)]
pub struct SplitArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// Page ranges (e.g., "1-5,6-10")
    #[arg(long)]
    pub ranges: String,
    /// Output directory
    #[arg(short, long, default_value = ".")]
    pub output_dir: PathBuf,
}

// --- Rotate ---

#[derive(Args)]
pub struct RotateArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// Rotation angle (90, 180, 270)
    #[arg(long)]
    pub angle: i32,
    /// Page numbers or ranges (default: all)
    #[arg(long)]
    pub pages: Option<String>,
    /// Output file
    #[arg(short, long)]
    pub output: PathBuf,
}

// --- Reorder ---

#[derive(Args)]
pub struct ReorderArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// New page order (comma-separated, e.g., "3,1,2,4")
    #[arg(long)]
    pub order: String,
    /// Output file
    #[arg(short, long)]
    pub output: PathBuf,
}

// --- Delete ---

#[derive(Args)]
pub struct DeleteArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// Page numbers or ranges to delete
    #[arg(long)]
    pub pages: String,
    /// Output file
    #[arg(short, long)]
    pub output: PathBuf,
}

// --- Watermark ---

#[derive(Args)]
pub struct WatermarkArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// Watermark text
    #[arg(long)]
    pub text: String,
    /// Font size
    #[arg(long, default_value = "60")]
    pub font_size: f64,
    /// Rotation in degrees
    #[arg(long, default_value = "45")]
    pub rotation: f64,
    /// Opacity (0.0-1.0)
    #[arg(long, default_value = "0.3")]
    pub opacity: f64,
    /// Position (center, top_left, top_right, bottom_left, bottom_right)
    #[arg(long, default_value = "center")]
    pub position: String,
    /// Layer (over, under)
    #[arg(long, default_value = "over")]
    pub layer: String,
    /// Page numbers or ranges (default: all)
    #[arg(long)]
    pub pages: Option<String>,
    /// Output file
    #[arg(short, long)]
    pub output: PathBuf,
}

// --- Stamp ---

#[derive(Args)]
pub struct StampArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// Stamp PDF file
    #[arg(long)]
    pub stamp: PathBuf,
    /// Page in stamp PDF to use (default: 1)
    #[arg(long, default_value = "1")]
    pub source_page: u32,
    /// Layer (over, under)
    #[arg(long, default_value = "over")]
    pub layer: String,
    /// Scale factor
    #[arg(long, default_value = "1.0")]
    pub scale: f64,
    /// Opacity (0.0-1.0)
    #[arg(long, default_value = "1.0")]
    pub opacity: f64,
    /// Output file
    #[arg(short, long)]
    pub output: PathBuf,
}

// --- Redact ---

#[derive(Args)]
pub struct RedactArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// Text or regex pattern to redact
    #[arg(long)]
    pub pattern: String,
    /// Treat pattern as regex
    #[arg(long)]
    pub regex: bool,
    /// Case-sensitive matching
    #[arg(long)]
    pub case_sensitive: bool,
    /// Fill color (r,g,b in 0-255, e.g. "0,0,0")
    #[arg(long)]
    pub fill_color: Option<String>,
    /// Output file
    #[arg(short, long)]
    pub output: PathBuf,
}

// --- Sanitize ---

#[derive(Args)]
pub struct SanitizeArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// Keep JavaScript
    #[arg(long)]
    pub keep_javascript: bool,
    /// Keep embedded files
    #[arg(long)]
    pub keep_embedded_files: bool,
    /// Keep actions
    #[arg(long)]
    pub keep_actions: bool,
    /// Keep links
    #[arg(long)]
    pub keep_links: bool,
    /// Output file
    #[arg(short, long)]
    pub output: PathBuf,
}

// --- Optimize ---

#[derive(Args)]
pub struct OptimizeArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// Skip stream compression
    #[arg(long)]
    pub no_compress: bool,
    /// Strip metadata
    #[arg(long)]
    pub strip_metadata: bool,
    /// Output file
    #[arg(short, long)]
    pub output: PathBuf,
}

// --- Encrypt ---

#[derive(Args)]
pub struct EncryptArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// User password
    #[arg(long)]
    pub user_password: String,
    /// Owner password (defaults to user password)
    #[arg(long)]
    pub owner_password: Option<String>,
    /// Algorithm (rc4, aes128, aes256)
    #[arg(long, default_value = "aes128")]
    pub algorithm: String,
    /// Disable printing
    #[arg(long)]
    pub no_print: bool,
    /// Disable copying
    #[arg(long)]
    pub no_copy: bool,
    /// Disable modification
    #[arg(long)]
    pub no_modify: bool,
    /// Output file
    #[arg(short, long)]
    pub output: PathBuf,
}

// --- Sign ---

#[derive(Args)]
pub struct SignArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// Certificate file (PEM or DER)
    #[arg(long)]
    pub cert: PathBuf,
    /// Private key file (PEM or DER)
    #[arg(long)]
    pub key: PathBuf,
    /// Reason for signing
    #[arg(long)]
    pub reason: Option<String>,
    /// Location
    #[arg(long)]
    pub location: Option<String>,
    /// Output file
    #[arg(short, long)]
    pub output: PathBuf,
}

// --- Verify ---

#[derive(Args)]
pub struct VerifyArgs {
    /// Input PDF file
    pub file: PathBuf,
}

// --- Validate ---

#[derive(Args)]
pub struct ValidateArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// Validation type (pdf-a, pdf-ua)
    #[arg(long, default_value = "pdf-a")]
    pub standard: String,
    /// PDF/A level (1b, 1a, 2b) — only for pdf-a
    #[arg(long, default_value = "1b")]
    pub level: String,
}

// --- Diff ---

#[derive(Args)]
pub struct DiffArgs {
    /// First PDF file
    pub file_a: PathBuf,
    /// Second PDF file
    pub file_b: PathBuf,
}

// --- Form ---

#[derive(Args)]
pub struct FormArgs {
    #[command(subcommand)]
    pub action: FormAction,
}

#[derive(Subcommand)]
pub enum FormAction {
    /// List form fields
    Inspect(FormInspectArgs),
    /// Fill form fields
    Fill(FormFillArgs),
}

#[derive(Args)]
pub struct FormInspectArgs {
    /// Input PDF file
    pub file: PathBuf,
}

#[derive(Args)]
pub struct FormFillArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// Field values as name=value pairs
    #[arg(long = "set", value_name = "NAME=VALUE")]
    pub fields: Vec<String>,
    /// Output file
    #[arg(short, long)]
    pub output: PathBuf,
}

// --- Toc ---

#[derive(Args)]
pub struct TocArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// Maximum heading depth (1-6)
    #[arg(long, default_value = "6")]
    pub max_depth: u8,
    /// Output file
    #[arg(short, long)]
    pub output: PathBuf,
}

// --- Render ---

#[cfg(feature = "render")]
#[derive(Args)]
pub struct RenderArgs {
    /// Input PDF file
    pub file: PathBuf,
    /// DPI for rendering
    #[arg(long, default_value = "150")]
    pub dpi: f32,
    /// Image format (png, jpeg, bmp)
    #[arg(long, default_value = "png")]
    pub image_format: String,
    /// Page numbers or ranges (default: all)
    #[arg(long)]
    pub pages: Option<String>,
    /// Output directory
    #[arg(short, long, default_value = ".")]
    pub output_dir: PathBuf,
}

// --- Pipeline ---

#[derive(Args)]
pub struct PipelineArgs {
    #[command(subcommand)]
    pub action: PipelineAction,
}

#[derive(Subcommand)]
pub enum PipelineAction {
    /// Run a pipeline from a YAML or JSON definition file
    Run(PipelineRunArgs),
    /// Validate a pipeline definition file
    Validate(PipelineValidateArgs),
}

#[derive(Args)]
pub struct PipelineRunArgs {
    /// Pipeline definition file (YAML or JSON)
    pub file: PathBuf,
    /// Process files in parallel
    #[arg(long)]
    pub parallel: bool,
    /// Error handling strategy (fail-fast, skip, collect-errors)
    #[arg(long)]
    pub on_error: Option<String>,
}

#[derive(Args)]
pub struct PipelineValidateArgs {
    /// Pipeline definition file (YAML or JSON)
    pub file: PathBuf,
}
