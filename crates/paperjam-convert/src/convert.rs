use std::path::Path;

use paperjam_model::format::DocumentFormat;

use crate::detect::detect_format;
use crate::error::ConvertError;
use crate::extract;
use crate::generate;

/// Summary of a completed conversion.
pub struct ConvertReport {
    pub from_format: DocumentFormat,
    pub to_format: DocumentFormat,
    pub content_blocks: usize,
    pub tables: usize,
    pub images: usize,
}

/// Convert a file from one format to another, reading from `input_path` and
/// writing to `output_path`.  The source and target formats are inferred from
/// file extensions (with magic-byte fallback for the input).
pub fn convert(input_path: &Path, output_path: &Path) -> Result<ConvertReport, ConvertError> {
    let input_bytes = std::fs::read(input_path)?;
    let from_format = detect_format(input_path);
    let to_format = DocumentFormat::detect(output_path);

    if from_format == DocumentFormat::Unknown {
        return Err(ConvertError::unsupported(from_format));
    }
    if to_format == DocumentFormat::Unknown {
        return Err(ConvertError::unsupported(to_format));
    }

    let intermediate = extract::extract(&input_bytes, from_format)?;
    let output_bytes = generate::generate(&intermediate, to_format)?;

    std::fs::write(output_path, &output_bytes)?;

    Ok(ConvertReport {
        from_format,
        to_format,
        content_blocks: intermediate.blocks.len(),
        tables: intermediate.tables.len(),
        images: intermediate.images.len(),
    })
}

/// Convert in-memory bytes from one format to another.
pub fn convert_bytes(
    input: &[u8],
    from_format: DocumentFormat,
    to_format: DocumentFormat,
) -> Result<Vec<u8>, ConvertError> {
    let intermediate = extract::extract(input, from_format)?;
    generate::generate(&intermediate, to_format)
}
