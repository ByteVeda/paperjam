use rust_xlsxwriter::Workbook;

use crate::document::XlsxDocument;
use crate::error::XlsxError;

/// Serialize an [`XlsxDocument`] back to XLSX bytes.
pub fn write_xlsx(doc: &XlsxDocument) -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();

    for sheet in &doc.sheets {
        let worksheet = workbook.add_worksheet();
        worksheet.set_name(&sheet.name)?;

        for (row_idx, row) in sheet.rows.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                worksheet.write_string(row_idx as u32, col_idx as u16, cell)?;
            }
        }
    }

    let bytes = workbook.save_to_buffer()?;
    Ok(bytes)
}
