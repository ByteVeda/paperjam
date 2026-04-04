"""Pipeline tools: disabled — pipelines bypass server-side path validation."""

from __future__ import annotations

import json

from paperjam_mcp.server import handle_errors, mcp


@mcp.tool()
@handle_errors
def run_pipeline(definition: str) -> str:
    """Execute a document processing pipeline.

    Pipelines are disabled in the MCP server because they perform file I/O
    with their own path resolution, bypassing the server's working directory
    sandbox. Use individual tools (open_document, extract_tables, convert_file,
    etc.) instead.
    """
    return json.dumps(
        {
            "error": "pipeline_disabled",
            "message": "Pipelines are disabled in the MCP server because they bypass path sandboxing. Use individual tools instead.",
        }
    )


@mcp.tool()
@handle_errors
def validate_pipeline(definition: str) -> str:
    """Validate a pipeline definition.

    Pipelines are disabled in the MCP server. See run_pipeline for details.
    """
    return json.dumps(
        {
            "error": "pipeline_disabled",
            "message": "Pipelines are disabled in the MCP server because they bypass path sandboxing. Use individual tools instead.",
        }
    )
