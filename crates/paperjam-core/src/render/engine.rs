use std::cell::RefCell;
use std::io::Cursor;

use image::ImageFormat as ImgFormat;
use pdfium_render::prelude::*;

use crate::error::{PdfError, Result};
use crate::render::types::{ImageFormat, RenderOptions, RenderedImage};

thread_local! {
    static PDFIUM: RefCell<Option<(Option<String>, Pdfium)>> = const { RefCell::new(None) };
}

/// Access a cached pdfium instance, creating one if needed.
///
/// The instance is cached per-thread. If `library_path` changes, the old
/// instance is dropped and a new one is created.
fn with_pdfium<F, R>(library_path: Option<&str>, f: F) -> Result<R>
where
    F: FnOnce(&Pdfium) -> Result<R>,
{
    PDFIUM.with(|cell| {
        let mut opt = cell.borrow_mut();
        let needs_reload = match (&*opt, library_path) {
            (Some((cached, _)), path) if cached.as_deref() == path => false,
            (Some(_), _) => true,
            (None, _) => true,
        };
        if needs_reload && opt.is_some() {
            *opt = None;
        }
        if opt.is_none() {
            let bindings = if let Some(path) = library_path {
                Pdfium::bind_to_library(path).or_else(|_| Pdfium::bind_to_system_library())
            } else {
                Pdfium::bind_to_system_library()
            }
            .map_err(|e| PdfError::Render(format!("Failed to load pdfium library: {}", e)))?;
            *opt = Some((library_path.map(String::from), Pdfium::new(bindings)));
        }
        f(&opt.as_ref().unwrap().1)
    })
}

/// Render a single page from PDF bytes.
pub fn render_page(
    pdf_bytes: &[u8],
    page_number: u32,
    options: &RenderOptions,
    library_path: Option<&str>,
) -> Result<RenderedImage> {
    with_pdfium(library_path, |pdfium| {
        let document = pdfium
            .load_pdf_from_byte_slice(pdf_bytes, None)
            .map_err(|e| PdfError::Render(format!("Failed to load PDF: {}", e)))?;

        let page_count = document.pages().len() as usize;
        let page_index = (page_number - 1) as u16;
        if page_index as usize >= page_count {
            return Err(PdfError::PageOutOfRange {
                page: page_number as usize,
                total: page_count,
            });
        }

        let page = document
            .pages()
            .get(page_index)
            .map_err(|e| PdfError::Render(format!("Failed to get page {}: {}", page_number, e)))?;

        render_pdfium_page(&page, page_number, options)
    })
}

/// Render multiple pages from PDF bytes.
pub fn render_pages(
    pdf_bytes: &[u8],
    page_numbers: Option<&[u32]>,
    options: &RenderOptions,
    library_path: Option<&str>,
) -> Result<Vec<RenderedImage>> {
    // Resolve page list and validate up front
    let page_count = with_pdfium(library_path, |pdfium| {
        let document = pdfium
            .load_pdf_from_byte_slice(pdf_bytes, None)
            .map_err(|e| PdfError::Render(format!("Failed to load PDF: {}", e)))?;
        Ok(document.pages().len() as usize)
    })?;

    let pages_to_render: Vec<u32> = match page_numbers {
        Some(nums) => nums.to_vec(),
        None => (1..=page_count as u32).collect(),
    };

    for &page_num in &pages_to_render {
        if page_num as usize > page_count || page_num == 0 {
            return Err(PdfError::PageOutOfRange {
                page: page_num as usize,
                total: page_count,
            });
        }
    }

    render_pages_impl(pdf_bytes, &pages_to_render, options, library_path)
}

/// Choose parallel or sequential rendering based on feature flag and page count.
fn render_pages_impl(
    pdf_bytes: &[u8],
    pages: &[u32],
    options: &RenderOptions,
    library_path: Option<&str>,
) -> Result<Vec<RenderedImage>> {
    #[cfg(feature = "parallel")]
    {
        if pages.len() > 4 {
            return render_pages_parallel(pdf_bytes, pages, options, library_path);
        }
    }
    render_pages_sequential(pdf_bytes, pages, options, library_path)
}

