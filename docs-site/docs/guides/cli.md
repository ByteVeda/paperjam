---
sidebar_position: 16
title: CLI Reference
---

# CLI Reference

`pj` is the paperjam command-line tool. It exposes every library feature as a subcommand, with structured JSON output for scripting.

## Installation

```bash
# From crates.io
cargo install paperjam-cli

# From source
git clone https://github.com/ByteVeda/paperjam.git
cd paperjam
cargo install --path crates/paperjam-cli
```

## Global options

These flags work with every subcommand:

| Flag | Description |
|------|-------------|
| `--format json` | Output structured JSON instead of human-readable text |
| `--password <pw>` | Password for encrypted PDFs |
| `--quiet` | Suppress non-essential output |
| `--help` | Show help for any command or subcommand |

## Info

Display document metadata, page count, and structural summary:

```bash
pj info report.pdf
pj info report.pdf --format json
```

## Extract

### Text

```bash
pj extract text report.pdf
pj extract text report.pdf --pages 1-5
pj extract text report.pdf --layout-aware
```

### Tables

```bash
pj extract tables report.pdf
pj extract tables report.pdf --strategy auto --min-rows 2 --min-cols 2
pj extract tables report.pdf --output-format csv > tables.csv
```

### Structure

```bash
pj extract structure report.pdf
pj extract structure report.pdf --heading-size-ratio 1.3 --detect-lists
```

### Bookmarks

```bash
pj extract bookmarks report.pdf
```

### Metadata

```bash
pj extract metadata report.pdf
pj extract metadata report.pdf --format json
```

## Convert

Convert between document formats. `auto` infers formats from file extensions:

```bash
# Auto-detect
pj convert auto input.docx -o output.pdf

# Explicit target format
pj convert to-pdf report.docx -o report.pdf
pj convert to-docx report.pdf -o report.docx
pj convert to-xlsx data.pdf -o data.xlsx
pj convert to-html report.pdf -o report.html
pj convert to-epub article.html -o article.epub

# Markdown
pj convert markdown report.pdf -o report.md
pj convert markdown report.pdf -o report.md --layout-aware --include-page-numbers
```

## Manipulate

### Merge

```bash
pj merge cover.pdf body.pdf appendix.pdf -o complete.pdf
```

### Split

```bash
# Split by page ranges
pj split report.pdf --ranges 1-5,6-10 -o "part_{n}.pdf"

# Split into individual pages
pj split report.pdf --each-page -o "page_{n}.pdf"
```

### Rotate

```bash
pj rotate report.pdf --pages 1:90,3:180 -o rotated.pdf
```

### Reorder

```bash
pj reorder report.pdf --order 3,1,2,4,5 -o reordered.pdf
```

### Delete pages

```bash
pj delete report.pdf --pages 2,4,6 -o trimmed.pdf
```

### Watermark

```bash
pj watermark report.pdf --text "DRAFT" -o watermarked.pdf
pj watermark report.pdf --text "CONFIDENTIAL" --opacity 0.2 --rotation 45 --font-size 72 -o watermarked.pdf
```

### Stamp

```bash
pj stamp report.pdf --stamp letterhead.pdf --layer under -o stamped.pdf
```

## Security

### Redact

```bash
# Redact literal text
pj redact report.pdf --text "John Smith" -o redacted.pdf

# Redact with regex
pj redact report.pdf --regex '\b\d{3}-\d{2}-\d{4}\b' -o redacted.pdf

# Case-insensitive
pj redact report.pdf --text "confidential" --ignore-case -o redacted.pdf
```

### Sanitize

```bash
pj sanitize untrusted.pdf -o safe.pdf
pj sanitize untrusted.pdf --keep-links -o safe.pdf
```

### Encrypt

```bash
pj encrypt report.pdf --user-password "open" --owner-password "admin" -o encrypted.pdf
pj encrypt report.pdf --user-password "open" --algorithm aes256 -o encrypted.pdf
```

### Sign

```bash
pj sign contract.pdf --key private.pem --cert chain.pem -o signed.pdf
pj sign contract.pdf --key private.pem --cert chain.pem --tsa-url https://freetsa.org/tsr -o signed.pdf
```

### Verify signatures

```bash
pj verify signed.pdf
pj verify signed.pdf --format json
```

## Validate

```bash
# PDF/A validation
pj validate report.pdf --standard pdf-a --level 1b
pj validate report.pdf --standard pdf-a --level 2b

# PDF/UA accessibility validation
pj validate report.pdf --standard pdf-ua

# JSON output for CI pipelines
pj validate report.pdf --standard pdf-a --format json
```

## Pipeline

```bash
# Run a pipeline
pj pipeline run workflow.yaml

# Run with parallelism
pj pipeline run workflow.yaml --parallel --workers 8

# Override input/output
pj pipeline run workflow.yaml --input "docs/*.pdf" --output-dir results/

# Validate a pipeline definition
pj pipeline validate workflow.yaml
```

See the [Pipeline Engine guide](pipeline) for the full YAML format and step reference.

## JSON output mode

Every command supports `--format json` for machine-readable output. This makes `pj` easy to integrate into shell scripts and CI pipelines:

```bash
# Document info as JSON
pj info report.pdf --format json | jq '.page_count'

# Validation in CI
result=$(pj validate report.pdf --standard pdf-a --format json)
compliant=$(echo "$result" | jq -r '.is_compliant')
if [ "$compliant" != "true" ]; then
    echo "PDF/A validation failed"
    echo "$result" | jq '.issues[]'
    exit 1
fi

# Extract tables as JSON
pj extract tables invoice.pdf --format json | jq '.tables[0].rows'
```
