"""Pipeline tools: run and validate document processing pipelines."""

from __future__ import annotations

import json

import paperjam
from paperjam_mcp.server import handle_errors, mcp


@mcp.tool()
@handle_errors
def run_pipeline(definition: str) -> str:
    """Execute a document processing pipeline from a YAML or JSON definition.

    Example YAML:
        pipeline:
          input: "docs/*.pdf"
          steps:
            - extract_tables: {}
            - convert: { format: xlsx }
            - save: { path: "output/{filename}.xlsx" }
    """
    result = paperjam.run_pipeline(definition)
    return json.dumps(result)


@mcp.tool()
@handle_errors
def validate_pipeline(definition: str) -> str:
    """Validate a pipeline definition without executing it.

    Returns success or raises an error describing what is invalid.
    """
    paperjam.validate_pipeline(definition)
    return json.dumps({"valid": True})
