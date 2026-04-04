"""Analyze the layout of a PDF page (column detection, reading order)."""

import argparse

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Analyze PDF page layout (columns, headers, footers, reading order).",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-p",
        "--page",
        type=int,
        default=1,
        help="Page number to analyze (default: 1)",
    )
    parser.add_argument(
        "--gutter-width",
        type=float,
        default=20.0,
        help="Minimum gutter width for column detection (default: 20.0)",
    )
    args = parser.parse_args()

    doc = paperjam.open_pdf(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    page = doc.pages[args.page - 1]
    layout = page.analyze_layout(min_gutter_width=args.gutter_width)

    print(f"\nPage {args.page} layout:")
    print(f"  Dimensions: {layout.page_width:.1f} x {layout.page_height:.1f}")
    print(f"  Columns:    {layout.column_count}")
    if layout.gutters:
        print(f"  Gutters:    {', '.join(f'{g:.1f}' for g in layout.gutters)}")

    print(f"\nRegions ({len(layout.regions)}):")
    for region in layout.regions:
        col = f" col={region.column_index}" if region.column_index is not None else ""
        print(f"  {region.kind}{col}: {len(region.lines)} lines")

    print("\nText in reading order:")
    text = page.extract_text_layout(min_gutter_width=args.gutter_width)
    for line in text.split("\n")[:20]:
        print(f"  {line}")
    if text.count("\n") > 20:
        print(f"  ... ({text.count(chr(10)) - 20} more lines)")


if __name__ == "__main__":
    main()
