---
sidebar_position: 17
title: MCP Server
---

# MCP Server

paperjam ships with an MCP (Model Context Protocol) server, making it the first document processing library with native AI-agent support. Any MCP-compatible client -- Claude Code, Claude Desktop, Cursor, or custom agents -- can open, extract from, convert, and manipulate documents through a standard tool interface.

## What is MCP?

The Model Context Protocol is an open standard for connecting AI models to external tools and data sources. Instead of writing custom glue code for each AI integration, you register an MCP server and the model discovers its capabilities automatically.

## Installation

```bash
# Run directly (no install needed)
uvx paperjam-mcp

# Or install globally
pip install paperjam-mcp
```

Verify the installation:

```bash
paperjam-mcp --version
```

## Configuration

### Claude Code

Add the server to your project's `.mcp.json`:

```json
{
  "mcpServers": {
    "paperjam": {
      "command": "uvx",
      "args": ["paperjam-mcp", "--working-dir", "."]
    }
  }
}
```

### Claude Desktop

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "paperjam": {
      "command": "uvx",
      "args": ["paperjam-mcp", "--working-dir", "/Users/you/Documents"]
    }
  }
}
```

### Cursor

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "paperjam": {
      "command": "uvx",
      "args": ["paperjam-mcp", "--working-dir", "."]
    }
  }
}
```

### Server options

| Flag | Default | Description |
|------|---------|-------------|
| `--working-dir` | `.` | Base directory for resolving relative file paths. All file access is sandboxed to this directory. |
| `--transport` | `stdio` | Transport: `stdio` or `sse` |
| `--port` | `8080` | Port for SSE transport |
| `--max-sessions` | `50` | Maximum concurrent document sessions |
| `--session-ttl` | `3600` | Session time-to-live in seconds (resets on each access) |
| `--log-level` | `warning` | Logging verbosity: `debug`, `info`, `warning`, `error` |

## Available tools

Once connected, the AI model can call these tools through the MCP protocol.

### Document management

| Tool | Description |
|------|-------------|
| `open_document` | Open a document by path. Returns a session ID. |
| `get_document_info` | Get page count, metadata, format, and structural summary. |
| `save_document` | Save the current document state to disk. |
| `close_document` | Close a session and free resources. |
| `list_sessions` | List all open document sessions. |

### Extraction

| Tool | Description |
|------|-------------|
| `extract_text` | Extract plain text from all pages. |
| `extract_tables` | Extract tables as structured data (rows, headers, cells). |
| `extract_structure` | Extract headings, paragraphs, and list items. |
| `to_markdown` | Convert the document to Markdown. |
| `search_document` | Full-text search with regex support (PDF only). |
| `extract_links` | Extract all hyperlinks (PDF only). |
| `extract_images` | Extract image metadata from a page (PDF only). |
| `extract_bookmarks` | Extract bookmark/TOC tree. |

### Page operations

| Tool | Description |
|------|-------------|
| `page_get_info` | Page dimensions and rotation. |
| `page_extract_text` | Text from a specific page. |
| `page_extract_tables` | Tables from a specific page. |
| `page_extract_structure` | Structure from a specific page. |
| `page_analyze_layout` | Detect columns, headers, footers. |
| `page_to_markdown` | Convert a page to Markdown. |

### Manipulation

| Tool | Description |
|------|-------------|
| `split_document` | Split by page ranges into multiple sessions. |
| `merge_documents` | Merge multiple PDFs into one session. |
| `reorder_pages` | Reorder, subset, or duplicate pages. |
| `delete_pages` | Remove specific pages. |
| `insert_blank_pages` | Add blank pages at positions. |
| `rotate_pages` | Rotate specific pages. |
| `optimize_document` | Compress and reduce file size. |

### Annotations & stamps

| Tool | Description |
|------|-------------|
| `add_watermark` | Apply a text watermark to pages. |
| `add_annotation` | Add annotation (text, highlight, stamp, etc.). |
| `remove_annotations` | Remove annotations by type or index. |
| `stamp_pages` | Overlay a page from another PDF. |

### Metadata & TOC

| Tool | Description |
|------|-------------|
| `set_metadata` | Update title, author, subject, keywords. |
| `set_bookmarks` | Set/replace bookmarks. |
| `generate_toc` | Auto-generate TOC from headings. |

