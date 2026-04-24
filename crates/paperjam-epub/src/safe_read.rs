//! Bounded readers for ZIP entries to mitigate decompression-bomb attacks.
//!
//! EPUB files are ZIP archives; malicious inputs can declare tiny compressed
//! sizes that expand to gigabytes. We cap the decompressed length on every
//! entry we pull out of the archive.

use std::io::Read;

use crate::error::{EpubError, Result};

/// Per-entry decompressed byte limit. Normal EPUB entries (XHTML chapters,
/// cover images, fonts) are comfortably under this. A document that exceeds
/// it is either pathological or malicious.
pub const MAX_ENTRY_BYTES: u64 = 100 * 1024 * 1024;

pub fn read_entry_string(
    archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
    name: &str,
) -> Result<String> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| EpubError::MissingEntry(name.to_string()))?;

    let declared = entry.size();
    if declared > MAX_ENTRY_BYTES {
        return Err(EpubError::EntryTooLarge {
            name: name.to_string(),
            size: declared,
            limit: MAX_ENTRY_BYTES,
        });
    }

    let mut buf = String::new();
    let read = (&mut entry)
        .take(MAX_ENTRY_BYTES + 1)
        .read_to_string(&mut buf)?;
    if read as u64 > MAX_ENTRY_BYTES {
        return Err(EpubError::EntryTooLarge {
            name: name.to_string(),
            size: read as u64,
            limit: MAX_ENTRY_BYTES,
        });
    }
    Ok(buf)
}

pub fn read_entry_bytes(
    archive: &mut zip::ZipArchive<std::io::Cursor<&[u8]>>,
    name: &str,
) -> Result<Vec<u8>> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| EpubError::MissingEntry(name.to_string()))?;

    let declared = entry.size();
    if declared > MAX_ENTRY_BYTES {
        return Err(EpubError::EntryTooLarge {
            name: name.to_string(),
            size: declared,
            limit: MAX_ENTRY_BYTES,
        });
    }

    let mut buf = Vec::new();
    let read = (&mut entry)
        .take(MAX_ENTRY_BYTES + 1)
        .read_to_end(&mut buf)?;
    if read as u64 > MAX_ENTRY_BYTES {
        return Err(EpubError::EntryTooLarge {
            name: name.to_string(),
            size: read as u64,
            limit: MAX_ENTRY_BYTES,
        });
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{Cursor, Write};

    fn build_archive_with_entry(name: &str, contents: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut w = zip::ZipWriter::new(Cursor::new(&mut buf));
            w.start_file::<_, ()>(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            w.write_all(contents).unwrap();
            w.finish().unwrap();
        }
        buf
    }

    #[test]
    fn small_entry_reads_normally() {
        let bytes = build_archive_with_entry("hello.txt", b"hello world");
        let bytes_slice: &[u8] = &bytes;
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes_slice)).unwrap();
        let s = read_entry_string(&mut archive, "hello.txt").unwrap();
        assert_eq!(s, "hello world");
    }

    #[test]
    fn oversized_entry_is_rejected_by_declared_size() {
        // Build an archive whose single entry exceeds the per-entry cap.
        let blob = vec![b'a'; (MAX_ENTRY_BYTES as usize) + 1];
        let bytes = build_archive_with_entry("big.bin", &blob);
        let bytes_slice: &[u8] = &bytes;
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes_slice)).unwrap();
        let err = read_entry_bytes(&mut archive, "big.bin").unwrap_err();
        assert!(matches!(err, EpubError::EntryTooLarge { .. }));
    }
}
