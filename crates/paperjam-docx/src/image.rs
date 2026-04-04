use paperjam_model::image::ImageInfo;

use crate::document::DocxDocument;
use crate::error::DocxError;

impl DocxDocument {
    /// Extract embedded images from the document.
    ///
    /// `docx-rs` exposes images read from the ZIP archive through
    /// `Docx::images` (a `Vec<(id, path, Image, Png)>` populated by the
    /// reader).  We return each image's raw bytes as an `ImageInfo`.
    pub fn extract_images(&self) -> Result<Vec<ImageInfo>, DocxError> {
        let mut images = Vec::new();
        for (_id, _path, raw_image, _png) in &self.inner.images {
            images.push(ImageInfo {
                width: 0,
                height: 0,
                color_space: None,
                bits_per_component: None,
                filters: Vec::new(),
                data: raw_image.0.clone(),
            });
        }
        Ok(images)
    }
}
