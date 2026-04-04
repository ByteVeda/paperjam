"""Sanitize a PDF by removing JavaScript, embedded files, and dangerous actions."""

import argparse
from pathlib import Path

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Sanitize a PDF by removing potentially dangerous content.",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o",
        "--output",
        default="./output",
        help="Output directory (default: ./output)",
    )
    parser.add_argument(
        "--keep-js",
        action="store_true",
        help="Keep JavaScript (don't remove)",
    )
    parser.add_argument(
        "--keep-files",
        action="store_true",
        help="Keep embedded files",
    )
    parser.add_argument(
        "--keep-actions",
        action="store_true",
        help="Keep auto-launch actions",
    )
    parser.add_argument(
        "--keep-links",
        action="store_true",
        help="Keep link annotations",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc = paperjam.open_pdf(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    sanitized, result = doc.sanitize(
        remove_javascript=not args.keep_js,
        remove_embedded_files=not args.keep_files,
        remove_actions=not args.keep_actions,
        remove_links=not args.keep_links,
    )

    print("\nSanitization results:")
    print(f"  JavaScript removed:      {result.javascript_removed}")
    print(f"  Embedded files removed:  {result.embedded_files_removed}")
    print(f"  Actions removed:         {result.actions_removed}")
    print(f"  Links removed:           {result.links_removed}")
    print(f"  Total items removed:     {result.total_removed}")

    if result.items:
        print("\nDetails:")
        for item in result.items:
            loc = f" (page {item.page})" if item.page else ""
            print(f"  [{item.category}]{loc} {item.description}")

    out = output / f"sanitized_{Path(args.input).name}"
    sanitized.save(str(out))
    print(f"\nSaved sanitized PDF to {out}")


if __name__ == "__main__":
    main()
