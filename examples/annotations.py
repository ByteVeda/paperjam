"""Read and add annotations to a PDF."""

import argparse
import os

import paperjam


def main():
    parser = argparse.ArgumentParser(description="Read and add annotations to a PDF")
    parser.add_argument("input", help="Input PDF file")
    parser.add_argument("-o", "--output", default=".", help="Output directory")
    parser.add_argument("--list", action="store_true", help="List existing annotations")
    parser.add_argument("--add-note", help="Add a sticky note with this text to page 1")
    parser.add_argument("--add-highlight", help="Add a highlight annotation (rect as x1,y1,x2,y2)")
    parser.add_argument("--remove", action="store_true", help="Remove annotations from a page")
    parser.add_argument("--remove-type", help="Only remove annotations of this type (e.g. Highlight, Text)")
    parser.add_argument("--remove-indices", help="Only remove annotations at these 0-based indices (comma-separated)")
    parser.add_argument("-p", "--page", type=int, default=1, help="Page number (default: 1)")
    args = parser.parse_args()

    os.makedirs(args.output, exist_ok=True)

    doc = paperjam.open(args.input)
    print(f"Loaded: {args.input} ({doc.page_count} pages)")

    if args.list:
        for i in range(1, doc.page_count + 1):
            page = doc.pages[i - 1]
            annots = page.annotations
            if annots:
                print(f"\nPage {i}: {len(annots)} annotation(s)")
                for j, a in enumerate(annots, 1):
                    print(f"  [{j}] Type: {a.type}")
                    print(f"       Rect: {a.rect}")
                    if a.contents:
                        print(f"       Contents: {a.contents}")
                    if a.author:
                        print(f"       Author: {a.author}")
                    if a.color:
                        print(f"       Color: {a.color}")
                    if a.opacity is not None:
                        print(f"       Opacity: {a.opacity}")

    modified = False
    result = doc

    if args.remove:
        annotation_types = None
        if args.remove_type:
            annotation_types = [args.remove_type]
        indices = None
        if args.remove_indices:
            indices = [int(i) for i in args.remove_indices.split(",")]
        result, count = result.remove_annotations(
            args.page, annotation_types=annotation_types, indices=indices,
        )
        print(f"\nRemoved {count} annotation(s) from page {args.page}")
        modified = count > 0

    if args.add_note:
        result = result.add_annotation(
            args.page,
            paperjam.AnnotationType.TEXT,
            (50.0, 700.0, 80.0, 730.0),
            contents=args.add_note,
            author="paperjam",
            color=(1.0, 1.0, 0.0),
        )
        print(f"\nAdded sticky note on page {args.page}: '{args.add_note}'")

    if args.add_highlight:
        coords = tuple(float(c) for c in args.add_highlight.split(","))
        result = result.add_annotation(
            args.page,
            paperjam.AnnotationType.HIGHLIGHT,
            coords,
            color=(1.0, 1.0, 0.0),
            opacity=0.5,
        )
        print(f"\nAdded highlight on page {args.page}: {coords}")

    if args.add_note or args.add_highlight:
        modified = True

    if modified:
        basename = os.path.splitext(os.path.basename(args.input))[0]
        output_path = os.path.join(args.output, f"{basename}_annotated.pdf")
        result.save(output_path)
        print(f"Saved: {output_path}")


if __name__ == "__main__":
    main()
