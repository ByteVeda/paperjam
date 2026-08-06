"""Format conversion functions."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    import os

from paperjam import _paperjam


def convert(
    input_path: str | os.PathLike[str],
    output_path: str | os.PathLike[str],
) -> dict:
    """Convert a file from one format to another.

    Formats are auto-detected from file extensions.

    Returns:
        A dict with keys: from_format, to_format, content_blocks, tables, images.
    """
    return dict(_paperjam.convert_file(str(input_path), str(output_path)))


def convert_bytes(
    data: bytes,
    *,
    from_format: str,
    to_format: str,
) -> bytes:
    """Convert in-memory bytes from one format to another.

    Args:
        data: Source document bytes.
        from_format: Source format (e.g., 'pdf', 'docx', 'html').
        to_format: Target format (e.g., 'pdf', 'markdown', 'html').

    Returns:
        Converted document bytes.
    """
    return bytes(_paperjam.convert_bytes(data, from_format, to_format))


def detect_format(path: str | os.PathLike[str]) -> str:
    """Detect the document format from a file path.

    Returns:
        Format string (e.g., 'pdf', 'docx', 'html', 'epub', 'unknown').
    """
    return str(_paperjam.detect_format(str(path)))
