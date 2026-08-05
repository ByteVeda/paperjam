use crate::document::SlideData;
use paperjam_model::structure::ContentBlock;

/// Convert parsed slides into a flat list of `ContentBlock` values.
pub fn slides_to_content_blocks(slides: &[SlideData]) -> Vec<ContentBlock> {
    let mut blocks = Vec::new();
    let zero_bbox = (0.0, 0.0, 0.0, 0.0);

    for slide in slides {
        let page = slide.index as u32;

        // Slide title -> Heading level 1
        if let Some(ref title) = slide.title {
            blocks.push(ContentBlock::Heading {
                text: title.clone(),
                level: 1,
                bbox: zero_bbox,
                page,
            });
        }

        // Body text blocks
        for block in &slide.text_blocks {
            if block.is_title {
                continue; // already emitted as heading
            }

            if block.is_bullet {
                blocks.push(ContentBlock::ListItem {
                    text: block.text.clone(),
                    indent_level: block.level,
                    bbox: zero_bbox,
                    page,
                });
            } else {
                blocks.push(ContentBlock::Paragraph {
                    text: block.text.clone(),
                    bbox: zero_bbox,
                    page,
                });
            }
        }

        // Tables
        for table in &slide.tables {
            blocks.push(ContentBlock::Table {
                table: table.clone(),
                page,
            });
        }
    }

    blocks
}
