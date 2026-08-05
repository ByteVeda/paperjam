"""Add a text watermark to a PDF."""

import argparse
import os

import paperjam


def main():
    parser = argparse.ArgumentParser(description="Add a text watermark to a PDF")
    parser.add_argument("input", help="Input PDF file")
    parser.add_argument("-t", "--text", default="CONFIDENTIAL", help="Watermark text")
    parser.add_argument("-o", "--output", default=".", help="Output directory")
    parser.add_argument("--font-size", type=float, default=60.0, help="Font size (default: 60)")
    parser.add_argument("--rotation", type=float, default=45.0, help="Rotation angle in degrees (default: 45)")
    parser.add_argument("--opacity", type=float, default=0.3, help="Opacity 0.0-1.0 (default: 0.3)")
    parser.add_argument("--color", default="0.5,0.5,0.5", help="RGB color as r,g,b (default: 0.5,0.5,0.5)")
    parser.add_argument("--font", default="Helvetica", help="Font name (default: Helvetica)")
    parser.add_argument("--position", default="center", help="Position: center, top_left, top_right, bottom_left, bottom_right")
    parser.add_argument("--layer", default="over", help="Layer: over or under (default: over)")
    parser.add_argument("-p", "--pages", help="Comma-separated page numbers (default: all)")
    args = parser.parse_args()

    os.makedirs(args.output, exist_ok=True)

    doc = paperjam.open_pdf(args.input)
    print(f"Loaded: {args.input} ({doc.page_count} pages)")

    color = tuple(float(c) for c in args.color.split(","))
    pages = [int(p) for p in args.pages.split(",")] if args.pages else None

    watermarked = doc.add_watermark(
        args.text,
        font_size=args.font_size,
        rotation=args.rotation,
        opacity=args.opacity,
        color=color,
        font=args.font,
        position=args.position,
        layer=args.layer,
        pages=pages,
    )

    basename = os.path.splitext(os.path.basename(args.input))[0]
    output_path = os.path.join(args.output, f"{basename}_watermarked.pdf")
    watermarked.save(output_path)
    print(f"Saved: {output_path}")
    print(f"Watermark: '{args.text}' at {args.opacity} opacity, {args.rotation}° rotation")


if __name__ == "__main__":
    main()
