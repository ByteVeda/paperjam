# Installation

## Requirements

- Python 3.12 or later
- A supported platform: Linux (x86-64, ARM64), macOS (x86-64, Apple Silicon), Windows (x86-64)

paperjam ships pre-built wheels for all major platforms. No Rust toolchain is required.

## Install from PyPI

```bash
pip install paperjam
```

## Optional extras

### pandas integration

If you want to call `table.to_dataframe()` you need pandas:

```bash
pip install "paperjam[pandas]"
```

### Documentation

The docs site uses [Docusaurus](https://docusaurus.io/). To build it locally:

```bash
git clone https://github.com/ByteVeda/paperjam
cd paperjam/docs-site
npm ci
npm run start   # dev server with hot reload
npm run build   # static site under docs-site/build/
```

## Installing from source

Building from source requires a Rust toolchain (stable, 1.85+) and [maturin](https://maturin.rs/):

```bash
pip install maturin
git clone https://github.com/ByteVeda/paperjam
cd paperjam
maturin develop --release
```

## Feature flags

The Rust core exposes optional features that affect which methods are available at runtime.
Pre-built wheels on PyPI include all features.

| Feature | Methods enabled |
|---------|----------------|
| `render` | `render_page`, `render_pages`, `page.render`, `visual_diff` |
| `signatures` | `sign_document`, `verify_signatures`, `extract_signatures` |
| `ltv` | LTV timestamp embedding (TSA, OCSP, CRL) for signing |
| `validation` | `validate_pdf_a`, `validate_pdf_ua`, `convert_to_pdf_a` |
| `parallel` | Rayon-based parallel processing (default) |
| `mmap` | Memory-mapped file access for large documents |

When building from source you can control features with the `--features` flag:

```bash
maturin develop --release --features render,signatures
```

## CLI Installation

Install the paperjam command-line tool:

```bash
# From crates.io
cargo install paperjam-cli

# From source
git clone https://github.com/ByteVeda/paperjam
cd paperjam
cargo build --release -p paperjam-cli
```

The CLI binary is called `pj`:

```bash
pj --help
pj info document.pdf
```

## MCP Server Setup

To use paperjam as an MCP server for AI assistants (Claude Code, Cursor):

```bash
# Build the MCP server
cargo build --release -p paperjam-mcp
```

Add to your Claude Code or Cursor MCP configuration:

```json
{
  "mcpServers": {
    "paperjam": {
      "command": "paperjam-mcp",
      "args": ["--working-dir", "/path/to/documents"]
    }
  }
}
```

## Verifying the installation

```python
import paperjam
print(paperjam.__version__)
```
