"""Entry point: python -m paperjam_mcp"""

from __future__ import annotations

import argparse
import logging
from pathlib import Path

from paperjam_mcp import __version__


def main() -> None:
    parser = argparse.ArgumentParser(description="paperjam MCP server")
    parser.add_argument("--version", action="version", version=f"paperjam-mcp {__version__}")
    parser.add_argument("--working-dir", default=".", help="Working directory for resolving relative file paths")
    parser.add_argument("--transport", choices=["stdio", "sse"], default="stdio", help="MCP transport (default: stdio)")
    parser.add_argument("--port", type=int, default=8080, help="Port for SSE transport (default: 8080)")
    parser.add_argument("--max-sessions", type=int, default=50, help="Maximum concurrent document sessions (default: 50)")
    parser.add_argument("--session-ttl", type=int, default=3600, help="Session TTL in seconds (default: 3600)")
    parser.add_argument("--log-level", choices=["debug", "info", "warning", "error"], default="warning", help="Log level (default: warning)")
    args = parser.parse_args()

    logging.basicConfig(
        level=getattr(logging, args.log_level.upper()),
        format="%(asctime)s [%(levelname)s] %(name)s: %(message)s",
    )

    import importlib

    from paperjam_mcp import server as srv

    # Register tools, resources, and prompts with the FastMCP instance.
    for module in ("paperjam_mcp.prompts", "paperjam_mcp.resources", "paperjam_mcp.tools"):
        importlib.import_module(module)

    srv.working_dir = Path(args.working_dir).resolve()
    srv.session_manager.configure(max_sessions=args.max_sessions, ttl_seconds=args.session_ttl)

    if args.transport == "stdio":
        srv.mcp.run(transport="stdio")
    else:
        srv.mcp.settings.port = args.port
        srv.mcp.run(transport="sse")


if __name__ == "__main__":
    main()
