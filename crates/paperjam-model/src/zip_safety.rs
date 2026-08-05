//! Shared hardened ZIP reader for the OOXML- and EPUB-family format
//! crates.
//!
//! ZIP-based document formats (DOCX, PPTX, XLSX, EPUB) are easy
//! decompression-bomb vectors: a malicious archive can declare a tiny
//! compressed size that expands to gigabytes, pack millions of trivially
//! empty entries, or pair a small compressed entry with a huge
//! decompressed size (compression-ratio attack).
//!
//! [`SafeArchive`] wraps a [`zip::ZipArchive`] and enforces four
//! independent caps across the lifetime of a single archive scan:
//!
//! 1. **Per-entry decompressed size** — no single entry can exceed
//!    [`ArchiveLimits::max_entry_bytes`].
//! 2. **Total decompressed bytes** — the running sum of bytes read
//!    across all entries cannot exceed [`ArchiveLimits::max_total_bytes`].
//! 3. **Entry count** — at most [`ArchiveLimits::max_entries`] entries
//!    will be decompressed.
//! 4. **Compression ratio** — declared decompressed size divided by
//!    compressed size cannot exceed [`ArchiveLimits::max_ratio`].
//!
//! Each check rejects with a structured [`ZipSafetyError`] so malicious
//! archives produce typed errors instead of an OOM.
//!
//! This module is compiled only when the `zip_safety` feature is
//! enabled, keeping `paperjam-model`'s default dependency surface empty.

use std::io::{Read, Seek};

/// Limits applied to a single archive scan. Tune individual fields as
/// needed; [`ArchiveLimits::DEFAULT`] is sized for typical office-family
/// documents and EPUBs.
#[derive(Debug, Clone, Copy)]
pub struct ArchiveLimits {
    /// Largest decompressed size permitted for any single entry, in
    /// bytes.
    pub max_entry_bytes: u64,
    /// Running total decompressed-byte budget across the whole scan.
    pub max_total_bytes: u64,
    /// Maximum number of entries [`SafeArchive`] will decompress.
    pub max_entries: usize,
    /// Maximum `entry.size() / entry.compressed_size()` ratio. Entries
    /// with `compressed_size() == 0` are exempt from this check and are
    /// governed by `max_entry_bytes` only.
    pub max_ratio: u64,
}

impl ArchiveLimits {
    /// Default limits tuned for ordinary office and EPUB documents.
    /// Real-world files fit comfortably; anything exceeding these is
    /// almost certainly pathological or malicious.
    pub const DEFAULT: Self = Self {
        max_entry_bytes: 100 * 1024 * 1024, // 100 MB
        max_total_bytes: 500 * 1024 * 1024, // 500 MB
        max_entries: 10_000,
        max_ratio: 100,
    };
}

impl Default for ArchiveLimits {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Errors surfaced by [`SafeArchive`].
///
/// Format crates typically wrap this with `#[from]` inside their own
/// error type.
#[derive(Debug, thiserror::Error)]
pub enum ZipSafetyError {
    /// Propagated from the underlying [`zip`] crate.
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Propagated from an I/O read on a ZIP entry.
    #[error("I/O error reading ZIP entry: {0}")]
    Io(#[from] std::io::Error),

    /// A named entry was not present in the archive.
    #[error("ZIP entry `{0}` not found")]
    MissingEntry(String),

    /// An entry was read but its bytes were not valid UTF-8.
    #[error("ZIP entry `{name}` is not valid UTF-8: {source}")]
    InvalidUtf8 {
        name: String,
        #[source]
        source: std::string::FromUtf8Error,
    },

    /// A single entry's decompressed size exceeded the per-entry cap.
    #[error("ZIP entry `{name}` is too large ({size} bytes, limit {limit})")]
    EntryTooLarge { name: String, size: u64, limit: u64 },

    /// The aggregate decompressed-byte budget was exhausted.
    #[error(
        "ZIP archive exceeded total decompressed-byte budget \
         (attempted {attempted} bytes, limit {limit})"
    )]
    TotalExceeded { attempted: u64, limit: u64 },

    /// The archive contained more entries than the scan will decompress.
    #[error("ZIP archive has too many entries (limit {limit})")]
    TooManyEntries { limit: usize },

    /// A single entry's declared compression ratio exceeds the cap.
    #[error(
        "ZIP entry `{name}` has compression ratio {ratio}x, \
         which exceeds the {limit}x safety cap"
    )]
    CompressionRatioExceeded {
        name: String,
        ratio: u64,
        limit: u64,
    },
}

