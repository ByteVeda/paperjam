use crate::error::Result;
use std::collections::HashMap;

/// A parsed ToUnicode CMap table mapping character codes to Unicode strings.
#[derive(Debug, Clone)]
pub struct CMapTable {
    mappings: HashMap<u32, String>,
}

impl CMapTable {
    /// Parse a ToUnicode CMap stream.
    pub fn parse(data: &[u8]) -> Result<Self> {
        let text = String::from_utf8_lossy(data);
        let mut mappings = HashMap::new();

        // Parse bfchar sections
        let mut chars = text.as_ref();
        while let Some(start) = chars.find("beginbfchar") {
            chars = &chars[start + "beginbfchar".len()..];
            if let Some(end) = chars.find("endbfchar") {
                let section = &chars[..end];
                parse_bfchar_section(section, &mut mappings);
                chars = &chars[end + "endbfchar".len()..];
            }
        }

        // Parse bfrange sections
        chars = text.as_ref();
        while let Some(start) = chars.find("beginbfrange") {
            chars = &chars[start + "beginbfrange".len()..];
            if let Some(end) = chars.find("endbfrange") {
                let section = &chars[..end];
                parse_bfrange_section(section, &mut mappings);
                chars = &chars[end + "endbfrange".len()..];
            }
        }

        Ok(CMapTable { mappings })
    }

    /// Look up a character code in the CMap.
    pub fn lookup(&self, code: u32) -> Option<&str> {
        self.mappings.get(&code).map(|s| s.as_str())
    }
}

fn parse_bfchar_section(section: &str, mappings: &mut HashMap<u32, String>) {
    let tokens = extract_hex_tokens(section);
    let mut i = 0;
    while i + 1 < tokens.len() {
        let code = parse_hex_to_u32(&tokens[i]);
        let unicode = hex_to_unicode_string(&tokens[i + 1]);
        mappings.insert(code, unicode);
        i += 2;
    }
}

fn parse_bfrange_section(section: &str, mappings: &mut HashMap<u32, String>) {
    let mut pos = 0;
    let bytes = section.as_bytes();

    while pos < bytes.len() {
        let start_code = match next_hex_token(bytes, &mut pos) {
            Some(t) => parse_hex_to_u32(&t),
            None => break,
        };

        let end_code = match next_hex_token(bytes, &mut pos) {
            Some(t) => parse_hex_to_u32(&t),
            None => break,
        };

        skip_ws(bytes, &mut pos);

        if pos < bytes.len() && bytes[pos] == b'[' {
            pos += 1;
            let mut values = Vec::new();
            while pos < bytes.len() && bytes[pos] != b']' {
                if let Some(t) = next_hex_token(bytes, &mut pos) {
                    values.push(hex_to_unicode_string(&t));
                } else {
                    skip_ws(bytes, &mut pos);
                    if pos < bytes.len() && bytes[pos] != b'<' && bytes[pos] != b']' {
                        pos += 1;
                    }
                }
            }
            if pos < bytes.len() {
                pos += 1;
            }
            for (i, val) in values.iter().enumerate() {
                mappings.insert(start_code + i as u32, val.clone());
            }
        } else {
            let unicode_start = match next_hex_token(bytes, &mut pos) {
                Some(t) => parse_hex_to_u32(&t),
                None => break,
            };
            for code in start_code..=end_code {
                let unicode_val = unicode_start + (code - start_code);
                if let Some(c) = char::from_u32(unicode_val) {
                    mappings.insert(code, c.to_string());
                }
            }
        }
    }
}

fn extract_hex_tokens(section: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut pos = 0;
    let bytes = section.as_bytes();
    while pos < bytes.len() {
        if let Some(t) = next_hex_token(bytes, &mut pos) {
            tokens.push(t);
        } else {
            pos += 1;
        }
    }
    tokens
}

fn next_hex_token(bytes: &[u8], pos: &mut usize) -> Option<String> {
    skip_ws(bytes, pos);
    if *pos >= bytes.len() || bytes[*pos] != b'<' {
        return None;
    }
    *pos += 1;
    let start = *pos;
    while *pos < bytes.len() && bytes[*pos] != b'>' {
        *pos += 1;
    }
    let token = String::from_utf8_lossy(&bytes[start..*pos]).to_string();
    if *pos < bytes.len() {
        *pos += 1;
    }
    Some(token)
}

fn skip_ws(bytes: &[u8], pos: &mut usize) {
    while *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
}

fn parse_hex_to_u32(hex: &str) -> u32 {
    u32::from_str_radix(hex.trim(), 16).unwrap_or(0)
}

fn hex_to_unicode_string(hex: &str) -> String {
    let hex = hex.trim();
    let bytes_vec: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|i| {
            let end = (i + 2).min(hex.len());
            u8::from_str_radix(&hex[i..end], 16).unwrap_or(0)
        })
        .collect();

    if bytes_vec.len() >= 2 {
        let code_units: Vec<u16> = bytes_vec
            .chunks(2)
            .map(|chunk| {
                if chunk.len() == 2 {
                    ((chunk[0] as u16) << 8) | (chunk[1] as u16)
                } else {
                    chunk[0] as u16
                }
            })
            .collect();
        String::from_utf16_lossy(&code_units)
    } else if bytes_vec.len() == 1 {
        char::from_u32(bytes_vec[0] as u32)
            .map(|c| c.to_string())
            .unwrap_or_default()
    } else {
        String::new()
    }
}
