//! Pure-Rust PDF engine: parsing, text and table extraction, page
//! manipulation, form fields, digital signatures, encryption, rendering,
//! and PDF/A / PDF/UA validation.
//!
//! `paperjam-core` is the PDF-specific implementation behind the
//! `paperjam` library. Non-PDF formats live in sibling crates
//! (`paperjam-docx`, `paperjam-xlsx`, `paperjam-pptx`, `paperjam-html`,
//! `paperjam-epub`); cross-format operations go through `paperjam-convert`.
//!
//! Heavy optional pieces are feature-gated:
//!
//! | Feature      | Enables                                                  |
//! |--------------|----------------------------------------------------------|
//! | `render`     | page-to-image rasterisation via pdfium                   |
//! | `signatures` | sign / verify / inspect digital signatures               |
//! | `ltv`        | long-term validation (TSA, OCSP, CRL embedding)          |
//! | `validation` | PDF/A and PDF/UA conformance checks                      |
//! | `parallel`   | rayon-based parallel processing (default on)             |
//! | `mmap`       | memory-mapped file access for large documents            |

pub mod annotations;
pub mod bookmarks;
#[cfg(feature = "validation")]
pub mod conversion;
pub mod diff;
pub mod document;
pub mod encryption;
pub mod error;
pub mod forms;
pub mod image;
pub mod io;
pub mod layout;
pub mod manipulation;
pub mod markdown;
pub mod metadata;
pub mod optimization;
pub mod page;
pub mod parallel;
pub mod redact;
#[cfg(feature = "render")]
pub mod render;
pub mod sanitize;
#[cfg(feature = "signatures")]
pub mod signature;
pub mod stamp;
pub mod structure;
pub mod table;
pub mod text;
pub mod toc;
#[cfg(feature = "validation")]
pub mod validation;
pub mod watermark;
