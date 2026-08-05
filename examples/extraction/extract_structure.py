"""Extract structured content (headings, paragraphs, lists, tables) from a PDF."""

import argparse
from pathlib import Path

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Extract structured content from a PDF file.",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o",
        "--output",
        default="./output",
        help="Output directory (default: ./output)",
    )
    parser.add_argument(
        "--heading-ratio",
        type=float,
        default=1.2,
        help="Font size ratio threshold for headings (default: 1.2)",
    )
    parser.add_argument(
        "--no-lists",
        action="store_true",
        help="Disable list detection",
    )
    parser.add_argument(
        "--no-tables",
        action="store_true",
        help="Disable table interleaving",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc = paperjam.open_pdf(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    blocks = doc.extract_structure(
        heading_size_ratio=args.heading_ratio,
        detect_lists=not args.no_lists,
        include_tables=not args.no_tables,
    )

    counts: dict[str, int] = {}
    for block in blocks:
        counts[block.type] = counts.get(block.type, 0) + 1

    print(f"Extracted {len(blocks)} content blocks:")
    for btype, count in sorted(counts.items()):
        print(f"  {btype}: {count}")
    print()

    lines: list[str] = []
    for block in blocks:
        if block.type == "heading":
            prefix = "#" * (block.level or 1)
            lines.append(f"{prefix} {block.text}")
            lines.append("")
        elif block.type == "paragraph":
            lines.append(block.text or "")
            lines.append("")
        elif block.type == "list_item":
            indent = "  " * (block.indent_level or 0)
            lines.append(f"{indent}- {block.text}")
        elif block.type == "table" and block.table:
            for row in block.table.rows:
                cells = " | ".join(c.text for c in row.cells)
                lines.append(f"| {cells} |")
            lines.append("")

    out = output / "structure.md"
    out.write_text("\n".join(lines))
    print(f"Saved markdown to {out}")

    # Also print first 20 blocks
    for block in blocks[:20]:
        if block.type == "heading":
            print(f"  [H{block.level}] {(block.text or '')[:80]}")
        elif block.type == "paragraph":
            text = (block.text or "")[:80]
            print(f"  [P]  {text}")
        elif block.type == "list_item":
            text = (block.text or "")[:80]
            print(f"  [LI] {'  ' * (block.indent_level or 0)}{text}")
        elif block.type == "table":
            rows = block.table.row_count if block.table else 0
            cols = block.table.col_count if block.table else 0
            print(f"  [T]  {rows}x{cols} table")

    if len(blocks) > 20:
        print(f"  ... and {len(blocks) - 20} more blocks")


if __name__ == "__main__":
    main()
