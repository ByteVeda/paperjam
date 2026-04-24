//! Bounded readers for ZIP entries to mitigate decompression-bomb attacks.

use std::io::{Read, Seek};

use crate::error::{PptxError, Result};

/// Per-entry decompressed byte limit. Typical slide XML is tens of KB; even
/// large decks rarely exceed a few MB per entry.
pub const MAX_ENTRY_BYTES: u64 = 100 * 1024 * 1024;

pub fn read_entry_string<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> Result<String> {
    let mut entry = archive
        .by_name(name)
        .map_err(|_| PptxError::MissingEntry(name.to_string()))?;

    let declared = entry.size();
    if declared > MAX_ENTRY_BYTES {
        return Err(PptxError::EntryTooLarge {
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
        return Err(PptxError::EntryTooLarge {
            name: name.to_string(),
            size: read as u64,
            limit: MAX_ENTRY_BYTES,
        });
    }
    Ok(buf)
}
