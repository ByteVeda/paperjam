"""Search for text across all pages of a PDF."""

import argparse
from pathlib import Path

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Search for text in a PDF file.",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument("query", help="Text to search for")
    parser.add_argument(
        "-o",
        "--output",
        default="./output",
        help="Output directory (default: ./output)",
    )
    parser.add_argument(
        "--ignore-case",
        action="store_true",
        help="Case-insensitive search",
    )
    parser.add_argument(
        "--max-results",
        type=int,
        default=0,
        help="Max results to return (default: unlimited)",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc = paperjam.open_pdf(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    mode = " (case-insensitive)" if args.ignore_case else ""
    print(f'Searching for: "{args.query}"{mode}')

    results = doc.search(
        args.query,
        case_sensitive=not args.ignore_case,
        max_results=args.max_results,
    )

    if not results:
        print("  No matches found.")
        return

    print(f"  Found {len(results)} match(es):\n")
    for r in results:
        text = r.text.strip()[:80]
        print(f"  Page {r.page}, line {r.line_number}: {text}")

    out = output / "search_results.txt"
    with open(out, "w") as f:
        for r in results:
            f.write(f"Page {r.page}, line {r.line_number}: {r.text}\n")
    print(f"\nSaved to {out}")


if __name__ == "__main__":
    main()
