use paperjam_model::format::DocumentFormat;

/// Async version of `paperjam_convert::convert_bytes`.
pub async fn convert_bytes(
    input: Vec<u8>,
    from_format: DocumentFormat,
    to_format: DocumentFormat,
) -> Result<Vec<u8>, paperjam_convert::ConvertError> {
    tokio::task::spawn_blocking(move || {
        paperjam_convert::convert_bytes(&input, from_format, to_format)
    })
    .await
    .unwrap_or_else(|e| Err(paperjam_convert::ConvertError::Extraction(e.to_string())))
}

/// Async version of `paperjam_convert::convert` (file-to-file).
pub async fn convert(
    input_path: String,
    output_path: String,
) -> Result<paperjam_convert::ConvertReport, paperjam_convert::ConvertError> {
    tokio::task::spawn_blocking(move || {
        paperjam_convert::convert(
            std::path::Path::new(&input_path),
            std::path::Path::new(&output_path),
        )
    })
    .await
    .unwrap_or_else(|e| Err(paperjam_convert::ConvertError::Extraction(e.to_string())))
}

/// Async extraction to `IntermediateDoc`.
pub async fn extract(
    bytes: Vec<u8>,
    format: DocumentFormat,
) -> Result<paperjam_convert::IntermediateDoc, paperjam_convert::ConvertError> {
    tokio::task::spawn_blocking(move || paperjam_convert::extract::extract(&bytes, format))
        .await
        .unwrap_or_else(|e| Err(paperjam_convert::ConvertError::Extraction(e.to_string())))
}
