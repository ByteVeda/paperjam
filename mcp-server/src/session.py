"""Document session management for the MCP server."""

from __future__ import annotations

import time
import uuid
from dataclasses import dataclass, field
from pathlib import Path

import paperjam


class McpError(Exception):
    """Structured error for MCP tool responses."""

    def __init__(self, code: str, message: str) -> None:
        self.code = code
        self.message = message
        super().__init__(message)


@dataclass
class DocumentSession:
    """An open document with its metadata."""

    id: str
    document: paperjam.Document | paperjam.AnyDocument
    format: str
    path: str | None
    is_pdf: bool
    created_at: float = field(default_factory=time.monotonic)


class SessionManager:
    """Manages open document sessions."""

    def __init__(self, *, max_sessions: int = 50, ttl_seconds: float = 3600) -> None:
        self._sessions: dict[str, DocumentSession] = {}
        self._max_sessions = max_sessions
        self._ttl_seconds = ttl_seconds

    def configure(self, *, max_sessions: int | None = None, ttl_seconds: float | None = None) -> None:
        """Update session limits. Only provided values are changed."""
        if max_sessions is not None:
            self._max_sessions = max_sessions
        if ttl_seconds is not None:
            self._ttl_seconds = ttl_seconds

    def _cleanup_expired(self) -> None:
        now = time.monotonic()
        expired = [sid for sid, s in self._sessions.items() if (now - s.created_at) > self._ttl_seconds]
        for sid in expired:
            session = self._sessions.pop(sid)
            session.document.close()

    def open_from_path(self, path: str, *, password: str | None = None, fmt: str | None = None) -> str:
        """Open a document from a file path. Returns session ID."""
        self._cleanup_expired()
        if len(self._sessions) >= self._max_sessions:
            raise McpError("max_sessions", f"Maximum of {self._max_sessions} concurrent sessions reached")

        doc = paperjam.open(path, password=password, format=fmt)
        is_pdf = isinstance(doc, paperjam.Document)
        detected_format = fmt or (paperjam.detect_format(path) if not is_pdf else "pdf")

        session_id = uuid.uuid4().hex[:12]
        self._sessions[session_id] = DocumentSession(
            id=session_id,
            document=doc,
            format=detected_format if detected_format else "pdf",
            path=str(Path(path).resolve()),
            is_pdf=is_pdf,
        )
        return session_id

    def get(self, session_id: str) -> DocumentSession:
        """Get a session by ID. Raises McpError if not found."""
        session = self._sessions.get(session_id)
        if session is None:
            raise McpError("session_not_found", f"No session with ID '{session_id}'")
        return session

    def get_pdf(self, session_id: str) -> tuple[DocumentSession, paperjam.Document]:
        """Get a session that must be a PDF. Raises McpError if not found or not PDF."""
        session = self.get(session_id)
        if not session.is_pdf:
            raise McpError("pdf_required", f"This operation requires a PDF document, but session '{session_id}' is {session.format.upper()}")
        return session, session.document  # type: ignore[return-value]

    def update_document(self, session_id: str, new_doc: paperjam.Document | paperjam.AnyDocument) -> None:
        """Replace the document in a session (for mutation operations)."""
        session = self.get(session_id)
        session.document = new_doc

    def register(self, doc: paperjam.Document | paperjam.AnyDocument, *, fmt: str, path: str | None = None) -> str:
        """Register an already-open document as a new session. Returns session ID."""
        self._cleanup_expired()
        if len(self._sessions) >= self._max_sessions:
            raise McpError("max_sessions", f"Maximum of {self._max_sessions} concurrent sessions reached")

        is_pdf = isinstance(doc, paperjam.Document)
        session_id = uuid.uuid4().hex[:12]
        self._sessions[session_id] = DocumentSession(
            id=session_id,
            document=doc,
            format=fmt,
            path=path,
            is_pdf=is_pdf,
        )
        return session_id

    def close(self, session_id: str) -> bool:
        """Close a session and free resources. Returns True if found."""
        session = self._sessions.pop(session_id, None)
        if session is None:
            return False
        session.document.close()
        return True

    def list_sessions(self) -> list[dict]:
        """List all open sessions."""
        self._cleanup_expired()
        return [
            {
                "session_id": s.id,
                "format": s.format,
                "path": s.path,
                "is_pdf": s.is_pdf,
                "page_count": s.document.page_count,
            }
            for s in self._sessions.values()
        ]
