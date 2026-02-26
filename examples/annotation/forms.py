"""Inspect and fill form fields in a PDF."""

import argparse
import os

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Inspect and fill form fields in a PDF.",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o", "--output", default="./output",
        help="Output directory (default: ./output)",
    )
    parser.add_argument(
        "--list", action="store_true",
        help="List all form fields and their current values",
    )
    parser.add_argument(
        "--fill", nargs=2, action="append", metavar=("FIELD", "VALUE"),
        help="Set a field value, e.g. --fill name 'John Doe' (repeatable)",
    )
    args = parser.parse_args()

    os.makedirs(args.output, exist_ok=True)

    doc = paperjam.open(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")
    print(f"Has form: {doc.has_form}")

    if not doc.has_form:
        print("No AcroForm found in this PDF.")
        return

    fields = doc.form_fields
    print(f"Form fields: {len(fields)}")

    if args.list or not args.fill:
        print()
        for f in fields:
            value = f.value or "(empty)"
            readonly = " [read-only]" if f.read_only else ""
            required = " [required]" if f.required else ""
            page = f" (page {f.page})" if f.page else ""
            print(f"  {f.name}: {f.field_type}{page}{readonly}{required}")
            print(f"    Value: {value}")
            if f.default_value:
                print(f"    Default: {f.default_value}")
            if f.options:
                choices = ", ".join(o.display for o in f.options[:5])
                more = f" ... +{len(f.options) - 5}" if len(f.options) > 5 else ""
                print(f"    Options: {choices}{more}")
            if f.max_length:
                print(f"    Max length: {f.max_length}")

    if args.fill:
        values = {name: val for name, val in args.fill}
        print(f"\nFilling {len(values)} field(s)...")

        filled_doc, result = doc.fill_form(values)

        print(f"  Fields filled:     {result.fields_filled}")
        print(f"  Fields not found:  {result.fields_not_found}")
        if result.not_found_names:
            print(f"  Not found names:   {', '.join(result.not_found_names)}")

        basename = os.path.splitext(os.path.basename(args.input))[0]
        output_path = os.path.join(args.output, f"{basename}_filled.pdf")
        filled_doc.save(output_path)
        print(f"\nSaved: {output_path}")


if __name__ == "__main__":
    main()
