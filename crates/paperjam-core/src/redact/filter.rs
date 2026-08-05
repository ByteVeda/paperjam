use crate::text::font::FontInfo;
use crate::text::operators::{ContentOperator, TJElement, TextState};

/// A redacted text item: (decoded_text, bounding_box).
pub type RedactedItem = (String, [f64; 4]);

/// Filter content stream operators, removing text that overlaps any redaction region.
///
/// Returns (filtered_operators, redacted_items).
pub fn filter_ops(
    ops: &[ContentOperator],
    regions: &[[f64; 4]], // each rect is [x1, y1, x2, y2]
    fonts: &[FontInfo],
) -> (Vec<ContentOperator>, Vec<RedactedItem>) {
    let mut filtered = Vec::with_capacity(ops.len());
    let mut redacted = Vec::new();

    let mut text_state = TextState::default();
    let mut ctm = [1.0f64, 0.0, 0.0, 1.0, 0.0, 0.0];
    let mut ctm_stack: Vec<[f64; 6]> = Vec::new();

    for op in ops {
        match op {
            ContentOperator::SaveGraphicsState => {
                ctm_stack.push(ctm);
                filtered.push(op.clone());
            }
            ContentOperator::RestoreGraphicsState => {
                if let Some(prev) = ctm_stack.pop() {
                    ctm = prev;
                }
                filtered.push(op.clone());
            }
            ContentOperator::ConcatMatrix { matrix } => {
                ctm = multiply_matrices(matrix, &ctm);
                filtered.push(op.clone());
            }
            ContentOperator::BeginText => {
                text_state.text_matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
                text_state.text_line_matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
                filtered.push(op.clone());
            }
            ContentOperator::EndText => {
                filtered.push(op.clone());
            }
            ContentOperator::SetFont { name, size } => {
                text_state.font_name = name.clone();
                text_state.font_size = *size;
                filtered.push(op.clone());
            }
            ContentOperator::SetCharSpacing(v) => {
                text_state.char_spacing = *v;
                filtered.push(op.clone());
            }
            ContentOperator::SetWordSpacing(v) => {
                text_state.word_spacing = *v;
                filtered.push(op.clone());
            }
            ContentOperator::SetHorizontalScaling(v) => {
                text_state.horizontal_scaling = *v / 100.0;
                filtered.push(op.clone());
            }
            ContentOperator::SetTextLeading(v) => {
                text_state.leading = *v;
                filtered.push(op.clone());
            }
            ContentOperator::SetTextRise(v) => {
                text_state.rise = *v;
                filtered.push(op.clone());
            }
            ContentOperator::MoveTextPosition { tx, ty } => {
                let new_lm = translate_matrix(&text_state.text_line_matrix, *tx, *ty);
                text_state.text_line_matrix = new_lm;
                text_state.text_matrix = new_lm;
                filtered.push(op.clone());
            }
            ContentOperator::MoveTextPositionSetLeading { tx, ty } => {
                text_state.leading = -*ty;
                let new_lm = translate_matrix(&text_state.text_line_matrix, *tx, *ty);
                text_state.text_line_matrix = new_lm;
                text_state.text_matrix = new_lm;
                filtered.push(op.clone());
            }
            ContentOperator::SetTextMatrix { matrix } => {
                text_state.text_matrix = *matrix;
                text_state.text_line_matrix = *matrix;
                filtered.push(op.clone());
            }
            ContentOperator::NextLine => {
                let new_lm =
                    translate_matrix(&text_state.text_line_matrix, 0.0, -text_state.leading);
                text_state.text_line_matrix = new_lm;
                text_state.text_matrix = new_lm;
                filtered.push(op.clone());
            }
            ContentOperator::ShowText { bytes } => {
                let (text, bbox) = compute_text_info(bytes, &text_state, &ctm, fonts);
                if overlaps_any_region(&bbox, regions) {
                    redacted.push((text, bbox));
                } else {
                    filtered.push(op.clone());
                }
                // Always advance text state to maintain positions for subsequent operators
                advance_text_matrix(bytes, &mut text_state, fonts);
            }
            ContentOperator::ShowTextArray { elements } => {
                // Check if any text element in the array overlaps redaction regions
                let mut should_redact = false;
                let mut redacted_texts = Vec::new();

                for elem in elements {
                    match elem {
                        TJElement::Text(bytes) => {
                            let (text, bbox) = compute_text_info(bytes, &text_state, &ctm, fonts);
                            if overlaps_any_region(&bbox, regions) {
                                redacted_texts.push((text, bbox));
                                should_redact = true;
                            }
                            advance_text_matrix(bytes, &mut text_state, fonts);
                        }
                        TJElement::Offset(offset) => {
                            let displacement = -offset / 1000.0
                                * text_state.font_size
                                * text_state.horizontal_scaling;
                            text_state.text_matrix[4] += displacement * text_state.text_matrix[0];
                            text_state.text_matrix[5] += displacement * text_state.text_matrix[1];
                        }
                    }
                }

                if should_redact {
                    // Remove the entire TJ operator
                    redacted.extend(redacted_texts);
                } else {
                    filtered.push(op.clone());
                }
            }
            ContentOperator::NextLineShowText { bytes } => {
                // Equivalent to T* then Tj
                let new_lm =
                    translate_matrix(&text_state.text_line_matrix, 0.0, -text_state.leading);
                text_state.text_line_matrix = new_lm;
                text_state.text_matrix = new_lm;

                let (text, bbox) = compute_text_info(bytes, &text_state, &ctm, fonts);
                if overlaps_any_region(&bbox, regions) {
                    redacted.push((text, bbox));
                    // Emit just the T* (newline positioning) without the text
                    filtered.push(ContentOperator::NextLine);
                } else {
                    filtered.push(op.clone());
                }
                advance_text_matrix(bytes, &mut text_state, fonts);
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

                let (text, bbox) = compute_text_info(bytes, &text_state, &ctm, fonts);
                if overlaps_any_region(&bbox, regions) {
                    redacted.push((text, bbox));
                    // Emit state changes and line advance without the text
                    filtered.push(ContentOperator::SetWordSpacing(*word_spacing));
                    filtered.push(ContentOperator::SetCharSpacing(*char_spacing));
                    filtered.push(ContentOperator::NextLine);
                } else {
                    filtered.push(op.clone());
                }
                advance_text_matrix(bytes, &mut text_state, fonts);
            }
            _ => {
                // All other operators (path, raw, etc.) are kept as-is
                filtered.push(op.clone());
            }
        }
    }

    (filtered, redacted)
}