### Comparison

| Tool | Description |
|------|-------------|
| `diff_documents` | Text-level diff between two PDFs. |
| `visual_diff` | Pixel-level visual comparison. |

### Conversion

| Tool | Description |
|------|-------------|
| `convert_document` | Convert between formats (PDF, DOCX, XLSX, PPTX, HTML, EPUB, Markdown). |
| `convert_file` | Direct file-to-file conversion. |
| `detect_format` | Detect document format from path. |

### Rendering

| Tool | Description |
|------|-------------|
| `render_page` | Render a page to PNG/JPEG/BMP. Max 600 DPI. |
| `render_pages` | Render multiple pages to images. Max 50 pages per call. |

### Forms

| Tool | Description |
|------|-------------|
| `has_form` | Check if document has a form. |
| `get_form_fields` | List all form fields. |
| `fill_form` | Fill form fields by name/value. |
| `modify_form_field` | Modify field properties. |
| `add_form_field` | Create a new form field. |

### Security

| Tool | Description |
|------|-------------|
| `sanitize_document` | Remove JavaScript, actions, embedded files, and links. |
| `redact_text` | Find and permanently redact text by query or regex. |
| `redact_regions` | Redact rectangular areas. |
| `encrypt_document` | Password-protect a document with AES-128, AES-256, or RC4. |

### Digital signatures

| Tool | Description |
|------|-------------|
| `get_signatures` | Extract signature info. |
| `verify_signatures` | Verify all signatures. |
| `sign_document` | Digitally sign a PDF. |

### Validation

| Tool | Description |
|------|-------------|
| `validate_pdf_a` | Check PDF/A compliance. |
| `validate_pdf_ua` | Check PDF/UA accessibility. |
| `convert_to_pdf_a` | Convert to PDF/A. |

## Example interaction

Here is what a typical AI-agent conversation looks like when the MCP server is connected:

**User:** "Summarize the Q3 financial report and redact all Social Security numbers."

The agent would call the following tools in sequence:

1. `open_document` with `path: "Q3_report.pdf"` -- receives session ID
2. `extract_text` with the session ID -- reads the full document text
3. `to_markdown` with the session ID -- gets a structured version for summarization
4. `redact_text` with `query: "\\b\\d{3}-\\d{2}-\\d{4}\\b"` and `use_regex: true`
5. `save_document` with `path: "Q3_report_redacted.pdf"`
6. `close_document` to release the session

The agent uses the extracted Markdown to write a summary, while the redacted PDF is saved to disk.

## Session management

The MCP server maintains document sessions. When a client calls `open_document`, the server loads the file into memory and returns a session ID. Subsequent tool calls reference this ID to operate on the same document without re-reading from disk.

Sessions are lightweight -- the document is loaded once and shared across all operations. The server enforces a maximum session count (`--max-sessions`) to bound memory usage. Sessions expire after the TTL (`--session-ttl`) of inactivity and are released when the client calls `close_document` or when the server shuts down.

Multiple documents can be open simultaneously in separate sessions:

1. `open_document("invoice.pdf")` -- session `s1`
2. `open_document("contract.docx")` -- session `s2`
3. `extract_tables` on `s1`
4. `to_markdown` on `s2`
5. `close_document` on `s1` and `s2`

## Security

The MCP server enforces a working directory sandbox:

- All file paths are resolved relative to `--working-dir`
- Paths that escape the working directory (e.g. `../../etc/passwd`) are rejected
- Absolute paths outside the working directory are rejected

Pipeline tools (`run_pipeline`, `validate_pipeline`) are disabled because they perform file I/O with their own path resolution that bypasses the sandbox. Use individual tools instead.

## Error handling

Tool calls return structured JSON errors when something goes wrong:

- **File not found** -- the path does not exist or is outside the working directory
- **Path escapes working directory** -- the resolved path is outside the sandbox
- **Invalid session** -- the session ID is expired or unrecognised
- **Unsupported operation** -- e.g. calling `redact_text` on an XLSX document
- **Conversion error** -- the requested format conversion is not supported

The AI model receives these errors as tool-call responses and can decide how to proceed (retry, ask the user, or try a different approach).
