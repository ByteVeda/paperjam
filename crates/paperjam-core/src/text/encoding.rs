/// Decode bytes using a predefined PDF encoding.
pub fn decode_predefined(bytes: &[u8], encoding_name: &str) -> String {
    match encoding_name {
        "WinAnsiEncoding" => decode_winansi(bytes),
        "MacRomanEncoding" => decode_macroman(bytes),
        "MacExpertEncoding" => decode_winansi(bytes), // Fallback
        "StandardEncoding" => decode_standard(bytes),
        _ => decode_winansi(bytes), // Default fallback
    }
}

/// Decode bytes using an encoding with a differences array.
pub fn decode_with_differences(bytes: &[u8], base: &str, differences: &[(u8, String)]) -> String {
    let mut result = String::new();

    for &b in bytes {
        // Check differences first
        if let Some((_, name)) = differences.iter().find(|(code, _)| *code == b) {
            if let Some(c) = glyph_name_to_char(name) {
                result.push(c);
                continue;
            }
        }

        // Fall back to base encoding
        let decoded = decode_predefined(&[b], base);
        result.push_str(&decoded);
    }

    result
}

/// Decode bytes as UTF-16BE.
pub fn decode_utf16be(bytes: &[u8]) -> String {
    if bytes.len() < 2 {
        return String::from_utf8_lossy(bytes).to_string();
    }

    let code_units: Vec<u16> = bytes
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
}

fn decode_winansi(bytes: &[u8]) -> String {
    // WinAnsiEncoding is essentially Windows-1252
    let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
    decoded.to_string()
}

fn decode_macroman(bytes: &[u8]) -> String {
    let (decoded, _, _) = encoding_rs::MACINTOSH.decode(bytes);
    decoded.to_string()
}

fn decode_standard(bytes: &[u8]) -> String {
    // StandardEncoding — mostly ASCII with some differences
    let mut result = String::new();
    for &b in bytes {
        let c = match b {
            0x20..=0x7E => b as char,
            0xA1 => '\u{00A1}', // exclamdown
            0xA2 => '\u{00A2}', // cent
            0xA3 => '\u{00A3}', // sterling
            0xA4 => '\u{2044}', // fraction
            0xA5 => '\u{00A5}', // yen
            0xA6 => '\u{0192}', // florin
            0xA7 => '\u{00A7}', // section
            0xAC => '\u{FB01}', // fi
            0xAD => '\u{FB02}', // fl
            0xB0 => '\u{2013}', // endash
            0xB7 => '\u{2022}', // bullet
            0xC6 => '\u{0152}', // OE
            0xE6 => '\u{0153}', // oe
            _ => {
                if b < 0x80 {
                    b as char
                } else {
                    '\u{FFFD}'
                }
            }
        };
        result.push(c);
    }
    result
}

