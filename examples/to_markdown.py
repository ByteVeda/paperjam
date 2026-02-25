"""Convert a PDF to Markdown."""

import argparse
from pathlib import Path

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Convert a PDF to clean Markdown (for LLM/RAG pipelines).",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o", "--output",
        help="Output markdown file (default: stdout)",
    )
    parser.add_argument(
        "--heading-offset", type=int, default=0,
        help="Add to heading levels, e.g. 1 makes # become ## (default: 0)",
    )
    parser.add_argument(
        "--page-numbers", action="store_true",
        help="Include page number comments",
    )
    parser.add_argument(
        "--html-tables", action="store_true",
        help="Use HTML tables instead of pipe tables",
    )
    parser.add_argument(
        "--layout-aware", action="store_true",
        help="Use layout-aware reading order (better for multi-column PDFs)",
    )
    args = parser.parse_args()

    doc = paperjam.open(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)", flush=True)

    md = doc.to_markdown(
        heading_offset=args.heading_offset,
        include_page_numbers=args.page_numbers,
        html_tables=args.html_tables,
        layout_aware=args.layout_aware,
    )

    if args.output:
        Path(args.output).write_text(md)
        print(f"Saved to {args.output} ({len(md)} chars)")
    else:
        print(md)


if __name__ == "__main__":
    main()
