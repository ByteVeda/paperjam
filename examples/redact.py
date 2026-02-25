"""Redact text from a PDF, removing it from the content stream (true redaction)."""

import argparse
from pathlib import Path

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Redact text from a PDF (true content removal, not cosmetic).",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument("query", help="Text to search for and redact")
    parser.add_argument(
        "-o", "--output", default="./output",
        help="Output directory (default: ./output)",
    )
    parser.add_argument(
        "--no-fill", action="store_true",
        help="Don't draw black rectangles over redacted areas",
    )
    parser.add_argument(
        "-i", "--case-insensitive", action="store_true",
        help="Case-insensitive search",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc = paperjam.open(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    fill_color = None if args.no_fill else (0.0, 0.0, 0.0)

    redacted, result = doc.redact_text(
        args.query,
        case_sensitive=not args.case_insensitive,
        fill_color=fill_color,
    )

    print("\nRedaction results:")
    print(f"  Query:          {args.query!r}")
    print(f"  Pages modified: {result.pages_modified}")
    print(f"  Items redacted: {result.items_redacted}")

    if result.items:
        print("\nRedacted items:")
        for item in result.items:
            print(f"  Page {item.page}: {item.text!r}")

    out = output / f"redacted_{Path(args.input).name}"
    redacted.save(str(out))
    print(f"\nSaved redacted PDF to {out}")


if __name__ == "__main__":
    main()