/// Compute the decoded text and approximate bounding box for a text-showing operation.
fn compute_text_info(
    bytes: &[u8],
    text_state: &TextState,
    ctm: &[f64; 6],
    fonts: &[FontInfo],
) -> (String, [f64; 4]) {
    let font = fonts.iter().find(|f| f.name == text_state.font_name);

    let text = match font {
        Some(f) => f.decode_bytes(bytes),
        None => String::from_utf8_lossy(bytes).to_string(),
    };

    // Text rendering matrix = text_matrix × CTM
    let trm = multiply_matrices(&text_state.text_matrix, ctm);
    let x = trm[4];
    let y = trm[5];
    let effective_font_size = text_state.font_size * (trm[0].powi(2) + trm[1].powi(2)).sqrt();

    let char_width = match font {
        Some(f) => f.estimate_string_width(bytes, text_state.font_size),
        None => text.len() as f64 * text_state.font_size * 0.5,
    };
    let width = char_width * text_state.horizontal_scaling;

    let fs = effective_font_size.abs();
    // Approximate bbox: descender at -0.3*fs, ascender at +0.8*fs from baseline
    let bbox = [x, y - fs * 0.3, x + width, y + fs * 0.8];

    (text, bbox)
}

/// Advance the text matrix after a ShowText operation (matching text/mod.rs behavior).
fn advance_text_matrix(bytes: &[u8], text_state: &mut TextState, fonts: &[FontInfo]) {
    let font = fonts.iter().find(|f| f.name == text_state.font_name);
    let char_width = match font {
        Some(f) => f.estimate_string_width(bytes, text_state.font_size),
        None => {
            let text = String::from_utf8_lossy(bytes);
            text.len() as f64 * text_state.font_size * 0.5
        }
    };
    let advance = char_width * text_state.horizontal_scaling;
    text_state.text_matrix[4] += advance * text_state.text_matrix[0];
    text_state.text_matrix[5] += advance * text_state.text_matrix[1];
}

/// Check if a bounding box overlaps any of the redaction regions.
fn overlaps_any_region(bbox: &[f64; 4], regions: &[[f64; 4]]) -> bool {
    regions.iter().any(|region| rects_overlap(bbox, region))
}

/// Two axis-aligned rectangles [x1, y1, x2, y2] overlap.
fn rects_overlap(a: &[f64; 4], b: &[f64; 4]) -> bool {
    a[0] < b[2] && a[2] > b[0] && a[1] < b[3] && a[3] > b[1]
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
