use crate::document::SlideData;
use paperjam_model::text::{TextLine, TextSpan};

/// Concatenate all slide text into a single string, with slide separators.
pub fn slides_to_text(slides: &[SlideData]) -> String {
    let mut out = String::new();

    for slide in slides {
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(&format!("--- Slide {} ---\n", slide.index));

        // Title first
        if let Some(ref title) = slide.title {
            out.push_str(title);
            out.push('\n');
        }

        // Body text
        for block in &slide.text_blocks {
            if block.is_title {
                continue; // already emitted
            }
            out.push_str(&block.text);
            out.push('\n');
        }

        // Notes
        if let Some(ref notes) = slide.notes {
            if !notes.is_empty() {
                out.push_str("\nNotes:\n");
                out.push_str(notes);
                out.push('\n');
            }
        }
    }

    out
}

/// Convert slides into positioned `TextLine` values.
///
/// Since PPTX doesn't give us precise x/y coordinates the way PDFs do, we
/// synthesize approximate positions: x = 0, y decreases by font_size for
/// each line, resetting per slide.
pub fn slides_to_text_lines(slides: &[SlideData]) -> Vec<TextLine> {
    let mut lines = Vec::new();
    let font_size = 12.0;

    for slide in slides {
        let mut y = 800.0; // start near top of a virtual page

        for block in &slide.text_blocks {
            if block.text.is_empty() {
                continue;
            }
            let span = TextSpan {
                text: block.text.clone(),
                x: 0.0,
                y,
                width: block.text.len() as f64 * 6.0, // rough estimate
                font_size,
                font_name: String::new(),
            };
            let width = span.width;
            lines.push(TextLine {
                spans: vec![span],
                bbox: (0.0, y, width, y + font_size),
            });
            y -= font_size * 1.5;
        }
    }

    lines
}
