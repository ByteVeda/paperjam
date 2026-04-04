use paperjam_model::image::ImageInfo;

use crate::document::EpubDocument;
use crate::error::EpubError;

impl EpubDocument {
    pub fn extract_images(&self) -> Result<Vec<ImageInfo>, EpubError> {
        let mut images = Vec::new();

        // Images from the archive (actual files in the ZIP).
        for (path, data) in &self.archive_images {
            let lower = path.to_ascii_lowercase();
            let color_space = if lower.ends_with(".png")
                || lower.ends_with(".gif")
                || lower.ends_with(".jpg")
                || lower.ends_with(".jpeg")
            {
                Some("RGB".to_string())
            } else {
                None
            };

            images.push(ImageInfo {
                width: 0,
                height: 0,
                color_space,
                bits_per_component: Some(8),
                filters: vec![path.clone()],
                data: data.clone(),
            });
        }

        // Inline data URI images from HTML chapters.
        for ch in &self.chapters {
            let html_images = paperjam_html::image::extract_images_from_html(ch.html.dom());
            // Only include images that have actual data (data URIs).
            for img in html_images {
                if !img.data.is_empty() {
                    images.push(img);
                }
            }
        }

        Ok(images)
    }
}
