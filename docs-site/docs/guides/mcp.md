---
sidebar_position: 17
title: MCP Server
---

# MCP Server

paperjam ships with a built-in MCP (Model Context Protocol) server, making it the first document processing library with native AI-agent support. Any MCP-compatible client -- Claude Code, Claude Desktop, or custom agents -- can open, extract from, convert, and manipulate documents through a standard tool interface.

## What is MCP?

The Model Context Protocol is an open standard for connecting AI models to external tools and data sources. Instead of writing custom glue code for each AI integration, you register an MCP server and the model discovers its capabilities automatically.

## Installation

```bash
# From crates.io
cargo install paperjam-mcp

# From source (in the paperjam repository)
cargo install --path crates/paperjam-mcp
```

Verify the installation:

```bash
paperjam-mcp --version
```

## Configuration

### Claude Code

Add the server to your project's `.mcp.json` or global MCP settings:

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

### Claude Desktop

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "paperjam": {
      "command": "paperjam-mcp",
      "args": ["--working-dir", "/Users/you/Documents"]
    }
  }
}
```

### Server options

| Flag | Description |
|------|-------------|
| `--working-dir <path>` | Base directory for resolving relative file paths |
| `--max-sessions <n>` | Maximum concurrent document sessions (default: 16) |
| `--log-level <level>` | Logging verbosity: `error`, `warn`, `info`, `debug` |

## Available tools

Once connected, the AI model can call these tools through the MCP protocol.

### Document tools

| Tool | Description |
|------|-------------|
| `open_document` | Open a document by path or URL. Returns a session ID. |
| `get_document_info` | Get page count, metadata, format, and structural summary. |
| `save_document` | Save the current document state to disk. |
| `close_document` | Close a session and free resources. |

### Extraction tools

| Tool | Description |
|------|-------------|
| `extract_text` | Extract plain text from all or specified pages. |
| `extract_tables` | Extract tables as structured data (rows, headers, cells). |
| `extract_structure` | Extract headings, paragraphs, and list items. |
| `to_markdown` | Convert the document to Markdown. |

### Conversion tools

| Tool | Description |
|------|-------------|
| `convert_document` | Convert between formats (PDF, DOCX, XLSX, PPTX, HTML, EPUB, Markdown). |

### Manipulation tools

| Tool | Description |
|------|-------------|
| `redact_text` | Find and permanently redact text by query or regex. |
| `add_watermark` | Apply a text watermark to pages. |
| `encrypt_document` | Password-protect a document with AES-128, AES-256, or RC4. |
| `sanitize_document` | Remove JavaScript, actions, embedded files, and links. |

### Pipeline tools

| Tool | Description |
|------|-------------|
| `run_pipeline` | Execute a multi-step processing pipeline from a YAML definition. |

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

Sessions are lightweight -- the document is loaded once and shared across all operations. The server enforces a maximum session count (`--max-sessions`) to bound memory usage. Sessions are released when the client calls `close_document` or when the MCP connection closes.

Multiple documents can be open simultaneously in separate sessions:

1. `open_document("invoice.pdf")` -- session `s1`
2. `open_document("contract.docx")` -- session `s2`
3. `extract_tables` on `s1`
4. `to_markdown` on `s2`
5. `close_document` on `s1` and `s2`

## Error handling

Tool calls return structured errors when something goes wrong:

- **File not found** -- the path does not exist or is outside the working directory
- **Invalid session** -- the session ID is expired or unrecognised
- **Unsupported operation** -- e.g. calling `redact_text` on an XLSX document
- **Conversion error** -- the requested format conversion is not supported

The AI model receives these errors as tool-call responses and can decide how to proceed (retry, ask the user, or try a different approach).
