"""Compare two PDFs at the text level and show differences."""

import argparse
from pathlib import Path

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Compare two PDF files and report text differences.",
    )
    parser.add_argument("file_a", help="Path to the first PDF")
    parser.add_argument("file_b", help="Path to the second PDF")
    parser.add_argument(
        "-o",
        "--output",
        default="./output",
        help="Output directory (default: ./output)",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc_a = paperjam.open(args.file_a)
    doc_b = paperjam.open(args.file_b)
    print("Comparing:")
    print(f"  A: {args.file_a} ({doc_a.page_count} pages)")
    print(f"  B: {args.file_b} ({doc_b.page_count} pages)")
    print()

    result = paperjam.diff(doc_a, doc_b)

    s = result.summary
    print("Summary:")
    print(f"  Pages changed:  {s.pages_changed}")
    print(f"  Pages added:    {s.pages_added}")
    print(f"  Pages removed:  {s.pages_removed}")
    print(f"  Additions:      {s.total_additions}")
    print(f"  Removals:       {s.total_removals}")
    print(f"  Changes:        {s.total_changes}")
    print()

    lines: list[str] = []
    for page_diff in result.page_diffs:
        if not page_diff.ops:
            continue
        header = f"--- Page {page_diff.page} ---"
        print(header)
        lines.append(header)

        for op in page_diff.ops:
            if op.kind == "added":
                line = f"+ {op.text_b}"
                print(f"  \033[32m{line}\033[0m")
            elif op.kind == "removed":
                line = f"- {op.text_a}"
                print(f"  \033[31m{line}\033[0m")
            elif op.kind == "changed":
                line = f"~ {op.text_a}  ->  {op.text_b}"
                print(f"  \033[33m{line}\033[0m")
            else:
                line = f"? {op.kind}: {op.text_a} / {op.text_b}"
            lines.append(line)

        lines.append("")

    if not lines:
        print("No differences found.")
    else:
        out = output / "diff.txt"
        out.write_text("\n".join(lines))
        print(f"\nSaved diff to {out}")


if __name__ == "__main__":
    main()
