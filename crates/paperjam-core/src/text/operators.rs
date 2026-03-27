use crate::error::{PdfError, Result};

/// PDF content stream operators relevant to text and graphics state.
#[derive(Debug, Clone)]
pub enum ContentOperator {
    // Graphics state
    SaveGraphicsState,
    RestoreGraphicsState,
    ConcatMatrix {
        matrix: [f64; 6],
    },

    // Text state
    BeginText,
    EndText,
    SetFont {
        name: String,
        size: f64,
    },
    SetCharSpacing(f64),
    SetWordSpacing(f64),
    SetHorizontalScaling(f64),
    SetTextLeading(f64),
    SetTextRise(f64),

    // Text positioning
    MoveTextPosition {
        tx: f64,
        ty: f64,
    },
    MoveTextPositionSetLeading {
        tx: f64,
        ty: f64,
    },
    SetTextMatrix {
        matrix: [f64; 6],
    },
    NextLine,

    // Text showing
    ShowText {
        bytes: Vec<u8>,
    },
    ShowTextArray {
        elements: Vec<TJElement>,
    },
    NextLineShowText {
        bytes: Vec<u8>,
    },
    SetSpacingNextLineShowText {
        word_spacing: f64,
        char_spacing: f64,
        bytes: Vec<u8>,
    },

    // Path operators (for table line detection)
    MoveTo {
        x: f64,
        y: f64,
    },
    LineTo {
        x: f64,
        y: f64,
    },
    Rectangle {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    },
    Stroke,
    CloseAndStroke,
    Fill,
    FillEvenOdd,
    ClosePath,
    SetLineWidth(f64),

    /// Unrecognized operator preserved as raw bytes for lossless round-trip.
    RawOperator {
        raw: Vec<u8>,
    },
}

#[derive(Debug, Clone)]
pub enum TJElement {
    Text(Vec<u8>),
    Offset(f64),
}

/// Text state maintained during content stream interpretation.
#[derive(Debug, Clone)]
pub struct TextState {
    pub text_matrix: [f64; 6],
    pub text_line_matrix: [f64; 6],
    pub font_name: String,
    pub font_size: f64,
    pub char_spacing: f64,
    pub word_spacing: f64,
    pub horizontal_scaling: f64,
    pub leading: f64,
    pub rise: f64,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            text_matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            text_line_matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            font_name: String::new(),
            font_size: 0.0,
            char_spacing: 0.0,
            word_spacing: 0.0,
            horizontal_scaling: 1.0,
            leading: 0.0,
            rise: 0.0,
        }
    }
}

/// Parse a PDF content stream into a sequence of operators.
pub fn parse_content_stream(bytes: &[u8]) -> Result<Vec<ContentOperator>> {
    let mut ops = Vec::new();
    let mut operand_stack: Vec<Operand> = Vec::new();
    let mut pos = 0;
    let mut operand_group_start: usize = 0;

    while pos < bytes.len() {
        skip_whitespace(bytes, &mut pos);
        if pos >= bytes.len() {
            break;
        }

        // Track start position of the current operand group
        if operand_stack.is_empty() {
            operand_group_start = pos;
        }

        let b = bytes[pos];

        // Comment
        if b == b'%' {
            while pos < bytes.len() && bytes[pos] != b'\n' && bytes[pos] != b'\r' {
                pos += 1;
            }
            continue;
        }

        // String literal (text bytes)
        if b == b'(' {
            let s = parse_string_literal(bytes, &mut pos)?;
            operand_stack.push(Operand::Bytes(s));
            continue;
        }

        // Hex string
        if b == b'<' && pos + 1 < bytes.len() && bytes[pos + 1] != b'<' {
            let s = parse_hex_string(bytes, &mut pos)?;
            operand_stack.push(Operand::Bytes(s));
            continue;
        }

        // Dictionary (skip, we don't need inline dicts in content streams typically)
        if b == b'<' && pos + 1 < bytes.len() && bytes[pos + 1] == b'<' {
            skip_dictionary(bytes, &mut pos);
            continue;
        }

        // Array
        if b == b'[' {
            let arr = parse_array(bytes, &mut pos)?;
            operand_stack.push(Operand::Array(arr));
            continue;
        }

        // Number
        if b == b'-' || b == b'+' || b == b'.' || b.is_ascii_digit() {
            let num = parse_number(bytes, &mut pos)?;
            operand_stack.push(Operand::Number(num));
            continue;
        }

        // Name (e.g., /FontName)
        if b == b'/' {
            let name = parse_name(bytes, &mut pos);
            operand_stack.push(Operand::Name(name));
            continue;
        }

        // Operator keyword
        if b.is_ascii_alphabetic() || b == b'\'' || b == b'"' || b == b'*' {
            let keyword = parse_keyword(bytes, &mut pos);

            if let Some(op) = build_operator(&keyword, &mut operand_stack) {
                ops.push(op);
            } else {
                // Preserve unrecognized operator with its operands as raw bytes
                ops.push(ContentOperator::RawOperator {
                    raw: bytes[operand_group_start..pos].to_vec(),
                });
            }
            operand_stack.clear();
            continue;
        }

        pos += 1; // Skip unknown bytes
    }

    Ok(ops)
}

