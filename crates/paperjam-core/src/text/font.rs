use crate::error::Result;
use crate::page::obj_to_f64;
use crate::text::cmap::CMapTable;
use crate::text::encoding;

/// Decoded font information for text rendering.
#[derive(Debug)]
pub struct FontInfo {
    pub name: String,
    pub subtype: FontSubtype,
    pub encoding: FontEncoding,
    pub to_unicode: Option<CMapTable>,
    pub widths: Vec<f64>,
    pub first_char: u32,
    pub default_width: f64,
}

#[derive(Debug, Clone)]
pub enum FontSubtype {
    Type1,
    TrueType,
    Type0,
    Type3,
    CIDFontType0,
    CIDFontType2,
    Unknown(String),
}

#[derive(Debug, Clone)]
pub enum FontEncoding {
    Predefined(String),
    Differences {
        base: String,
        differences: Vec<(u8, String)>,
    },
    Identity,
    BuiltIn,
}

impl FontInfo {
    pub fn from_lopdf_dict(
        doc: &lopdf::Document,
        name: &str,
        dict: &lopdf::Dictionary,
    ) -> Result<Self> {
        let subtype = dict
            .get(b"Subtype")
            .ok()
            .and_then(|v| v.as_name().ok())
            .map(|s| match std::str::from_utf8(s).unwrap_or("") {
                "Type1" => FontSubtype::Type1,
                "TrueType" => FontSubtype::TrueType,
                "Type0" => FontSubtype::Type0,
                "Type3" => FontSubtype::Type3,
                "CIDFontType0" => FontSubtype::CIDFontType0,
                "CIDFontType2" => FontSubtype::CIDFontType2,
                other => FontSubtype::Unknown(other.to_string()),
            })
            .unwrap_or(FontSubtype::Unknown("unknown".to_string()));

        let font_encoding = parse_encoding(doc, dict);
        let to_unicode = parse_to_unicode(doc, dict);

        let first_char = dict
            .get(b"FirstChar")
            .ok()
            .and_then(|v| {
                let (_, v) = doc.dereference(v).unwrap_or((None, v));
                v.as_i64().ok()
            })
            .unwrap_or(0) as u32;

        let widths: Vec<f64> = dict
            .get(b"Widths")
            .ok()
            .and_then(|v| {
                let (_, v) = doc.dereference(v).unwrap_or((None, v));
                v.as_array().ok()
            })
            .map(|arr| {
                arr.iter()
                    .map(|v| {
                        let (_, v) = doc.dereference(v).unwrap_or((None, v));
                        obj_to_f64(v).unwrap_or(0.0)
                    })
                    .collect()
            })
            .unwrap_or_default();

        let default_width = if matches!(subtype, FontSubtype::Type0) {
            get_cid_default_width(doc, dict).unwrap_or(1000.0)
        } else {
            0.0
        };

        Ok(FontInfo {
            name: name.to_string(),
            subtype,
            encoding: font_encoding,
            to_unicode,
            widths,
            first_char,
            default_width,
        })
    }

    pub fn decode_bytes(&self, bytes: &[u8]) -> String {
        if let Some(ref cmap) = self.to_unicode {
            return self.decode_via_cmap(bytes, cmap);
        }
        match &self.encoding {
            FontEncoding::Predefined(name) => encoding::decode_predefined(bytes, name),
            FontEncoding::Differences { base, differences } => {
                encoding::decode_with_differences(bytes, base, differences)
            }
            FontEncoding::Identity => encoding::decode_utf16be(bytes),
            FontEncoding::BuiltIn => encoding::decode_predefined(bytes, "WinAnsiEncoding"),
        }
    }

    fn decode_via_cmap(&self, bytes: &[u8], cmap: &CMapTable) -> String {
        let mut result = String::new();
        match &self.subtype {
            FontSubtype::Type0 | FontSubtype::CIDFontType0 | FontSubtype::CIDFontType2 => {
                let mut i = 0;
                while i < bytes.len() {
                    if i + 1 < bytes.len() {
                        let code = ((bytes[i] as u32) << 8) | (bytes[i + 1] as u32);
                        if let Some(s) = cmap.lookup(code) {
                            result.push_str(s);
                            i += 2;
                            continue;
                        }
                    }
                    let code = bytes[i] as u32;
                    if let Some(s) = cmap.lookup(code) {
                        result.push_str(s);
                    }
                    i += 1;
                }
            }
            _ => {
                for &b in bytes {
                    if let Some(s) = cmap.lookup(b as u32) {
                        result.push_str(s);
                    } else {
                        result.push(b as char);
                    }
                }
            }
        }
        result
    }

