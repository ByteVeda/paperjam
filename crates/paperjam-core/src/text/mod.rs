pub mod cmap;
pub mod encoding;
pub mod font;
pub mod layout;
pub mod operators;

use crate::error::Result;
use crate::text::font::FontInfo;
use crate::text::layout::TextSpan;
use crate::text::operators::{parse_content_stream, ContentOperator, TJElement, TextState};

/// Extract positioned text spans from a content stream using the provided fonts.
pub fn extract_spans(content_bytes: &[u8], fonts: &[FontInfo]) -> Result<Vec<TextSpan>> {
    let ops = parse_content_stream(content_bytes)?;

    let mut spans = Vec::new();
    let mut text_state = TextState::default();
    let mut ctm = [1.0_f64, 0.0, 0.0, 1.0, 0.0, 0.0]; // Current transformation matrix
    let mut ctm_stack: Vec<[f64; 6]> = Vec::new();

    for op in &ops {
        match op {
            ContentOperator::SaveGraphicsState => {
                ctm_stack.push(ctm);
            }
            ContentOperator::RestoreGraphicsState => {
                if let Some(prev) = ctm_stack.pop() {
                    ctm = prev;
                }
            }
            ContentOperator::ConcatMatrix { matrix } => {
                ctm = multiply_matrices(matrix, &ctm);
            }
            ContentOperator::BeginText => {
                text_state.text_matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
                text_state.text_line_matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
            }
            ContentOperator::EndText => {}
            ContentOperator::SetFont { name, size } => {
                text_state.font_name = name.clone();
                text_state.font_size = *size;
            }
            ContentOperator::SetCharSpacing(v) => text_state.char_spacing = *v,
            ContentOperator::SetWordSpacing(v) => text_state.word_spacing = *v,
            ContentOperator::SetHorizontalScaling(v) => text_state.horizontal_scaling = *v / 100.0,
            ContentOperator::SetTextLeading(v) => text_state.leading = *v,
            ContentOperator::SetTextRise(v) => text_state.rise = *v,
            ContentOperator::MoveTextPosition { tx, ty } => {
                let new_lm = translate_matrix(&text_state.text_line_matrix, *tx, *ty);
                text_state.text_line_matrix = new_lm;
                text_state.text_matrix = new_lm;
            }
            ContentOperator::MoveTextPositionSetLeading { tx, ty } => {
                text_state.leading = -*ty;
                let new_lm = translate_matrix(&text_state.text_line_matrix, *tx, *ty);
                text_state.text_line_matrix = new_lm;
                text_state.text_matrix = new_lm;
            }
            ContentOperator::SetTextMatrix { matrix } => {
                text_state.text_matrix = *matrix;
                text_state.text_line_matrix = *matrix;
            }
            ContentOperator::NextLine => {
                let new_lm =
                    translate_matrix(&text_state.text_line_matrix, 0.0, -text_state.leading);
                text_state.text_line_matrix = new_lm;
                text_state.text_matrix = new_lm;
            }
            ContentOperator::ShowText { bytes } => {
                process_show_text(bytes, &mut text_state, &ctm, fonts, &mut spans);
            }
            ContentOperator::ShowTextArray { elements } => {
                for elem in elements {
                    match elem {
                        TJElement::Text(bytes) => {
                            process_show_text(bytes, &mut text_state, &ctm, fonts, &mut spans);
                        }
                        TJElement::Offset(offset) => {
                            // Offset is in thousandths of a unit of text space.
                            // Negative = move right, positive = move left.
                            let displacement = -offset / 1000.0
                                * text_state.font_size
                                * text_state.horizontal_scaling;
                            text_state.text_matrix[4] += displacement * text_state.text_matrix[0];
                            text_state.text_matrix[5] += displacement * text_state.text_matrix[1];
                        }
                    }
                }
            }
            ContentOperator::NextLineShowText { bytes } => {
                // Equivalent to T* followed by Tj
                let new_lm =
                    translate_matrix(&text_state.text_line_matrix, 0.0, -text_state.leading);
                text_state.text_line_matrix = new_lm;
                text_state.text_matrix = new_lm;
                process_show_text(bytes, &mut text_state, &ctm, fonts, &mut spans);
            }
            ContentOperator::SetSpacingNextLineShowText {
                word_spacing,
                char_spacing,
                bytes,
            } => {
                text_state.word_spacing = *word_spacing;
                text_state.char_spacing = *char_spacing;
                let new_lm =
                    translate_matrix(&text_state.text_line_matrix, 0.0, -text_state.leading);
                text_state.text_line_matrix = new_lm;
                text_state.text_matrix = new_lm;
                process_show_text(bytes, &mut text_state, &ctm, fonts, &mut spans);
            }
            _ => {} // Ignore non-text operators
        }
    }

    Ok(spans)
}

fn process_show_text(
    bytes: &[u8],
    text_state: &mut TextState,
    ctm: &[f64; 6],
    fonts: &[FontInfo],
    spans: &mut Vec<TextSpan>,
) {
    if bytes.is_empty() {
        return;
    }

    let font = fonts.iter().find(|f| f.name == text_state.font_name);

    let text = match font {
        Some(f) => f.decode_bytes(bytes),
        None => String::from_utf8_lossy(bytes).to_string(),
    };

    if text.is_empty() {
        return;
    }

    // Compute the position in user space by applying text matrix then CTM
    let trm = multiply_matrices(&text_state.text_matrix, ctm);
    let x = trm[4];
    let y = trm[5];
    let effective_font_size = text_state.font_size * (trm[0].powi(2) + trm[1].powi(2)).sqrt();

    // Estimate width based on character count (rough heuristic, improved with font widths later)
    let char_width = match font {
        Some(f) => f.estimate_string_width(bytes, text_state.font_size),
        None => text.len() as f64 * text_state.font_size * 0.5,
    };
    let width = char_width * text_state.horizontal_scaling;

    spans.push(TextSpan {
        text,
        x,
        y,
        width,
        font_size: effective_font_size.abs(),
        font_name: text_state.font_name.clone(),
    });

    // Advance text matrix
    let advance = width;
    text_state.text_matrix[4] += advance * text_state.text_matrix[0];
    text_state.text_matrix[5] += advance * text_state.text_matrix[1];
}

fn multiply_matrices(a: &[f64; 6], b: &[f64; 6]) -> [f64; 6] {
    [
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
        a[4] * b[0] + a[5] * b[2] + b[4],
        a[4] * b[1] + a[5] * b[3] + b[5],
    ]
}

fn translate_matrix(m: &[f64; 6], tx: f64, ty: f64) -> [f64; 6] {
    [
        m[0],
        m[1],
        m[2],
        m[3],
        tx * m[0] + ty * m[2] + m[4],
        tx * m[1] + ty * m[3] + m[5],
    ]
}