#[derive(Debug, Clone)]
enum Operand {
    Number(f64),
    Bytes(Vec<u8>),
    Name(String),
    Array(Vec<Operand>),
}

impl Operand {
    fn as_number(&self) -> f64 {
        match self {
            Operand::Number(n) => *n,
            _ => 0.0,
        }
    }

    fn as_bytes(&self) -> Vec<u8> {
        match self {
            Operand::Bytes(b) => b.clone(),
            _ => Vec::new(),
        }
    }

    fn as_name(&self) -> String {
        match self {
            Operand::Name(n) => n.clone(),
            _ => String::new(),
        }
    }
}

fn build_operator(keyword: &str, stack: &mut [Operand]) -> Option<ContentOperator> {
    match keyword {
        // Graphics state
        "q" => Some(ContentOperator::SaveGraphicsState),
        "Q" => Some(ContentOperator::RestoreGraphicsState),
        "cm" if stack.len() >= 6 => {
            let f = stack[0].as_number();
            let matrix = [
                stack[stack.len() - 6].as_number(),
                stack[stack.len() - 5].as_number(),
                stack[stack.len() - 4].as_number(),
                stack[stack.len() - 3].as_number(),
                stack[stack.len() - 2].as_number(),
                stack[stack.len() - 1].as_number(),
            ];
            let _ = f;
            Some(ContentOperator::ConcatMatrix { matrix })
        }

        // Text state
        "BT" => Some(ContentOperator::BeginText),
        "ET" => Some(ContentOperator::EndText),
        "Tf" if stack.len() >= 2 => {
            let name = stack[stack.len() - 2].as_name();
            let size = stack[stack.len() - 1].as_number();
            Some(ContentOperator::SetFont { name, size })
        }
        "Tc" if !stack.is_empty() => {
            Some(ContentOperator::SetCharSpacing(stack.last()?.as_number()))
        }
        "Tw" if !stack.is_empty() => {
            Some(ContentOperator::SetWordSpacing(stack.last()?.as_number()))
        }
        "Tz" if !stack.is_empty() => Some(ContentOperator::SetHorizontalScaling(
            stack.last()?.as_number(),
        )),
        "TL" if !stack.is_empty() => {
            Some(ContentOperator::SetTextLeading(stack.last()?.as_number()))
        }
        "Ts" if !stack.is_empty() => Some(ContentOperator::SetTextRise(stack.last()?.as_number())),

        // Text positioning
        "Td" if stack.len() >= 2 => Some(ContentOperator::MoveTextPosition {
            tx: stack[stack.len() - 2].as_number(),
            ty: stack[stack.len() - 1].as_number(),
        }),
        "TD" if stack.len() >= 2 => Some(ContentOperator::MoveTextPositionSetLeading {
            tx: stack[stack.len() - 2].as_number(),
            ty: stack[stack.len() - 1].as_number(),
        }),
        "Tm" if stack.len() >= 6 => {
            let matrix = [
                stack[stack.len() - 6].as_number(),
                stack[stack.len() - 5].as_number(),
                stack[stack.len() - 4].as_number(),
                stack[stack.len() - 3].as_number(),
                stack[stack.len() - 2].as_number(),
                stack[stack.len() - 1].as_number(),
            ];
            Some(ContentOperator::SetTextMatrix { matrix })
        }
        "T*" => Some(ContentOperator::NextLine),

        // Text showing
        "Tj" if !stack.is_empty() => Some(ContentOperator::ShowText {
            bytes: stack.last()?.as_bytes(),
        }),
        "TJ" if !stack.is_empty() => {
            let elements = match stack.last()? {
                Operand::Array(arr) => arr
                    .iter()
                    .map(|item| match item {
                        Operand::Number(n) => TJElement::Offset(*n),
                        Operand::Bytes(b) => TJElement::Text(b.clone()),
                        _ => TJElement::Text(Vec::new()),
                    })
                    .collect(),
                _ => return None,
            };
            Some(ContentOperator::ShowTextArray { elements })
        }
        "'" if !stack.is_empty() => Some(ContentOperator::NextLineShowText {
            bytes: stack.last()?.as_bytes(),
        }),
        "\"" if stack.len() >= 3 => Some(ContentOperator::SetSpacingNextLineShowText {
            word_spacing: stack[stack.len() - 3].as_number(),
            char_spacing: stack[stack.len() - 2].as_number(),
            bytes: stack[stack.len() - 1].as_bytes(),
        }),

        // Path operators
        "m" if stack.len() >= 2 => Some(ContentOperator::MoveTo {
            x: stack[stack.len() - 2].as_number(),
            y: stack[stack.len() - 1].as_number(),
        }),
        "l" if stack.len() >= 2 => Some(ContentOperator::LineTo {
            x: stack[stack.len() - 2].as_number(),
            y: stack[stack.len() - 1].as_number(),
        }),
        "re" if stack.len() >= 4 => Some(ContentOperator::Rectangle {
            x: stack[stack.len() - 4].as_number(),
            y: stack[stack.len() - 3].as_number(),
            w: stack[stack.len() - 2].as_number(),
            h: stack[stack.len() - 1].as_number(),
        }),
        "S" => Some(ContentOperator::Stroke),
        "s" => Some(ContentOperator::CloseAndStroke),
        "f" | "F" => Some(ContentOperator::Fill),
        "f*" => Some(ContentOperator::FillEvenOdd),
        "h" => Some(ContentOperator::ClosePath),
        "w" if !stack.is_empty() => Some(ContentOperator::SetLineWidth(stack.last()?.as_number())),

        _ => None,
    }
}

