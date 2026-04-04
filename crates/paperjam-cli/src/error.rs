use std::fmt;
use std::path::PathBuf;

use paperjam_convert::ConvertError;
use paperjam_core::error::PdfError;
use paperjam_docx::DocxError;
use paperjam_epub::EpubError;
use paperjam_html::HtmlError;
use paperjam_pptx::PptxError;
use paperjam_xlsx::XlsxError;

#[derive(Debug)]
pub enum CliError {
    Pdf(PdfError),
    Docx(DocxError),
    Xlsx(XlsxError),
    Pptx(PptxError),
    Html(HtmlError),
    Epub(EpubError),
    Convert(ConvertError),
    FileNotFound(PathBuf),
    InvalidArgument(String),
    Io(std::io::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Pdf(e) => match e {
                PdfError::PasswordRequired => {
                    write!(f, "Error: Password required to open this document\n  Hint: Use --password <PASSWORD> to provide the document password.")
                }
                PdfError::InvalidPassword => {
                    write!(
                        f,
                        "Error: Invalid password\n  Hint: Check the password and try again."
                    )
                }
                PdfError::PageOutOfRange { page, total } => {
                    write!(
                        f,
                        "Error: Page {} out of range (document has {} pages)",
                        page, total
                    )
                }
                other => write!(f, "Error: {}", other),
            },
            CliError::Docx(e) => write!(f, "Error: {}", e),
            CliError::Xlsx(e) => write!(f, "Error: {}", e),
            CliError::Pptx(e) => write!(f, "Error: {}", e),
            CliError::Html(e) => write!(f, "Error: {}", e),
            CliError::Epub(e) => write!(f, "Error: {}", e),
            CliError::Convert(e) => write!(f, "Error: {}", e),
            CliError::FileNotFound(path) => {
                write!(
                    f,
                    "Error: File not found: {}\n  Hint: Check the file path and ensure it exists.",
                    path.display()
                )
            }
            CliError::InvalidArgument(msg) => {
                write!(f, "Error: {}", msg)
            }
            CliError::Io(e) => write!(f, "Error: I/O error: {}", e),
        }
    }
}

impl From<PdfError> for CliError {
    fn from(e: PdfError) -> Self {
        CliError::Pdf(e)
    }
}

impl From<DocxError> for CliError {
    fn from(e: DocxError) -> Self {
        CliError::Docx(e)
    }
}

impl From<XlsxError> for CliError {
    fn from(e: XlsxError) -> Self {
        CliError::Xlsx(e)
    }
}

impl From<PptxError> for CliError {
    fn from(e: PptxError) -> Self {
        CliError::Pptx(e)
    }
}

impl From<HtmlError> for CliError {
    fn from(e: HtmlError) -> Self {
        CliError::Html(e)
    }
}

impl From<EpubError> for CliError {
    fn from(e: EpubError) -> Self {
        CliError::Epub(e)
    }
}

impl From<ConvertError> for CliError {
    fn from(e: ConvertError) -> Self {
        CliError::Convert(e)
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::Io(e)
    }
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::FileNotFound(_) | CliError::InvalidArgument(_) => 1,
            CliError::Pdf(_)
            | CliError::Docx(_)
            | CliError::Xlsx(_)
            | CliError::Pptx(_)
            | CliError::Html(_)
            | CliError::Epub(_)
            | CliError::Convert(_)
            | CliError::Io(_) => 2,
        }
    }
}
