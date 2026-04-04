mod columns;
mod reading_order;
mod regions;

use crate::error::Result;
use crate::page::Page;

pub use paperjam_model::layout::*;

/// Analyze the layout of a single page.
pub fn analyze_layout(page: &Page, options: &LayoutOptions) -> Result<PageLayout> {
    let lines = page.text_lines()?;

    if lines.is_empty() {
        return Ok(PageLayout {
            page_width: page.width,
            page_height: page.height,
            column_count: 1,
            gutters: Vec::new(),
            regions: Vec::new(),
        });
    }

    // Step 1: Separate header/footer
    let (header, body, footer) = if options.detect_headers_footers {
        regions::separate_header_footer(
            &lines,
            page.height,
            options.header_zone_fraction,
            options.footer_zone_fraction,
        )
    } else {
        (Vec::new(), lines.iter().collect(), Vec::new())
    };

    // Step 2: Detect columns from body lines
    let extents = columns::extract_extents(&body.iter().map(|l| (*l).clone()).collect::<Vec<_>>());
    let profile = columns::build_x_projection(&extents, page.width, 1.0);
    let gutters = columns::find_gutters(
        &profile,
        1.0,
        options.min_gutter_width,
        page.width,
        body.len(),
    );
    let column_bands = columns::determine_columns(
        &gutters,
        &extents,
        page.width,
        options.min_column_line_fraction,
        options.max_columns,
    );

    // Step 3: Separate full-width lines from column lines
    let (full_width, col_lines) =
        regions::separate_full_width(&body, &gutters, page.width, &column_bands);

    // Step 4: Build reading order
    let region_list = reading_order::compute_reading_order(
        header,
        full_width,
        col_lines,
        footer,
        column_bands.len(),
    );

    Ok(PageLayout {
        page_width: page.width,
        page_height: page.height,
        column_count: column_bands.len(),
        gutters: gutters.iter().map(|g| g.center).collect(),
        regions: region_list,
    })
}

/// Analyze layout across all pages of a document.
pub fn analyze_document_layout(
    doc: &crate::document::Document,
    options: &LayoutOptions,
) -> Result<Vec<PageLayout>> {
    let mut layouts = Vec::new();
    for i in 1..=doc.page_count() as u32 {
        let page = doc.page(i)?;
        layouts.push(analyze_layout(&page, options)?);
    }
    Ok(layouts)
}
