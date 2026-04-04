"""MCP prompt templates for common document processing workflows."""

from __future__ import annotations

from mcp.server.fastmcp.prompts import base
from paperjam_mcp.server import mcp


@mcp.prompt()
def extract_and_summarize(path: str) -> list[base.Message]:
    """Open a document and extract its contents for analysis."""
    return [
        base.UserMessage(
            content=f"""Open the document at '{path}' and:
1. Extract all text content
2. Extract any tables found
3. Extract the document structure (headings, sections)
4. Provide a concise summary of the document's type, purpose, and key content
5. Close the document when done"""
        ),
    ]


@mcp.prompt()
def compare_documents(path_a: str, path_b: str) -> list[base.Message]:
    """Compare two documents and report differences."""
    return [
        base.UserMessage(
            content=f"""Compare these two documents:
- Document A: '{path_a}'
- Document B: '{path_b}'

Steps:
1. Open both documents
2. Run a text-level diff to find content changes
3. Report: pages changed, additions, removals, and modifications
4. Provide a summary of the key differences
5. Close both documents when done"""
        ),
    ]


@mcp.prompt()
def redact_sensitive_data(path: str, patterns: str | None = None) -> list[base.Message]:
    """Guide through redacting sensitive information from a document."""
    default_patterns = r"SSNs (\b\d{3}-\d{2}-\d{4}\b), emails (\b[\w.]+@[\w.]+\.\w+\b), phone numbers (\b\d{3}[-.]?\d{3}[-.]?\d{4}\b)"
    pattern_desc = patterns or default_patterns
    return [
        base.UserMessage(
            content=f"""Redact sensitive data from '{path}':

Patterns to redact: {pattern_desc}

Steps:
1. Open the document
2. Search for each sensitive pattern to preview what will be redacted
3. Apply redaction for each pattern (use regex mode)
4. Save the redacted document to a new file (append '_redacted' to the filename)
5. Report what was redacted and how many items per pattern
6. Close the document"""
        ),
    ]


@mcp.prompt()
def form_filling_assistant(path: str) -> list[base.Message]:
    """Read form fields and guide through filling them."""
    return [
        base.UserMessage(
            content=f"""Help me fill out the form in '{path}':

1. Open the document
2. Check if it has an interactive form
3. List all form fields with their names, types, and current values
4. For each unfilled required field, ask me what value to enter
5. Fill the form with the provided values
6. Save the filled form
7. Close the document"""
        ),
    ]


@mcp.prompt()
def document_to_markdown(path: str) -> list[base.Message]:
    """Convert any document to clean Markdown."""
    return [
        base.UserMessage(
            content=f"""Convert '{path}' to Markdown:

1. Open the document (any format: PDF, DOCX, XLSX, PPTX, HTML, EPUB)
2. Convert to Markdown with tables and page numbers
3. Return the full Markdown content
4. Close the document"""
        ),
    ]