/// Parallel multi-page rendering — each rayon thread gets its own pdfium instance
/// via `thread_local!` and re-parses the PDF bytes.
#[cfg(feature = "parallel")]
fn render_pages_parallel(
    pdf_bytes: &[u8],
    pages: &[u32],
    options: &RenderOptions,
    library_path: Option<&str>,
) -> Result<Vec<RenderedImage>> {
    use rayon::prelude::*;

    let results: Vec<Result<RenderedImage>> = pages
        .par_iter()
        .map(|&page_num| render_page(pdf_bytes, page_num, options, library_path))
        .collect();

    results.into_iter().collect()
}

/// Sequential multi-page rendering (single pdfium instance).
fn render_pages_sequential(
    pdf_bytes: &[u8],
    pages: &[u32],
    options: &RenderOptions,
    library_path: Option<&str>,
) -> Result<Vec<RenderedImage>> {
    with_pdfium(library_path, |pdfium| {
        let document = pdfium
            .load_pdf_from_byte_slice(pdf_bytes, None)
            .map_err(|e| PdfError::Render(format!("Failed to load PDF: {}", e)))?;

        let mut results = Vec::with_capacity(pages.len());
        for &page_num in pages {
            let page_index = (page_num - 1) as u16;
            let page = document
                .pages()
                .get(page_index)
                .map_err(|e| PdfError::Render(format!("Failed to get page {}: {}", page_num, e)))?;

            results.push(render_pdfium_page(&page, page_num, options)?);
        }

        Ok(results)
    })
}

/// Render a single pdfium page to a RenderedImage.
fn render_pdfium_page(
    page: &PdfPage<'_>,
    page_number: u32,
    options: &RenderOptions,
) -> Result<RenderedImage> {
    let width_pts = page.width().value;
    let height_pts = page.height().value;

    // Determine pixel dimensions based on scale_to_width/height or DPI
    let (pixel_width, pixel_height) = match (options.scale_to_width, options.scale_to_height) {
        (Some(tw), Some(th)) => {
            // Fit within both constraints (preserve aspect ratio)
            let scale_w = tw as f32 / width_pts;
            let scale_h = th as f32 / height_pts;
            let scale = scale_w.min(scale_h);
            ((width_pts * scale) as u32, (height_pts * scale) as u32)
        }
        (Some(tw), None) => {
            let scale = tw as f32 / width_pts;
            (tw, (height_pts * scale) as u32)
        }
        (None, Some(th)) => {
            let scale = th as f32 / height_pts;
            ((width_pts * scale) as u32, th)
        }
        (None, None) => {
            let scale = options.dpi / 72.0;
            ((width_pts * scale) as u32, (height_pts * scale) as u32)
        }
    };

    let mut config = PdfRenderConfig::new()
        .set_target_width(pixel_width as i32)
        .set_target_height(pixel_height as i32);

    if let Some([r, g, b]) = options.background_color {
        config = config.set_clear_color(PdfColor::new(r, g, b, 255));
    }

    let bitmap = page
        .render_with_config(&config)
        .map_err(|e| PdfError::Render(format!("Render failed: {}", e)))?;

    let img = bitmap.as_image().into_rgba8();

    let data = encode_image(&img, options)?;

    Ok(RenderedImage {
        data,
        width: pixel_width,
        height: pixel_height,
        format: options.format,
        page: page_number,
    })
}

/// Encode an image buffer to the requested format.
fn encode_image(img: &image::RgbaImage, options: &RenderOptions) -> Result<Vec<u8>> {
    // Pre-allocate based on estimated compressed size
    let estimated = (img.width() * img.height()) as usize;
    let mut buf = Cursor::new(Vec::with_capacity(estimated));

    match options.format {
        ImageFormat::Png => {
            img.write_to(&mut buf, ImgFormat::Png)
                .map_err(|e| PdfError::Render(format!("PNG encode failed: {}", e)))?;
        }
        ImageFormat::Jpeg => {
            let mut encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, options.quality);
            encoder
                .encode_image(img)
                .map_err(|e| PdfError::Render(format!("JPEG encode failed: {}", e)))?;
        }
        ImageFormat::Bmp => {
            img.write_to(&mut buf, ImgFormat::Bmp)
                .map_err(|e| PdfError::Render(format!("BMP encode failed: {}", e)))?;
        }
    }

    Ok(buf.into_inner())
}
