use crate::bookmarks::{set_bookmarks, BookmarkSpec};
use crate::document::Document;
use crate::error::Result;
use crate::structure::{self, ContentBlock, StructureOptions};

/// Options for automatic table-of-contents generation.
pub struct TocOptions {
    /// Maximum heading depth to include (1-6, default 6).
    pub max_depth: u8,
    /// Font size ratio for heading detection (passed to StructureOptions).
    pub heading_size_ratio: f64,
    /// Whether to use layout-aware reading order.
    pub layout_aware: bool,
    /// If true, replace existing bookmarks. If false, append.
    pub replace_existing: bool,
}

impl Default for TocOptions {
    fn default() -> Self {
        Self {
            max_depth: 6,
            heading_size_ratio: 1.2,
            layout_aware: false,
            replace_existing: true,
        }
    }
}

/// Generate a table of contents from document heading structure and inject as bookmarks.
///
/// Returns the new document with bookmarks set and the list of bookmark specs generated.
pub fn generate_toc(doc: &Document, options: &TocOptions) -> Result<(Document, Vec<BookmarkSpec>)> {
    let structure_opts = StructureOptions {
        heading_size_ratio: options.heading_size_ratio,
        detect_lists: false,
        include_tables: false,
        layout_aware: options.layout_aware,
    };

    let blocks = structure::extract_document_structure(doc, &structure_opts)?;

    // Filter headings within max_depth
    let headings: Vec<(&str, u8, u32)> = blocks
        .iter()
        .filter_map(|block| {
            if let ContentBlock::Heading {
                text, level, page, ..
            } = block
            {
                if *level <= options.max_depth {
                    Some((text.as_str(), *level, *page))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    if headings.is_empty() {
        // No headings found; if replace_existing, clear bookmarks; otherwise no-op
        if options.replace_existing {
            let new_doc = set_bookmarks(doc, &[])?;
            return Ok((new_doc, Vec::new()));
        }
        // Return a clone with no changes
        let new_doc = Document::from_lopdf(doc.inner().clone())?;
        return Ok((new_doc, Vec::new()));
    }

    // Build hierarchical BookmarkSpec tree from flat heading list using a stack
    let specs = build_hierarchy(&headings);

    // Set bookmarks
    if options.replace_existing {
        let new_doc = set_bookmarks(doc, &specs)?;
        Ok((new_doc, specs))
    } else {
        // Append: get existing bookmarks and merge
        let existing = doc.bookmarks()?;
        let mut all_specs: Vec<BookmarkSpec> = existing
            .iter()
            .map(|b| BookmarkSpec {
                title: b.title.clone(),
                page: b.page as u32,
                children: Vec::new(),
            })
            .collect();
        all_specs.extend(specs.clone());
        let new_doc = set_bookmarks(doc, &all_specs)?;
        Ok((new_doc, specs))
    }
}

/// Build a hierarchical bookmark tree from a flat list of (text, level, page) headings.
fn build_hierarchy(headings: &[(&str, u8, u32)]) -> Vec<BookmarkSpec> {
    if headings.is_empty() {
        return Vec::new();
    }

    let mut result: Vec<BookmarkSpec> = Vec::new();
    let mut stack: Vec<(u8, usize)> = Vec::new(); // (level, index in parent's children)

    for &(text, level, page) in headings {
        let spec = BookmarkSpec {
            title: text.to_string(),
            page,
            children: Vec::new(),
        };

        // Pop stack until we find a parent with lower level
        while let Some(&(parent_level, _)) = stack.last() {
            if parent_level >= level {
                stack.pop();
            } else {
                break;
            }
        }

        if stack.is_empty() {
            // Top-level item
            result.push(spec);
            let idx = result.len() - 1;
            stack.push((level, idx));
        } else {
            // Nested: add as child of the current stack top
            // Navigate to the right parent
            let parent = get_spec_mut(&mut result, &stack);
            parent.children.push(spec);
            let child_idx = parent.children.len() - 1;
            stack.push((level, child_idx));
        }
    }

    result
}

/// Navigate the tree to get a mutable reference to the spec at the given stack path.
fn get_spec_mut<'a>(roots: &'a mut [BookmarkSpec], stack: &[(u8, usize)]) -> &'a mut BookmarkSpec {
    let mut current = &mut roots[stack[0].1];
    for &(_, idx) in &stack[1..] {
        current = &mut current.children[idx];
    }
    current
}
