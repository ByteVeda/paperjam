use paperjam_model::metadata::Metadata;

use crate::document::EpubDocument;
use crate::error::EpubError;

impl EpubDocument {
    pub fn extract_metadata(&self) -> Result<Metadata, EpubError> {
        let m = &self.opf_metadata;
        Ok(Metadata {
            title: m.title.clone(),
            author: m.creator.clone(),
            subject: m.subject.clone().or(m.description.clone()),
            keywords: None,
            creator: m.publisher.clone(),
            producer: None,
            creation_date: m.date.clone(),
            modification_date: None,
            pdf_version: String::new(),
            page_count: self.chapters.len(),
            is_encrypted: false,
            xmp_metadata: None,
        })
    }
}