/// Map a PostScript glyph name to a Unicode character.
fn glyph_name_to_char(name: &str) -> Option<char> {
    // Common glyph names — this covers the vast majority of cases
    match name {
        "space" => Some(' '),
        "exclam" => Some('!'),
        "quotedbl" => Some('"'),
        "numbersign" => Some('#'),
        "dollar" => Some('$'),
        "percent" => Some('%'),
        "ampersand" => Some('&'),
        "quotesingle" => Some('\''),
        "parenleft" => Some('('),
        "parenright" => Some(')'),
        "asterisk" => Some('*'),
        "plus" => Some('+'),
        "comma" => Some(','),
        "hyphen" | "minus" => Some('-'),
        "period" => Some('.'),
        "slash" => Some('/'),
        "zero" => Some('0'),
        "one" => Some('1'),
        "two" => Some('2'),
        "three" => Some('3'),
        "four" => Some('4'),
        "five" => Some('5'),
        "six" => Some('6'),
        "seven" => Some('7'),
        "eight" => Some('8'),
        "nine" => Some('9'),
        "colon" => Some(':'),
        "semicolon" => Some(';'),
        "less" => Some('<'),
        "equal" => Some('='),
        "greater" => Some('>'),
        "question" => Some('?'),
        "at" => Some('@'),
        "bracketleft" => Some('['),
        "backslash" => Some('\\'),
        "bracketright" => Some(']'),
        "asciicircum" => Some('^'),
        "underscore" => Some('_'),
        "grave" | "quoteleft" => Some('\u{2018}'),
        "braceleft" => Some('{'),
        "bar" => Some('|'),
        "braceright" => Some('}'),
        "asciitilde" => Some('~'),
        "quoteright" => Some('\u{2019}'),
        "quotedblleft" => Some('\u{201C}'),
        "quotedblright" => Some('\u{201D}'),
        "bullet" => Some('\u{2022}'),
        "endash" => Some('\u{2013}'),
        "emdash" => Some('\u{2014}'),
        "tilde" => Some('\u{02DC}'),
        "fi" => Some('\u{FB01}'),
        "fl" => Some('\u{FB02}'),
        "ellipsis" => Some('\u{2026}'),
        "copyright" => Some('\u{00A9}'),
        "registered" => Some('\u{00AE}'),
        "trademark" => Some('\u{2122}'),
        "degree" => Some('\u{00B0}'),
        // Common Latin-1 supplement
        "Agrave" => Some('\u{00C0}'),
        "Aacute" => Some('\u{00C1}'),
        "Acircumflex" => Some('\u{00C2}'),
        "Atilde" => Some('\u{00C3}'),
        "Adieresis" => Some('\u{00C4}'),
        "Aring" => Some('\u{00C5}'),
        "AE" => Some('\u{00C6}'),
        "Ccedilla" => Some('\u{00C7}'),
        "Egrave" => Some('\u{00C8}'),
        "Eacute" => Some('\u{00C9}'),
        "Ecircumflex" => Some('\u{00CA}'),
        "Edieresis" => Some('\u{00CB}'),
        "Igrave" => Some('\u{00CC}'),
        "Iacute" => Some('\u{00CD}'),
        "Icircumflex" => Some('\u{00CE}'),
        "Idieresis" => Some('\u{00CF}'),
        "Ntilde" => Some('\u{00D1}'),
        "Ograve" => Some('\u{00D2}'),
        "Oacute" => Some('\u{00D3}'),
        "Ocircumflex" => Some('\u{00D4}'),
        "Otilde" => Some('\u{00D5}'),
        "Odieresis" => Some('\u{00D6}'),
        "Ugrave" => Some('\u{00D9}'),
        "Uacute" => Some('\u{00DA}'),
        "Ucircumflex" => Some('\u{00DB}'),
        "Udieresis" => Some('\u{00DC}'),
        "agrave" => Some('\u{00E0}'),
        "aacute" => Some('\u{00E1}'),
        "acircumflex" => Some('\u{00E2}'),
        "atilde" => Some('\u{00E3}'),
        "adieresis" => Some('\u{00E4}'),
        "aring" => Some('\u{00E5}'),
        "ae" => Some('\u{00E6}'),
        "ccedilla" => Some('\u{00E7}'),
        "egrave" => Some('\u{00E8}'),
        "eacute" => Some('\u{00E9}'),
        "ecircumflex" => Some('\u{00EA}'),
        "edieresis" => Some('\u{00EB}'),
        "igrave" => Some('\u{00EC}'),
        "iacute" => Some('\u{00ED}'),
        "icircumflex" => Some('\u{00EE}'),
        "idieresis" => Some('\u{00EF}'),
        "ntilde" => Some('\u{00F1}'),
        "ograve" => Some('\u{00F2}'),
        "oacute" => Some('\u{00F3}'),
        "ocircumflex" => Some('\u{00F4}'),
        "otilde" => Some('\u{00F5}'),
        "odieresis" => Some('\u{00F6}'),
        "ugrave" => Some('\u{00F9}'),
        "uacute" => Some('\u{00FA}'),
        "ucircumflex" => Some('\u{00FB}'),
        "udieresis" => Some('\u{00FC}'),
        // Try parsing "uniXXXX" format
        _ => {
            if let Some(hex) = name.strip_prefix("uni") {
                u32::from_str_radix(hex, 16)
                    .ok()
                    .and_then(char::from_u32)
            } else if name.len() == 1 {
                name.chars().next()
            } else {
                // Single uppercase/lowercase letter names
                if name.len() == 1 {
                    name.chars().next()
                } else {
                    None
                }
            }
        }
    }
}
