"""Extract tables from PDF pages and save them as CSV files."""

import argparse
from pathlib import Path

import paperjam


def parse_pages(spec: str, total: int) -> list[int]:
    """Parse '1-5' or '1,3,5' into 1-indexed page numbers."""
    pages: list[int] = []
    for part in spec.split(","):
        if "-" in part:
            start, end = part.split("-", 1)
            pages.extend(range(int(start), int(end) + 1))
        else:
            pages.append(int(part))
    return [p for p in pages if 1 <= p <= total]


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Extract tables from a PDF file.",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o",
        "--output",
        default="./output",
        help="Output directory (default: ./output)",
    )
    parser.add_argument(
        "-p",
        "--pages",
        help="Pages to scan, e.g. '1-5' or '1,3,5' (default: all)",
    )
    parser.add_argument(
        "--strategy",
        choices=["auto", "lattice", "stream"],
        default="auto",
        help="Table detection strategy (default: auto)",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc = paperjam.open(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    page_nums = parse_pages(args.pages, doc.page_count) if args.pages else list(range(1, doc.page_count + 1))

    total_tables = 0
    for num in page_nums:
        page = doc.pages[num - 1]
        tables = page.extract_tables(strategy=args.strategy)
        if not tables:
            continue

        print(f"  Page {num}: {len(tables)} table(s)")
        for i, table in enumerate(tables):
            out = output / f"page{num}_table{i}.csv"
            out.write_text(table.to_csv())
            print(f"    Table {i}: {table.row_count} rows x {table.col_count} cols ({table.strategy}) -> {out.name}")

            if table.row_count > 0:
                header = [c.text for c in table.rows[0].cells]
                print(f"      Header: {header}")

            total_tables += 1

    if total_tables == 0:
        print("  No tables found.")
    else:
        print(f"\nExtracted {total_tables} table(s) to {output}/")


if __name__ == "__main__":
    main()
