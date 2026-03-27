/// A positioned piece of text on a page.
#[derive(Debug, Clone)]
pub struct TextSpan {
    /// The text content (Unicode).
    pub text: String,
    /// X coordinate of the span origin (in points from page left).
    pub x: f64,
    /// Y coordinate of the span origin (in points from page bottom).
    pub y: f64,
    /// Width of the rendered text in points.
    pub width: f64,
    /// Font size in points.
    pub font_size: f64,
    /// Font name as declared in the PDF.
    pub font_name: String,
}

/// A line of text (multiple spans grouped by vertical proximity).
#[derive(Debug, Clone)]
pub struct TextLine {
    /// Spans composing this line, sorted left-to-right.
    pub spans: Vec<TextSpan>,
    /// Bounding box: (x_min, y_min, x_max, y_max) in points.
    pub bbox: (f64, f64, f64, f64),
}

impl TextLine {
    /// Concatenate span texts into a single line string.
    pub fn text(&self) -> String {
        let mut result = String::new();
        for (i, span) in self.spans.iter().enumerate() {
            if i > 0 {
                let prev = &self.spans[i - 1];
                let gap = span.x - (prev.x + prev.width);
                // Insert space if gap exceeds threshold relative to font size
                let threshold = if prev.font_size > 0.0 {
                    prev.font_size * 0.25
                } else {
                    2.0
                };
                if gap > threshold {
                    result.push(' ');
                }
            }
            result.push_str(&span.text);
        }
        result
    }

    /// Group a flat list of spans into lines based on Y-coordinate proximity.
    pub fn group_from_spans(spans: &[TextSpan]) -> Vec<TextLine> {
        if spans.is_empty() {
            return Vec::new();
        }

        // Sort spans by Y descending (top of page first), then X ascending.
        let mut sorted: Vec<&TextSpan> = spans.iter().collect();
        sorted.sort_by(|a, b| {
            b.y.partial_cmp(&a.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
        });

        let mut lines: Vec<TextLine> = Vec::new();

        for span in sorted {
            // Find an existing line with similar Y
            let avg_font_size = if span.font_size > 0.0 {
                span.font_size
            } else {
                12.0
            };
            let y_threshold = avg_font_size * 0.5;

            let matching_line = lines.iter_mut().find(|line| {
                // Check if this span's Y is close to the line's Y range
                let line_y = line.spans.first().map(|s| s.y).unwrap_or(0.0);
                (span.y - line_y).abs() < y_threshold
            });

            match matching_line {
                Some(line) => {
                    // Insert in X-sorted order
                    let insert_pos = line
                        .spans
                        .iter()
                        .position(|s| s.x > span.x)
                        .unwrap_or(line.spans.len());
                    line.spans.insert(insert_pos, span.clone());
                }
                None => {
                    lines.push(TextLine {
                        spans: vec![span.clone()],
                        bbox: (0.0, 0.0, 0.0, 0.0), // Computed later
                    });
                }
            }
        }

        // Compute bounding boxes and sort lines top-to-bottom
        for line in &mut lines {
            if line.spans.is_empty() {
                continue;
            }
            let x_min = line.spans.iter().map(|s| s.x).fold(f64::INFINITY, f64::min);
            let y_min = line.spans.iter().map(|s| s.y).fold(f64::INFINITY, f64::min);
            let x_max = line
                .spans
                .iter()
                .map(|s| s.x + s.width)
                .fold(f64::NEG_INFINITY, f64::max);
            let y_max = line
                .spans
                .iter()
                .map(|s| s.y + s.font_size)
                .fold(f64::NEG_INFINITY, f64::max);
            line.bbox = (x_min, y_min, x_max, y_max);
        }

        // Sort lines by Y descending (top of page first in PDF coordinates)
        lines.sort_by(|a, b| {
            b.bbox
                .1
                .partial_cmp(&a.bbox.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        lines
    }
}
