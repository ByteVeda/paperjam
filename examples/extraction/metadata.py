"""Display PDF document metadata and page information."""

import argparse
from pathlib import Path

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(description="Display PDF metadata and page info.")
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o",
        "--output",
        default=None,
        help="Output directory to save metadata.txt (optional)",
    )
    args = parser.parse_args()

    doc = paperjam.open_pdf(args.input)
    meta = doc.metadata

    print(f"File: {args.input}\n")

    fields = [
        ("Title", meta.title),
        ("Author", meta.author),
        ("Subject", meta.subject),
        ("Keywords", meta.keywords),
        ("Creator", meta.creator),
        ("Producer", meta.producer),
        ("Creation date", meta.creation_date),
        ("Modification date", meta.modification_date),
        ("PDF version", meta.pdf_version),
        ("Page count", meta.page_count),
        ("Encrypted", meta.is_encrypted),
    ]

    print("Document Metadata:")
    for label, value in fields:
        print(f"  {label:20s}: {value}")

    print("\nPage Info:")
    targets = {"First": 0, "Middle": doc.page_count // 2, "Last": doc.page_count - 1}
    for label, idx in targets.items():
        page = doc.pages[idx]
        print(f"  {label} (page {page.number}): {page.width:.1f} x {page.height:.1f} pts, rotation={page.rotation}")

    if args.output:
        output = Path(args.output)
        output.mkdir(parents=True, exist_ok=True)
        out = output / "metadata.txt"
        with open(out, "w") as f:
            for label, value in fields:
                f.write(f"{label}: {value}\n")
        print(f"\nSaved to {out}")


if __name__ == "__main__":
    main()
