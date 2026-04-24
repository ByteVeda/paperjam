//! Format-agnostic types and traits shared across the paperjam workspace.
//!
//! Holds the stable data model — bookmarks, metadata, tables, text layout,
//! annotations, structure blocks — plus the `DocumentTrait` that every
//! format crate (`paperjam-docx`, `paperjam-xlsx`, ...) implements.
//!
//! This crate intentionally has no format-specific dependencies, so
//! downstream crates can depend on it without pulling in parsers they
//! do not use.

pub mod document;
pub mod format;

#[cfg(feature = "zip_safety")]
pub mod zip_safety;

pub mod annotations;
pub mod bookmarks;
pub mod conversion;
pub mod diff;
pub mod encryption;
pub mod forms;
pub mod image;
pub mod layout;
pub mod manipulation;
pub mod markdown;
pub mod metadata;
pub mod optimization;
pub mod redact;
pub mod render;
pub mod sanitize;
pub mod signature;
pub mod stamp;
pub mod structure;
pub mod table;
pub mod text;
pub mod toc;
pub mod validation;
pub mod visual_diff;
pub mod watermark;
