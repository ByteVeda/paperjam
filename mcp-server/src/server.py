"""FastMCP server definition and global state."""

from __future__ import annotations

import functools
import json
import logging
from pathlib import Path

import paperjam
from mcp.server.fastmcp import FastMCP
from paperjam_mcp.session import McpError, SessionManager

log = logging.getLogger("paperjam_mcp")

mcp = FastMCP(
    "paperjam",
    instructions=(
        "Document processing server powered by paperjam. Supports PDF, DOCX, XLSX, PPTX, HTML, EPUB."
        " Open documents into sessions, then extract, manipulate, convert, and analyze them."
    ),
)

session_manager = SessionManager()
working_dir: Path = Path.cwd()


def resolve_path(path: str) -> Path:
    """Resolve a path relative to the working directory. Rejects paths outside it."""
    p = Path(path)
    resolved = (p if p.is_absolute() else working_dir / p).resolve()
    if not resolved.is_relative_to(working_dir):
        raise ValueError(f"Path '{path}' escapes the working directory")
    return resolved


def handle_errors(func):
    """Decorator that converts paperjam/session exceptions to structured JSON error strings."""

    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        try:
            return func(*args, **kwargs)
        except McpError as e:
            return json.dumps({"error": e.code, "message": e.message})
        except paperjam.PasswordRequired:
            return json.dumps({"error": "password_required", "message": "Document is encrypted; provide a password"})
        except paperjam.InvalidPassword:
            return json.dumps({"error": "invalid_password", "message": "The provided password is incorrect"})
        except paperjam.PageOutOfRange as e:
            return json.dumps({"error": "page_out_of_range", "message": str(e)})
        except paperjam.PdfError as e:
            return json.dumps({"error": "pdf_error", "message": str(e)})
        except FileNotFoundError as e:
            return json.dumps({"error": "file_not_found", "message": str(e)})
        except ValueError as e:
            return json.dumps({"error": "invalid_argument", "message": str(e)})
        except Exception:
            log.exception("Unexpected error in tool %s", func.__name__)
            return json.dumps({"error": "internal_error", "message": "An unexpected error occurred. Check server logs for details."})

    return wrapper
