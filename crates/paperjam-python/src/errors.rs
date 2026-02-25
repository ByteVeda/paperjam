use paperjam_core::error::PdfError;
use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

create_exception!(_paperjam, PaperJamError, PyException, "Base paperjam error.");
create_exception!(_paperjam, ParseError, PaperJamError, "PDF parsing error.");
create_exception!(
    _paperjam,
    PasswordRequired,
    PaperJamError,
    "Password required."
);
create_exception!(
    _paperjam,
    InvalidPassword,
    PaperJamError,
    "Invalid password."
);
create_exception!(
    _paperjam,
    PageOutOfRange,
    PaperJamError,
    "Page out of range."
);
create_exception!(
    _paperjam,
    UnsupportedFeature,
    PaperJamError,
    "Unsupported feature."
);
create_exception!(
    _paperjam,
    TableExtractionError,
    PaperJamError,
    "Table extraction error."
);
create_exception!(
    _paperjam,
    OptimizationError,
    PaperJamError,
    "Optimization error."
);
create_exception!(
    _paperjam,
    AnnotationError,
    PaperJamError,
    "Annotation error."
);
create_exception!(
    _paperjam,
    WatermarkError,
    PaperJamError,
    "Watermark error."
);

/// Convert a Rust PdfError into the appropriate Python exception.
pub fn to_py_err(err: PdfError) -> PyErr {
    match err {
        PdfError::Io(e) => pyo3::exceptions::PyIOError::new_err(e.to_string()),
        PdfError::Parse { message, .. } => ParseError::new_err(message),
        PdfError::Structure(msg) => ParseError::new_err(msg),
        PdfError::PasswordRequired => PasswordRequired::new_err("Password required"),
        PdfError::InvalidPassword => InvalidPassword::new_err("Invalid password"),
        PdfError::PageOutOfRange { page, total } => {
            PageOutOfRange::new_err(format!(
                "Page {} out of range (document has {} pages)",
                page, total
            ))
        }
        PdfError::Unsupported(msg) => UnsupportedFeature::new_err(msg),
        PdfError::TableExtraction(msg) => TableExtractionError::new_err(msg),
        PdfError::Optimization(msg) => OptimizationError::new_err(msg),
        PdfError::Annotation(msg) => AnnotationError::new_err(msg),
        PdfError::Watermark(msg) => WatermarkError::new_err(msg),
        PdfError::FontDecode { font_name, message } => {
            ParseError::new_err(format!("Font '{}': {}", font_name, message))
        }
        PdfError::ObjectNotFound(num, gen) => {
            ParseError::new_err(format!("Object ({}, {}) not found", num, gen))
        }
        PdfError::Lopdf(e) => ParseError::new_err(e.to_string()),
        PdfError::Encryption(msg) => ParseError::new_err(msg),
    }
}

/// Register exception types on the module.
pub fn register_exceptions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("PaperJamError", m.py().get_type::<PaperJamError>())?;
    m.add("ParseError", m.py().get_type::<ParseError>())?;
    m.add("PasswordRequired", m.py().get_type::<PasswordRequired>())?;
    m.add("InvalidPassword", m.py().get_type::<InvalidPassword>())?;
    m.add("PageOutOfRange", m.py().get_type::<PageOutOfRange>())?;
    m.add("UnsupportedFeature", m.py().get_type::<UnsupportedFeature>())?;
    m.add(
        "TableExtractionError",
        m.py().get_type::<TableExtractionError>(),
    )?;
    m.add(
        "OptimizationError",
        m.py().get_type::<OptimizationError>(),
    )?;
    m.add("AnnotationError", m.py().get_type::<AnnotationError>())?;
    m.add("WatermarkError", m.py().get_type::<WatermarkError>())?;
    Ok(())
}