/// A [`zip::ZipArchive`] wrapper that enforces [`ArchiveLimits`] across
/// every read.
///
/// `SafeArchive` borrows the underlying archive mutably and accumulates
/// running totals for the lifetime of the scan. A fresh scan needs a
/// fresh `SafeArchive`; reusing one across archives would leak state
/// between them.
pub struct SafeArchive<'a, R: Read + Seek> {
    archive: &'a mut zip::ZipArchive<R>,
    limits: ArchiveLimits,
    bytes_consumed: u64,
    entries_consumed: usize,
}

impl<'a, R: Read + Seek> SafeArchive<'a, R> {
    /// Wrap an archive with the given limits.
    pub fn new(archive: &'a mut zip::ZipArchive<R>, limits: ArchiveLimits) -> Self {
        Self {
            archive,
            limits,
            bytes_consumed: 0,
            entries_consumed: 0,
        }
    }

    /// Read a named entry and decode it as UTF-8 text.
    pub fn read_entry_string(&mut self, name: &str) -> Result<String, ZipSafetyError> {
        let bytes = self.read_entry_bytes(name)?;
        String::from_utf8(bytes).map_err(|source| ZipSafetyError::InvalidUtf8 {
            name: name.to_string(),
            source,
        })
    }

    /// Read a named entry as raw bytes.
    pub fn read_entry_bytes(&mut self, name: &str) -> Result<Vec<u8>, ZipSafetyError> {
        self.check_and_bump_entry_count()?;

        let mut entry = self
            .archive
            .by_name(name)
            .map_err(|_| ZipSafetyError::MissingEntry(name.to_string()))?;

        let declared = entry.size();
        let compressed = entry.compressed_size();
        check_entry_declared(name, declared, compressed, &self.limits)?;

        let remaining_budget = self
            .limits
            .max_total_bytes
            .saturating_sub(self.bytes_consumed);
        let per_read_cap = self.limits.max_entry_bytes.min(remaining_budget);

        // `+ 1` so we can distinguish "exactly at cap" from "lied about
        // size and actually exceeded"; without the extra byte the
        // attacker can silently truncate to the cap.
        let mut buf = Vec::with_capacity(declared.min(per_read_cap) as usize);
        let read = (&mut entry).take(per_read_cap + 1).read_to_end(&mut buf)? as u64;

        if read > per_read_cap {
            return Err(if read > self.limits.max_entry_bytes {
                ZipSafetyError::EntryTooLarge {
                    name: name.to_string(),
                    size: read,
                    limit: self.limits.max_entry_bytes,
                }
            } else {
                ZipSafetyError::TotalExceeded {
                    attempted: self.bytes_consumed.saturating_add(read),
                    limit: self.limits.max_total_bytes,
                }
            });
        }

        self.bytes_consumed = self.bytes_consumed.saturating_add(read);
        Ok(buf)
    }

    /// Total decompressed bytes read so far.
    pub fn bytes_consumed(&self) -> u64 {
        self.bytes_consumed
    }

    /// Number of entries decompressed so far.
    pub fn entries_consumed(&self) -> usize {
        self.entries_consumed
    }

    fn check_and_bump_entry_count(&mut self) -> Result<(), ZipSafetyError> {
        if self.entries_consumed >= self.limits.max_entries {
            return Err(ZipSafetyError::TooManyEntries {
                limit: self.limits.max_entries,
            });
        }
        self.entries_consumed += 1;
        Ok(())
    }
}

