"""Render PDF pages to images."""

import argparse
import os

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


def render_fast(args: argparse.Namespace) -> None:
    """Render using the standalone paperjam.render() fast path (bytes -> image)."""
    os.makedirs(args.output, exist_ok=True)
    basename = os.path.splitext(os.path.basename(args.input))[0]

    with open(args.input, "rb") as f:
        data = f.read()

    # Determine which pages to render
    doc = paperjam.open(args.input)
    total = doc.page_count
    print(f"Opened: {args.input} ({total} pages)")

    if args.pages:
        page_nums = parse_pages(args.pages, total)
    else:
        page_nums = list(range(1, total + 1))

    print(f"Rendering {len(page_nums)} page(s) at {args.dpi} DPI ({args.format}) [fast mode]...")

    for num in page_nums:
        img = paperjam.render(data, page=num, dpi=args.dpi, format=args.format, quality=args.quality)
        filename = f"{basename}_page_{num}.{img.format}"
        output_path = os.path.join(args.output, filename)
        img.save(output_path)
        print(f"  Page {num}: {img.width}x{img.height} -> {filename}")

    print(f"\nSaved {len(page_nums)} image(s) to {args.output}/")


def render_batch(args: argparse.Namespace) -> None:
    """Render using doc.render_pages() batch path (parallel via rayon)."""
    os.makedirs(args.output, exist_ok=True)
    basename = os.path.splitext(os.path.basename(args.input))[0]

    doc = paperjam.open(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    if args.pages:
        page_nums = parse_pages(args.pages, doc.page_count)
    else:
        page_nums = None  # render_pages treats None as all pages

    page_desc = f"{len(page_nums)} page(s)" if page_nums else f"all {doc.page_count} page(s)"
    print(f"Rendering {page_desc} at {args.dpi} DPI ({args.format})...")

    images = doc.render_pages(
        pages=page_nums,
        dpi=args.dpi,
        format=args.format,
        quality=args.quality,
    )
    for img in images:
        filename = f"{basename}_page_{img.page}.{img.format}"
        output_path = os.path.join(args.output, filename)
        img.save(output_path)
        print(f"  Page {img.page}: {img.width}x{img.height} -> {filename}")

    print(f"\nSaved {len(images)} image(s) to {args.output}/")


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Render PDF pages to images (PNG, JPEG, or BMP).",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o", "--output", default="./output",
        help="Output directory (default: ./output)",
    )
    parser.add_argument(
        "-p", "--pages",
        help="Pages to render, e.g. '1-5' or '1,3,5' (default: all)",
    )
    parser.add_argument(
        "--dpi", type=float, default=150.0,
        help="Resolution in DPI (default: 150)",
    )
    parser.add_argument(
        "--format", choices=["png", "jpeg", "bmp"], default="png",
        help="Image format (default: png)",
    )
    parser.add_argument(
        "--quality", type=int, default=85,
        help="JPEG quality 1-100 (default: 85, ignored for PNG/BMP)",
    )
    parser.add_argument(
        "--fast", action="store_true",
        help="Use standalone render() fast path (bytes -> image, no Document needed)",
    )
    args = parser.parse_args()

    if args.fast:
        render_fast(args)
    else:
        render_batch(args)


if __name__ == "__main__":
    main()
