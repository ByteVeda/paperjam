"""Merge multiple PDF files into one."""

import argparse
from pathlib import Path

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Merge multiple PDF files into one.",
    )
    parser.add_argument(
        "inputs", nargs="+",
        help="Two or more PDF files to merge",
    )
    parser.add_argument(
        "-o", "--output", default="./output",
        help="Output directory (default: ./output)",
    )
    parser.add_argument(
        "--name", default="merged.pdf",
        help="Output filename (default: merged.pdf)",
    )
    args = parser.parse_args()

    if len(args.inputs) < 2:
        parser.error("At least 2 input files are required.")

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    docs = []
    total_pages = 0
    for path in args.inputs:
        doc = paperjam.open(path)
        docs.append(doc)
        total_pages += doc.page_count
        print(f"  {path}: {doc.page_count} pages")

    merged = paperjam.merge(docs)
    out = output / args.name
    merged.save(str(out))

    size = out.stat().st_size
    print(f"\nMerged {len(docs)} files ({total_pages} pages) "
          f"-> {out} ({size:,} bytes)")


if __name__ == "__main__":
    main()