// --- Parsing helpers ---

fn skip_whitespace(bytes: &[u8], pos: &mut usize) {
    while *pos < bytes.len() {
        match bytes[*pos] {
            b' ' | b'\t' | b'\n' | b'\r' | b'\x00' | b'\x0c' => *pos += 1,
            _ => break,
        }
    }
}

fn parse_number(bytes: &[u8], pos: &mut usize) -> Result<f64> {
    let start = *pos;

    if *pos < bytes.len() && (bytes[*pos] == b'-' || bytes[*pos] == b'+') {
        *pos += 1;
    }

    while *pos < bytes.len() && (bytes[*pos].is_ascii_digit() || bytes[*pos] == b'.') {
        *pos += 1;
    }

    let s = std::str::from_utf8(&bytes[start..*pos]).unwrap_or("0");
    s.parse::<f64>().map_err(|_| PdfError::Parse {
        message: format!("Invalid number: '{}'", s),
        offset: Some(start as u64),
    })
}

fn parse_string_literal(bytes: &[u8], pos: &mut usize) -> Result<Vec<u8>> {
    assert_eq!(bytes[*pos], b'(');
    *pos += 1;

    let mut result = Vec::new();
    let mut depth = 1;

    while *pos < bytes.len() && depth > 0 {
        let b = bytes[*pos];
        match b {
            b'(' => {
                depth += 1;
                result.push(b);
            }
            b')' => {
                depth -= 1;
                if depth > 0 {
                    result.push(b);
                }
            }
            b'\\' => {
                *pos += 1;
                if *pos < bytes.len() {
                    match bytes[*pos] {
                        b'n' => result.push(b'\n'),
                        b'r' => result.push(b'\r'),
                        b't' => result.push(b'\t'),
                        b'b' => result.push(8),  // backspace
                        b'f' => result.push(12), // form feed
                        b'(' => result.push(b'('),
                        b')' => result.push(b')'),
                        b'\\' => result.push(b'\\'),
                        c if c.is_ascii_digit() => {
                            // Octal escape
                            let mut octal = (c - b'0') as u16;
                            for _ in 0..2 {
                                if *pos + 1 < bytes.len() && bytes[*pos + 1].is_ascii_digit() {
                                    *pos += 1;
                                    octal = octal * 8 + (bytes[*pos] - b'0') as u16;
                                } else {
                                    break;
                                }
                            }
                            result.push(octal as u8);
                        }
                        _ => result.push(bytes[*pos]),
                    }
                }
            }
            _ => result.push(b),
        }
        *pos += 1;
    }

    Ok(result)
}

