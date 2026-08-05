use std::io::Write as IoWrite;

use crate::text::operators::{ContentOperator, TJElement};

/// Encode a sequence of content stream operators back to PDF content stream bytes.
pub fn encode_content_stream(ops: &[ContentOperator]) -> Vec<u8> {
    let mut out = Vec::new();
    for op in ops {
        encode_operator(&mut out, op);
    }
    out
}

fn encode_operator(out: &mut Vec<u8>, op: &ContentOperator) {
    match op {
        ContentOperator::SaveGraphicsState => out.extend_from_slice(b"q\n"),
        ContentOperator::RestoreGraphicsState => out.extend_from_slice(b"Q\n"),
        ContentOperator::ConcatMatrix { matrix } => {
            write_number(out, matrix[0]);
            out.push(b' ');
            write_number(out, matrix[1]);
            out.push(b' ');
            write_number(out, matrix[2]);
            out.push(b' ');
            write_number(out, matrix[3]);
            out.push(b' ');
            write_number(out, matrix[4]);
            out.push(b' ');
            write_number(out, matrix[5]);
            out.extend_from_slice(b" cm\n");
        }
        ContentOperator::BeginText => out.extend_from_slice(b"BT\n"),
        ContentOperator::EndText => out.extend_from_slice(b"ET\n"),
        ContentOperator::SetFont { name, size } => {
            out.push(b'/');
            out.extend_from_slice(name.as_bytes());
            out.push(b' ');
            write_number(out, *size);
            out.extend_from_slice(b" Tf\n");
        }
        ContentOperator::SetCharSpacing(v) => {
            write_number(out, *v);
            out.extend_from_slice(b" Tc\n");
        }
        ContentOperator::SetWordSpacing(v) => {
            write_number(out, *v);
            out.extend_from_slice(b" Tw\n");
        }
        ContentOperator::SetHorizontalScaling(v) => {
            write_number(out, *v);
            out.extend_from_slice(b" Tz\n");
        }
        ContentOperator::SetTextLeading(v) => {
            write_number(out, *v);
            out.extend_from_slice(b" TL\n");
        }
        ContentOperator::SetTextRise(v) => {
            write_number(out, *v);
            out.extend_from_slice(b" Ts\n");
        }
        ContentOperator::MoveTextPosition { tx, ty } => {
            write_number(out, *tx);
            out.push(b' ');
            write_number(out, *ty);
            out.extend_from_slice(b" Td\n");
        }
        ContentOperator::MoveTextPositionSetLeading { tx, ty } => {
            write_number(out, *tx);
            out.push(b' ');
            write_number(out, *ty);
            out.extend_from_slice(b" TD\n");
        }
        ContentOperator::SetTextMatrix { matrix } => {
            write_number(out, matrix[0]);
            out.push(b' ');
            write_number(out, matrix[1]);
            out.push(b' ');
            write_number(out, matrix[2]);
            out.push(b' ');
            write_number(out, matrix[3]);
            out.push(b' ');
            write_number(out, matrix[4]);
            out.push(b' ');
            write_number(out, matrix[5]);
            out.extend_from_slice(b" Tm\n");
        }
        ContentOperator::NextLine => out.extend_from_slice(b"T*\n"),
        ContentOperator::ShowText { bytes } => {
            write_hex_string(out, bytes);
            out.extend_from_slice(b" Tj\n");
        }
        ContentOperator::ShowTextArray { elements } => {
            out.push(b'[');
            for elem in elements {
                match elem {
                    TJElement::Text(bytes) => write_hex_string(out, bytes),
                    TJElement::Offset(offset) => {
                        out.push(b' ');
                        write_number(out, *offset);
                        out.push(b' ');
                    }
                }
            }
            out.extend_from_slice(b"] TJ\n");
        }
        ContentOperator::NextLineShowText { bytes } => {
            write_hex_string(out, bytes);
            out.extend_from_slice(b" '\n");
        }
        ContentOperator::SetSpacingNextLineShowText {
            word_spacing,
            char_spacing,
            bytes,
        } => {
            write_number(out, *word_spacing);
            out.push(b' ');
            write_number(out, *char_spacing);
            out.push(b' ');
            write_hex_string(out, bytes);
            out.extend_from_slice(b" \"\n");
        }
        ContentOperator::MoveTo { x, y } => {
            write_number(out, *x);
            out.push(b' ');
            write_number(out, *y);
            out.extend_from_slice(b" m\n");
        }
        ContentOperator::LineTo { x, y } => {
            write_number(out, *x);
            out.push(b' ');
            write_number(out, *y);
            out.extend_from_slice(b" l\n");
        }
        ContentOperator::Rectangle { x, y, w, h } => {
            write_number(out, *x);
            out.push(b' ');
            write_number(out, *y);
            out.push(b' ');
            write_number(out, *w);
            out.push(b' ');
            write_number(out, *h);
            out.extend_from_slice(b" re\n");
        }
        ContentOperator::Stroke => out.extend_from_slice(b"S\n"),
        ContentOperator::CloseAndStroke => out.extend_from_slice(b"s\n"),
        ContentOperator::Fill => out.extend_from_slice(b"f\n"),
        ContentOperator::FillEvenOdd => out.extend_from_slice(b"f*\n"),
        ContentOperator::ClosePath => out.extend_from_slice(b"h\n"),
        ContentOperator::SetLineWidth(v) => {
            write_number(out, *v);
            out.extend_from_slice(b" w\n");
        }
        ContentOperator::RawOperator { raw } => {
            out.extend_from_slice(raw);
            out.push(b'\n');
        }
    }
}

/// Write a number in PDF-compatible format (no scientific notation).
fn write_number(out: &mut Vec<u8>, n: f64) {
    if n.is_nan() || n.is_infinite() {
        out.push(b'0');
        return;
    }
    if n.fract() == 0.0 && n.abs() < 1e15 {
        let _ = write!(out, "{}", n as i64);
    } else {
        let s = format!("{:.6}", n);
        let s = s.trim_end_matches('0').trim_end_matches('.');
        out.extend_from_slice(s.as_bytes());
    }
}

/// Write bytes as a PDF hex string: <4F6E...>
fn write_hex_string(out: &mut Vec<u8>, bytes: &[u8]) {
    out.push(b'<');
    for b in bytes {
        let _ = write!(out, "{:02X}", b);
    }
    out.push(b'>');
}
