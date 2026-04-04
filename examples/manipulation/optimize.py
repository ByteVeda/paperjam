"""Optimize a PDF to reduce file size."""

import argparse
import os

import paperjam


def main():
    parser = argparse.ArgumentParser(description="Optimize a PDF to reduce file size")
    parser.add_argument("input", help="Input PDF file")
    parser.add_argument("-o", "--output", default=".", help="Output directory")
    parser.add_argument("--strip-metadata", action="store_true", help="Remove document metadata")
    parser.add_argument("--no-compress", action="store_true", help="Skip stream compression")
    parser.add_argument("--no-dedup", action="store_true", help="Skip duplicate removal")
    args = parser.parse_args()

    os.makedirs(args.output, exist_ok=True)

    doc = paperjam.open_pdf(args.input)
    print(f"Loaded: {args.input} ({doc.page_count} pages)")

    optimized, result = doc.optimize(
        compress_streams=not args.no_compress,
        remove_unused=True,
        remove_duplicates=not args.no_dedup,
        strip_metadata=args.strip_metadata,
    )

    print("\nOptimization results:")
    print(f"  Original size:      {result.original_size:,} bytes")
    print(f"  Optimized size:     {result.optimized_size:,} bytes")
    print(f"  Reduction:          {result.reduction_percent:.1f}%")
    print(f"  Objects removed:    {result.objects_removed}")
    print(f"  Streams compressed: {result.streams_compressed}")

    basename = os.path.splitext(os.path.basename(args.input))[0]
    output_path = os.path.join(args.output, f"{basename}_optimized.pdf")
    optimized.save(output_path)
    print(f"\nSaved: {output_path}")


if __name__ == "__main__":
    main()