    pub fn estimate_string_width(&self, bytes: &[u8], font_size: f64) -> f64 {
        let mut total_width = 0.0;
        match &self.subtype {
            FontSubtype::Type0 | FontSubtype::CIDFontType0 | FontSubtype::CIDFontType2 => {
                let mut i = 0;
                while i + 1 < bytes.len() {
                    let code = ((bytes[i] as u32) << 8) | (bytes[i + 1] as u32);
                    total_width += self.get_width(code);
                    i += 2;
                }
            }
            _ => {
                for &b in bytes {
                    total_width += self.get_width(b as u32);
                }
            }
        }
        total_width / 1000.0 * font_size
    }

    fn get_width(&self, code: u32) -> f64 {
        if code >= self.first_char {
            let idx = (code - self.first_char) as usize;
            if idx < self.widths.len() {
                return self.widths[idx];
            }
        }
        if self.default_width > 0.0 {
            self.default_width
        } else {
            600.0
        }
    }
}

fn parse_encoding(doc: &lopdf::Document, dict: &lopdf::Dictionary) -> FontEncoding {
    let enc = match dict.get(b"Encoding") {
        Ok(v) => v,
        Err(_) => return FontEncoding::BuiltIn,
    };

    let (_, enc) = doc.dereference(enc).unwrap_or((None, enc));

    if let Ok(name_bytes) = enc.as_name() {
        let name = String::from_utf8_lossy(name_bytes).to_string();
        if name == "Identity-H" || name == "Identity-V" {
            return FontEncoding::Identity;
        }
        return FontEncoding::Predefined(name);
    }

    if let Ok(enc_dict) = enc.as_dict() {
        let base = enc_dict
            .get(b"BaseEncoding")
            .ok()
            .and_then(|v| v.as_name().ok())
            .map(|s| String::from_utf8_lossy(s).to_string())
            .unwrap_or_else(|| "WinAnsiEncoding".to_string());

        let differences = enc_dict
            .get(b"Differences")
            .ok()
            .and_then(|v| v.as_array().ok())
            .map(|arr| parse_differences(arr))
            .unwrap_or_default();

        return FontEncoding::Differences { base, differences };
    }

    FontEncoding::BuiltIn
}

fn parse_differences(arr: &[lopdf::Object]) -> Vec<(u8, String)> {
    let mut result = Vec::new();
    let mut current_code: u8 = 0;
    for item in arr {
        match item {
            lopdf::Object::Integer(n) => current_code = *n as u8,
            lopdf::Object::Name(name) => {
                result.push((current_code, String::from_utf8_lossy(name).to_string()));
                current_code = current_code.wrapping_add(1);
            }
            _ => {}
        }
    }
    result
}

fn parse_to_unicode(doc: &lopdf::Document, dict: &lopdf::Dictionary) -> Option<CMapTable> {
    let tu_ref = dict.get(b"ToUnicode").ok()?;
    let (_, tu_obj) = doc.dereference(tu_ref).ok()?;
    let stream = tu_obj.as_stream().ok()?;
    let mut stream = stream.clone();
    let _ = stream.decompress();
    CMapTable::parse(&stream.content).ok()
}

fn get_cid_default_width(doc: &lopdf::Document, dict: &lopdf::Dictionary) -> Option<f64> {
    let descendants = dict.get(b"DescendantFonts").ok()?;
    let (_, descendants) = doc.dereference(descendants).unwrap_or((None, descendants));
    let arr = descendants.as_array().ok()?;
    let first = arr.first()?;
    let (_, desc_font) = doc.dereference(first).ok()?;
    let desc_dict = desc_font.as_dict().ok()?;
    desc_dict
        .get(b"DW")
        .ok()
        .and_then(|v| {
            let (_, v) = doc.dereference(v).unwrap_or((None, v));
            obj_to_f64(v).ok()
        })
}
