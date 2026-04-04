"""Tool registration — importing each module registers its tools with the FastMCP server."""

from paperjam_mcp.tools import (
    annotation,
    comparison,
    conversion,
    document,
    extraction,
    forms,
    manipulation,
    metadata,
    page,
    pipeline,
    render,
    security,
    signatures,
    validation,
)

__all__ = [
    "annotation",
    "comparison",
    "conversion",
    "document",
    "extraction",
    "forms",
    "manipulation",
    "metadata",
    "page",
    "pipeline",
    "render",
    "security",
    "signatures",
    "validation",
]
