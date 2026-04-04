# paperjam-mcp

MCP server for [paperjam](https://github.com/ByteVeda/paperjam) — document processing for AI agents.

Open, extract from, convert, and manipulate PDF, DOCX, XLSX, PPTX, HTML, and EPUB documents through the [Model Context Protocol](https://modelcontextprotocol.io/).

## Installation

```bash
# Run directly (no install needed)
uvx paperjam-mcp

# Or install globally
pip install paperjam-mcp
```

## Configuration

### Claude Code

Add to your project's `.mcp.json`:

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

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "paperjam": {
      "command": "uvx",
      "args": ["paperjam-mcp", "--working-dir", "/path/to/documents"]
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

## CLI Options

| Flag | Default | Description |
|------|---------|-------------|
| `--working-dir` | `.` | Base directory for resolving relative file paths |
| `--transport` | `stdio` | Transport: `stdio` or `sse` |
| `--port` | `8080` | Port for SSE transport |
| `--max-sessions` | `50` | Maximum concurrent document sessions |
| `--session-ttl` | `3600` | Session time-to-live in seconds |
| `--log-level` | `warning` | Log level: `debug`, `info`, `warning`, `error` |

## Available Tools

### Document Management
| Tool | Description |
|------|-------------|
| `open_document` | Open a document (PDF, DOCX, XLSX, PPTX, HTML, EPUB). Returns a session ID. |
| `close_document` | Close a session and free resources. |
| `list_sessions` | List all open document sessions. |
| `get_document_info` | Get metadata, page count, and format info. |
| `save_document` | Save a document to disk. |

### Extraction
| Tool | Description |
|------|-------------|
| `extract_text` | Extract all text from a document. |
| `extract_tables` | Extract tables as structured data. |
| `extract_structure` | Extract headings, paragraphs, lists, tables. |
| `to_markdown` | Convert a document to Markdown. |
| `search_document` | Full-text search with regex support (PDF). |
| `extract_links` | Extract all hyperlinks (PDF). |
| `extract_images` | Extract image metadata from a page (PDF). |
| `extract_bookmarks` | Extract bookmark/TOC tree. |

### Page Operations
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
| `split_document` | Split by page ranges. |
| `merge_documents` | Merge multiple PDFs. |
| `reorder_pages` | Reorder, subset, or duplicate pages. |
| `delete_pages` | Remove pages. |
| `insert_blank_pages` | Insert blank pages. |
| `rotate_pages` | Rotate pages. |
| `optimize_document` | Compress and reduce file size. |

### Annotations & Stamps
| Tool | Description |
|------|-------------|
| `add_watermark` | Add text watermark. |
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
| `convert_document` | Convert between formats. |
| `convert_file` | Direct file-to-file conversion. |
| `detect_format` | Detect document format. |

### Rendering
| Tool | Description |
|------|-------------|
| `render_page` | Render a page to PNG/JPEG/BMP. |
| `render_pages` | Render multiple pages to images. |

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
| `sanitize_document` | Remove JavaScript, actions, embedded files. |
| `redact_text` | Redact text by query/regex (true redaction). |
| `redact_regions` | Redact rectangular areas. |
| `encrypt_document` | Encrypt with AES-128/256 or RC4. |

### Digital Signatures
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

### Pipelines
| Tool | Description |
|------|-------------|
| `run_pipeline` | Execute a YAML/JSON processing pipeline. |
| `validate_pipeline` | Validate a pipeline definition. |

## License

MIT
