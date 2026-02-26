"""Extract images from PDF pages and save them to disk."""

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
        description="Extract images from a PDF file.",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o", "--output", default="./output",
        help="Output directory (default: ./output)",
    )
    parser.add_argument(
        "-p", "--pages",
        help="Pages to scan, e.g. '1-5' or '1,3,5' (default: all)",
    )
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    doc = paperjam.open(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    page_nums = parse_pages(args.pages, doc.page_count) if args.pages else list(range(1, doc.page_count + 1))

    total_images = 0
    for num in page_nums:
        page = doc.pages[num - 1]
        images = page.extract_images()
        if not images:
            continue

        print(f"  Page {num}: {len(images)} image(s)")
        for i, img in enumerate(images):
            if "DCTDecode" in img.filters:
                ext = "jpg"
            elif "FlateDecode" in img.filters:
                ext = "png"
            else:
                ext = "raw"
            out = output / f"page{num}_img{i}.{ext}"
            img.save(str(out))
            cs = img.color_space or "unknown"
            print(f"    {img.width}x{img.height} {cs} "
                  f"({len(img.data):,} bytes) -> {out.name}")
            total_images += 1

    if total_images == 0:
        print("  No images found.")
    else:
        print(f"\nExtracted {total_images} image(s) to {output}/")


if __name__ == "__main__":
    main()
