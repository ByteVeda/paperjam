use crate::error::CliError;

/// Parse a page range string like "1-5,8,10-12" into a sorted, deduplicated list of page numbers.
pub fn parse_page_ranges(input: &str, total_pages: u32) -> Result<Vec<u32>, CliError> {
    let mut pages = Vec::new();

    for part in input.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let mut parts = part.splitn(2, '-');
            let start: u32 =
                parts.next().unwrap().trim().parse().map_err(|_| {
                    CliError::InvalidArgument(format!("Invalid page range: {}", part))
                })?;
            let end: u32 =
                parts.next().unwrap().trim().parse().map_err(|_| {
                    CliError::InvalidArgument(format!("Invalid page range: {}", part))
                })?;
            if start > end {
                return Err(CliError::InvalidArgument(format!(
                    "Invalid page range \"{}\": start must be <= end",
                    part
                )));
            }
            if end > total_pages {
                return Err(CliError::InvalidArgument(format!(
                    "Page {} out of range (document has {} pages)",
                    end, total_pages
                )));
            }
            for p in start..=end {
                pages.push(p);
            }
        } else {
            let p: u32 = part
                .parse()
                .map_err(|_| CliError::InvalidArgument(format!("Invalid page number: {}", part)))?;
            if p > total_pages || p == 0 {
                return Err(CliError::InvalidArgument(format!(
                    "Page {} out of range (document has {} pages)",
                    p, total_pages
                )));
            }
            pages.push(p);
        }
    }

    pages.sort();
    pages.dedup();
    Ok(pages)
}

/// Parse page ranges into (start, end) tuples for splitting.
pub fn parse_split_ranges(input: &str, total_pages: u32) -> Result<Vec<(u32, u32)>, CliError> {
    let mut ranges = Vec::new();

    for part in input.split(',') {
        let part = part.trim();
        if part.contains('-') {
            let mut parts = part.splitn(2, '-');
            let start: u32 = parts
                .next()
                .unwrap()
                .trim()
                .parse()
                .map_err(|_| CliError::InvalidArgument(format!("Invalid range: {}", part)))?;
            let end: u32 = parts
                .next()
                .unwrap()
                .trim()
                .parse()
                .map_err(|_| CliError::InvalidArgument(format!("Invalid range: {}", part)))?;
            if start > end || end > total_pages || start == 0 {
                return Err(CliError::InvalidArgument(format!(
                    "Invalid range \"{}\": must be within 1-{}",
                    part, total_pages
                )));
            }
            ranges.push((start, end));
        } else {
            let p: u32 = part
                .parse()
                .map_err(|_| CliError::InvalidArgument(format!("Invalid page: {}", part)))?;
            if p > total_pages || p == 0 {
                return Err(CliError::InvalidArgument(format!(
                    "Page {} out of range (1-{})",
                    p, total_pages
                )));
            }
            ranges.push((p, p));
        }
    }

    Ok(ranges)
}
