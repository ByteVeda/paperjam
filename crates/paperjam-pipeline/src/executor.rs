use paperjam_model::format::DocumentFormat;

use crate::context::PipelineContext;
use crate::error::{PipelineError, Result};
use crate::step::Step;

/// Execute a single step on a pipeline context.
pub fn execute_step(ctx: &mut PipelineContext, step: &Step) -> Result<()> {
    match step {
        Step::ExtractText { .. } => {
            let intermediate = paperjam_convert::extract::extract(&ctx.bytes, ctx.format)?;
            let text = intermediate
                .blocks
                .iter()
                .map(|b| b.text().to_string())
                .filter(|t| !t.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n");
            ctx.text = Some(text);
        }

        Step::ExtractTables { .. } => {
            let intermediate = paperjam_convert::extract::extract(&ctx.bytes, ctx.format)?;
            ctx.tables = Some(intermediate.tables);
        }

        Step::ExtractStructure => {
            let intermediate = paperjam_convert::extract::extract(&ctx.bytes, ctx.format)?;
            ctx.structure = Some(intermediate.blocks);
        }

        Step::Convert { format } => {
            let target = DocumentFormat::from_extension(format);
            if target == DocumentFormat::Unknown {
                return Err(PipelineError::Step {
                    step: "convert".to_string(),
                    message: format!("unknown target format: {}", format),
                });
            }
            let output = paperjam_convert::convert_bytes(&ctx.bytes, ctx.format, target)?;
            ctx.bytes = output;
            ctx.format = target;
            // Clear cached extractions since the document changed.
            ctx.text = None;
            ctx.tables = None;
            ctx.structure = None;
            ctx.markdown = None;
        }

        Step::ToMarkdown => {
            let intermediate = paperjam_convert::extract::extract(&ctx.bytes, ctx.format)?;
            let md_bytes =
                paperjam_convert::generate::generate(&intermediate, DocumentFormat::Markdown)
                    .map_err(|e| PipelineError::Generation(e.to_string()))?;
            let md = String::from_utf8(md_bytes)
                .map_err(|e| PipelineError::Generation(e.to_string()))?;
            ctx.markdown = Some(md);
        }

        Step::Redact {
            pattern,
            case_sensitive,
        } => {
            require_pdf(ctx, "redact")?;
            let doc = paperjam_core::document::Document::open_bytes(&ctx.bytes)?;
            let case_sensitive = case_sensitive.unwrap_or(false);
            let (new_doc, _result) =
                paperjam_core::redact::redact_text(&doc, pattern, case_sensitive, false, None)?;
            ctx.bytes = save_pdf_doc(new_doc)?;
        }

        Step::Watermark {
            text,
            font_size,
            opacity,
            rotation,
        } => {
            require_pdf(ctx, "watermark")?;
            let mut doc = paperjam_core::document::Document::open_bytes(&ctx.bytes)?;
            let options = paperjam_core::watermark::WatermarkOptions {
                text: text.clone(),
                font_size: font_size.unwrap_or(60.0),
                rotation: rotation.unwrap_or(45.0),
                opacity: opacity.unwrap_or(0.3),
                ..paperjam_core::watermark::WatermarkOptions::default()
            };
            paperjam_core::watermark::add_watermark(&mut doc, &options)?;
            ctx.bytes = save_pdf_doc(doc)?;
        }

        Step::Optimize { strip_metadata } => {
            require_pdf(ctx, "optimize")?;
            let doc = paperjam_core::document::Document::open_bytes(&ctx.bytes)?;
            let options = paperjam_core::optimization::OptimizeOptions {
                compress_streams: true,
                remove_unused_objects: true,
                remove_duplicates: true,
                strip_metadata: strip_metadata.unwrap_or(false),
            };
            let (new_doc, _result) = paperjam_core::optimization::optimize(&doc, &options)?;
            ctx.bytes = save_pdf_doc(new_doc)?;
        }

        Step::Sanitize {
            remove_javascript,
            remove_embedded_files,
        } => {
            require_pdf(ctx, "sanitize")?;
            let doc = paperjam_core::document::Document::open_bytes(&ctx.bytes)?;
            let options = paperjam_core::sanitize::SanitizeOptions {
                remove_javascript: remove_javascript.unwrap_or(true),
                remove_embedded_files: remove_embedded_files.unwrap_or(true),
                remove_actions: true,
                remove_links: false,
            };
            let (new_doc, _result) = paperjam_core::sanitize::sanitize(&doc, &options)?;
            ctx.bytes = save_pdf_doc(new_doc)?;
        }

        Step::Encrypt {
            user_password,
            owner_password,
            algorithm,
        } => {
            require_pdf(ctx, "encrypt")?;
            let doc = paperjam_core::document::Document::open_bytes(&ctx.bytes)?;
            let algo = match algorithm.as_deref() {
                Some("aes256") | Some("AES-256") => {
                    paperjam_core::encryption::EncryptionAlgorithm::Aes256
                }
                Some("rc4") | Some("RC4") => paperjam_core::encryption::EncryptionAlgorithm::Rc4,
                _ => paperjam_core::encryption::EncryptionAlgorithm::Aes128,
            };
            let options = paperjam_core::encryption::EncryptionOptions {
                user_password: user_password.clone(),
                owner_password: owner_password.clone().unwrap_or_default(),
                permissions: paperjam_core::encryption::Permissions::default(),
                algorithm: algo,
            };
            ctx.bytes = paperjam_core::encryption::encrypt(&doc, &options)?;
        }

        Step::Save { path } => {
            let output_path = ctx.resolve_save_path(path);
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Save the appropriate artifact.
            if let Some(ref text) = ctx.text {
                std::fs::write(&output_path, text)?;
            } else if let Some(ref md) = ctx.markdown {
                std::fs::write(&output_path, md)?;
            } else {
                std::fs::write(&output_path, &ctx.bytes)?;
            }
        }
    }

    Ok(())
}

/// Check that the current format is PDF, or return an error.
fn require_pdf(ctx: &PipelineContext, step_name: &str) -> Result<()> {
    if ctx.format != DocumentFormat::Pdf {
        return Err(PipelineError::Step {
            step: step_name.to_string(),
            message: format!(
                "{} requires PDF input, but document is {}",
                step_name,
                ctx.format.display_name()
            ),
        });
    }
    Ok(())
}

/// Serialize a PDF Document back to bytes.
fn save_pdf_doc(doc: paperjam_core::document::Document) -> Result<Vec<u8>> {
    let mut inner = doc.into_inner();
    let mut buf = Vec::new();
    inner
        .save_to(&mut buf)
        .map_err(|e| PipelineError::Generation(e.to_string()))?;
    Ok(buf)
}
