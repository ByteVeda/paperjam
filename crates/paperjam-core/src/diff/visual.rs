use crate::diff;
use crate::error::{PdfError, Result};
use crate::render::engine::render_page;
use crate::render::types::{ImageFormat, RenderOptions, RenderedImage};

pub use paperjam_model::visual_diff::*;

/// Perform a visual diff between two PDF documents.
pub fn visual_diff(
    doc_a_bytes: &[u8],
    doc_b_bytes: &[u8],
    doc_a: &crate::document::Document,
    doc_b: &crate::document::Document,
    options: &VisualDiffOptions,
    library_path: Option<&str>,
) -> Result<VisualDiffResult> {
    // Get text diff first
    let text_diff = diff::diff_documents(doc_a, doc_b)?;

    let render_opts = RenderOptions {
        dpi: options.dpi,
        format: ImageFormat::Png,
        quality: 85,
        background_color: Some([255, 255, 255]),
        scale_to_width: None,
        scale_to_height: None,
    };

    let count_a = doc_a.page_count() as u32;
    let count_b = doc_b.page_count() as u32;
    let max_pages = count_a.max(count_b);

    let mut pages = Vec::new();
    let mut total_pixels: u64 = 0;
    let mut total_changed: u64 = 0;

    for page_num in 1..=max_pages {
        let img_a = if page_num <= count_a {
            render_page(doc_a_bytes, page_num, &render_opts, library_path)?
        } else {
            // Create blank white image matching doc_b's page
            let img_b_temp = render_page(doc_b_bytes, page_num, &render_opts, library_path)?;
            create_blank_image(img_b_temp.width, img_b_temp.height, page_num)
        };

        let img_b = if page_num <= count_b {
            render_page(doc_b_bytes, page_num, &render_opts, library_path)?
        } else {
            create_blank_image(img_a.width, img_a.height, page_num)
        };

        // Compute pixel diff
        let (diff_image, similarity, changed_count) =
            compute_pixel_diff(&img_a, &img_b, options, page_num)?;

        let pixel_count = (img_a.width as u64) * (img_a.height as u64);
        total_pixels += pixel_count;
        total_changed += changed_count;

        pages.push(VisualDiffPage {
            page: page_num,
            image_a: img_a,
            image_b: img_b,
            diff_image,
            similarity,
            changed_pixel_count: changed_count,
        });
    }

    let overall_similarity = if total_pixels > 0 {
        1.0 - (total_changed as f64 / total_pixels as f64)
    } else {
        1.0
    };

    Ok(VisualDiffResult {
        pages,
        overall_similarity,
        text_diff,
    })
}

/// Compute pixel-level diff between two rendered images.
fn compute_pixel_diff(
    img_a: &RenderedImage,
    img_b: &RenderedImage,
    options: &VisualDiffOptions,
    page: u32,
) -> Result<(RenderedImage, f64, u64)> {
    // Decode PNG images to raw RGBA
    let rgba_a = decode_png_to_rgba(&img_a.data)?;
    let rgba_b = decode_png_to_rgba(&img_b.data)?;

    let width = img_a.width.min(img_b.width);
    let height = img_a.height.min(img_b.height);
    let pixel_count = (width as u64) * (height as u64);

    let mut diff_pixels = Vec::with_capacity((width * height * 4) as usize);
    let mut changed_count: u64 = 0;

    let w_a = img_a.width as usize;
    let w_b = img_b.width as usize;

    for y in 0..height as usize {
        for x in 0..width as usize {
            let idx_a = (y * w_a + x) * 4;
            let idx_b = (y * w_b + x) * 4;

            if idx_a + 3 >= rgba_a.len() || idx_b + 3 >= rgba_b.len() {
                diff_pixels.extend_from_slice(&[128, 128, 128, 255]);
                changed_count += 1;
                continue;
            }

            let r_diff = (rgba_a[idx_a] as i16 - rgba_b[idx_b] as i16).unsigned_abs() as u8;
            let g_diff = (rgba_a[idx_a + 1] as i16 - rgba_b[idx_b + 1] as i16).unsigned_abs() as u8;
            let b_diff = (rgba_a[idx_a + 2] as i16 - rgba_b[idx_b + 2] as i16).unsigned_abs() as u8;

            let is_changed = r_diff > options.threshold
                || g_diff > options.threshold
                || b_diff > options.threshold;

            if is_changed {
                diff_pixels.extend_from_slice(&options.highlight_color);
                changed_count += 1;
            } else {
                // Dimmed version of original
                let r = rgba_a[idx_a] / 2 + 128;
                let g = rgba_a[idx_a + 1] / 2 + 128;
                let b = rgba_a[idx_a + 2] / 2 + 128;
                diff_pixels.extend_from_slice(&[r, g, b, 255]);
            }
        }
    }

    // Encode diff image as PNG
    let diff_data = encode_rgba_png(&diff_pixels, width, height)?;

    let similarity = if pixel_count > 0 {
        1.0 - (changed_count as f64 / pixel_count as f64)
    } else {
        1.0
    };

    Ok((
        RenderedImage {
            data: diff_data,
            width,
            height,
            format: ImageFormat::Png,
            page,
        },
        similarity,
        changed_count,
    ))
}

/// Decode PNG bytes to raw RGBA pixel data.
fn decode_png_to_rgba(png_bytes: &[u8]) -> Result<Vec<u8>> {
    use image::ImageReader;
    use std::io::Cursor;

    let reader = ImageReader::new(Cursor::new(png_bytes))
        .with_guessed_format()
        .map_err(|e| PdfError::Render(format!("Failed to read image: {}", e)))?;

    let img = reader
        .decode()
        .map_err(|e| PdfError::Render(format!("Failed to decode image: {}", e)))?;

    Ok(img.to_rgba8().into_raw())
}

/// Encode raw RGBA pixels to PNG.
fn encode_rgba_png(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    use image::RgbaImage;
    use std::io::Cursor;

    let img = RgbaImage::from_raw(width, height, pixels.to_vec())
        .ok_or_else(|| PdfError::Render("Failed to create image from pixels".to_string()))?;

    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png)
        .map_err(|e| PdfError::Render(format!("PNG encode failed: {}", e)))?;

    Ok(buf.into_inner())
}

/// Create a blank white RenderedImage.
fn create_blank_image(width: u32, height: u32, page: u32) -> RenderedImage {
    use image::RgbaImage;
    use std::io::Cursor;

    let img = RgbaImage::from_pixel(width, height, image::Rgba([255, 255, 255, 255]));
    let mut buf = Cursor::new(Vec::new());
    let _ = img.write_to(&mut buf, image::ImageFormat::Png);

    RenderedImage {
        data: buf.into_inner(),
        width,
        height,
        format: ImageFormat::Png,
        page,
    }
}