fn parse_hex_string(bytes: &[u8], pos: &mut usize) -> Result<Vec<u8>> {
    assert_eq!(bytes[*pos], b'<');
    *pos += 1;

    let mut hex = Vec::new();
    while *pos < bytes.len() && bytes[*pos] != b'>' {
        let b = bytes[*pos];
        if b.is_ascii_hexdigit() {
            hex.push(b);
        }
        *pos += 1;
    }
    if *pos < bytes.len() {
        *pos += 1; // Skip '>'
    }

    // Pad with 0 if odd number of hex digits
    if hex.len() % 2 != 0 {
        hex.push(b'0');
    }

    let mut result = Vec::with_capacity(hex.len() / 2);
    for chunk in hex.chunks(2) {
        let hi = hex_digit(chunk[0]);
        let lo = hex_digit(chunk[1]);
        result.push((hi << 4) | lo);
    }

    Ok(result)
}

fn hex_digit(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}

fn parse_name(bytes: &[u8], pos: &mut usize) -> String {
    assert_eq!(bytes[*pos], b'/');
    *pos += 1;

    let start = *pos;
    while *pos < bytes.len() {
        let b = bytes[*pos];
        if b.is_ascii_whitespace()
            || b == b'/'
            || b == b'('
            || b == b')'
            || b == b'<'
            || b == b'>'
            || b == b'['
            || b == b']'
            || b == b'{'
            || b == b'}'
        {
            break;
        }
        *pos += 1;
    }

    String::from_utf8_lossy(&bytes[start..*pos]).to_string()
}

fn parse_keyword(bytes: &[u8], pos: &mut usize) -> String {
    let start = *pos;
    while *pos < bytes.len() {
        let b = bytes[*pos];
        if b.is_ascii_whitespace() || b == b'/' || b == b'(' || b == b'<' || b == b'[' || b == b'{'
        {
            break;
        }
        // Allow * in keywords like f* and T*
        if b == b'*' {
            *pos += 1;
            break;
        }
        if !b.is_ascii_alphanumeric() && b != b'\'' && b != b'"' {
            break;
        }
        *pos += 1;
    }

    String::from_utf8_lossy(&bytes[start..*pos]).to_string()
}

fn parse_array(bytes: &[u8], pos: &mut usize) -> Result<Vec<Operand>> {
    assert_eq!(bytes[*pos], b'[');
    *pos += 1;

    let mut items = Vec::new();

    while *pos < bytes.len() {
        skip_whitespace(bytes, pos);
        if *pos >= bytes.len() {
            break;
        }

        let b = bytes[*pos];
        if b == b']' {
            *pos += 1;
            break;
        }

        if b == b'(' {
            items.push(Operand::Bytes(parse_string_literal(bytes, pos)?));
        } else if b == b'<' && *pos + 1 < bytes.len() && bytes[*pos + 1] != b'<' {
            items.push(Operand::Bytes(parse_hex_string(bytes, pos)?));
        } else if b == b'-' || b == b'+' || b == b'.' || b.is_ascii_digit() {
            items.push(Operand::Number(parse_number(bytes, pos)?));
        } else if b == b'/' {
            items.push(Operand::Name(parse_name(bytes, pos)));
        } else {
            *pos += 1; // Skip unknown
        }
    }

    Ok(items)
}

fn skip_dictionary(bytes: &[u8], pos: &mut usize) {
    if *pos + 1 < bytes.len() && bytes[*pos] == b'<' && bytes[*pos + 1] == b'<' {
        *pos += 2;
        let mut depth = 1;
        while *pos + 1 < bytes.len() && depth > 0 {
            if bytes[*pos] == b'<' && bytes[*pos + 1] == b'<' {
                depth += 1;
                *pos += 2;
            } else if bytes[*pos] == b'>' && bytes[*pos + 1] == b'>' {
                depth -= 1;
                *pos += 2;
            } else {
                *pos += 1;
            }
        }
    }
}