fn check_entry_declared(
    name: &str,
    declared: u64,
    compressed: u64,
    limits: &ArchiveLimits,
) -> Result<(), ZipSafetyError> {
    if declared > limits.max_entry_bytes {
        return Err(ZipSafetyError::EntryTooLarge {
            name: name.to_string(),
            size: declared,
            limit: limits.max_entry_bytes,
        });
    }
    if let Some(ratio) = declared.checked_div(compressed) {
        if ratio > limits.max_ratio {
            return Err(ZipSafetyError::CompressionRatioExceeded {
                name: name.to_string(),
                ratio,
                limit: limits.max_ratio,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};

    fn build_archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut w = zip::ZipWriter::new(Cursor::new(&mut buf));
            for (name, bytes) in entries {
                w.start_file::<_, ()>(*name, zip::write::SimpleFileOptions::default())
                    .unwrap();
                w.write_all(bytes).unwrap();
            }
            w.finish().unwrap();
        }
        buf
    }

    fn open(bytes: &[u8]) -> zip::ZipArchive<Cursor<&[u8]>> {
        zip::ZipArchive::new(Cursor::new(bytes)).unwrap()
    }

    #[test]
    fn reads_small_entry_successfully() {
        let bytes = build_archive(&[("hello.txt", b"hello world")]);
        let mut zip = open(&bytes);
        let mut safe = SafeArchive::new(&mut zip, ArchiveLimits::DEFAULT);
        assert_eq!(safe.read_entry_string("hello.txt").unwrap(), "hello world");
        assert_eq!(safe.entries_consumed(), 1);
        assert_eq!(safe.bytes_consumed(), 11);
    }

    #[test]
    fn missing_entry_returns_structured_error() {
        let bytes = build_archive(&[("a", b"x")]);
        let mut zip = open(&bytes);
        let mut safe = SafeArchive::new(&mut zip, ArchiveLimits::DEFAULT);
        let err = safe.read_entry_bytes("not_there").unwrap_err();
        assert!(matches!(err, ZipSafetyError::MissingEntry(name) if name == "not_there"));
    }

    #[test]
    fn rejects_entry_larger_than_per_entry_cap() {
        let limits = ArchiveLimits {
            max_entry_bytes: 16,
            ..ArchiveLimits::DEFAULT
        };
        let bytes = build_archive(&[("big.bin", &[b'a'; 32])]);
        let mut zip = open(&bytes);
        let mut safe = SafeArchive::new(&mut zip, limits);
        assert!(matches!(
            safe.read_entry_bytes("big.bin"),
            Err(ZipSafetyError::EntryTooLarge { .. })
        ));
    }

    #[test]
    fn rejects_when_running_total_exceeds_budget() {
        let limits = ArchiveLimits {
            max_total_bytes: 20,
            ..ArchiveLimits::DEFAULT
        };
        let bytes = build_archive(&[("a", &[b'x'; 16]), ("b", &[b'y'; 16])]);
        let mut zip = open(&bytes);
        let mut safe = SafeArchive::new(&mut zip, limits);
        safe.read_entry_bytes("a").unwrap();
        assert!(matches!(
            safe.read_entry_bytes("b"),
            Err(ZipSafetyError::TotalExceeded { .. })
        ));
    }

    #[test]
    fn rejects_when_entry_count_exceeded() {
        let limits = ArchiveLimits {
            max_entries: 1,
            ..ArchiveLimits::DEFAULT
        };
        let bytes = build_archive(&[("a", b"x"), ("b", b"y")]);
        let mut zip = open(&bytes);
        let mut safe = SafeArchive::new(&mut zip, limits);
        safe.read_entry_bytes("a").unwrap();
        assert!(matches!(
            safe.read_entry_bytes("b"),
            Err(ZipSafetyError::TooManyEntries { .. })
        ));
    }

    #[test]
    fn rejects_compression_ratio_bomb() {
        // Deflated 1 MB of a single byte compresses ~1000x; with the
        // ratio cap at 10x that must be rejected.
        let payload = vec![b'a'; 1_000_000];
        let bytes = {
            let mut buf = Vec::new();
            {
                let mut w = zip::ZipWriter::new(Cursor::new(&mut buf));
                let options = zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated);
                w.start_file::<_, ()>("bomb.bin", options).unwrap();
                w.write_all(&payload).unwrap();
                w.finish().unwrap();
            }
            buf
        };
        let limits = ArchiveLimits {
            max_ratio: 10,
            ..ArchiveLimits::DEFAULT
        };
        let mut zip = open(&bytes);
        let mut safe = SafeArchive::new(&mut zip, limits);
        assert!(matches!(
            safe.read_entry_bytes("bomb.bin"),
            Err(ZipSafetyError::CompressionRatioExceeded { .. })
        ));
    }

    #[test]
    fn non_utf8_bytes_return_structured_error() {
        let bytes = build_archive(&[("bad", &[0xff, 0xfe, 0xfd])]);
        let mut zip = open(&bytes);
        let mut safe = SafeArchive::new(&mut zip, ArchiveLimits::DEFAULT);
        let err = safe.read_entry_string("bad").unwrap_err();
        assert!(matches!(err, ZipSafetyError::InvalidUtf8 { .. }));
    }
}
