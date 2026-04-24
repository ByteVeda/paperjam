use calamine::{Data, Reader, Xlsx};

use crate::document::{SheetData, XlsxDocument};
use crate::error::XlsxError;

/// Read an XLSX workbook from raw bytes into an [`XlsxDocument`].
pub fn read_xlsx(bytes: &[u8]) -> Result<XlsxDocument, XlsxError> {
    let cursor = std::io::Cursor::new(bytes);
    let mut workbook: Xlsx<_> = Xlsx::new(cursor)?;

    let sheet_names = workbook.sheet_names().to_vec();
    // Cap the initial allocation — sheet_names comes from the workbook
    // header and is attacker-controlled.
    let mut sheets = Vec::with_capacity(sheet_names.len().min(1024));

    for name in &sheet_names {
        if let Ok(range) = workbook.worksheet_range(name) {
            let rows: Vec<Vec<String>> = range
                .rows()
                .map(|row| row.iter().map(cell_to_string).collect())
                .collect();
            sheets.push(SheetData {
                name: name.clone(),
                rows,
            });
        }
    }

    Ok(XlsxDocument {
        sheet_names,
        sheets,
    })
}

/// Convert a calamine [`Data`] cell value into a display string.
fn cell_to_string(data: &Data) -> String {
    match data {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => format_float(*f),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => format!("{dt}"),
        Data::Error(e) => format!("#ERR:{e:?}"),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
    }
}

/// Format a float, stripping unnecessary trailing zeros.
fn format_float(f: f64) -> String {
    if f.fract() == 0.0 && f.abs() < i64::MAX as f64 {
        format!("{}", f as i64)
    } else {
        format!("{f}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_float_integer() {
        assert_eq!(format_float(42.0), "42");
    }

    #[test]
    fn format_float_decimal() {
        assert_eq!(format_float(3.15), "3.15");
    }

    #[test]
    fn cell_to_string_empty() {
        assert_eq!(cell_to_string(&Data::Empty), "");
    }

    #[test]
    fn cell_to_string_bool() {
        assert_eq!(cell_to_string(&Data::Bool(true)), "true");
    }
}
