"""Inspect, fill, create, and modify form fields in a PDF."""

import argparse
import os

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Inspect, fill, create, and modify form fields in a PDF.",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o",
        "--output",
        default="./output",
        help="Output directory (default: ./output)",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="List all form fields and their current values",
    )
    parser.add_argument(
        "--fill",
        nargs=2,
        action="append",
        metavar=("FIELD", "VALUE"),
        help="Set a field value, e.g. --fill name 'John Doe' (repeatable)",
    )
    parser.add_argument(
        "--generate-appearances",
        action="store_true",
        help="Generate explicit appearance streams when filling (for viewers that ignore /NeedAppearances)",
    )
    parser.add_argument(
        "--create",
        nargs=2,
        metavar=("FIELD", "TYPE"),
        help="Create a new form field. TYPE is one of: text, checkbox, combo_box, list_box, push_button, signature",
    )
    parser.add_argument(
        "--page",
        type=int,
        default=1,
        help="Page number for --create (default: 1)",
    )
    parser.add_argument(
        "--rect",
        nargs=4,
        type=float,
        metavar=("X1", "Y1", "X2", "Y2"),
        default=[100.0, 700.0, 300.0, 720.0],
        help="Rectangle for --create (default: 100 700 300 720)",
    )
    parser.add_argument(
        "--modify",
        metavar="FIELD",
        help="Modify an existing form field's properties",
    )
    parser.add_argument(
        "--read-only",
        action="store_true",
        help="Set the field read-only (used with --modify)",
    )
    parser.add_argument(
        "--required",
        action="store_true",
        help="Set the field required (used with --modify)",
    )
    parser.add_argument(
        "--max-length",
        type=int,
        help="Set max length for a text field (used with --modify)",
    )
    args = parser.parse_args()

    os.makedirs(args.output, exist_ok=True)

    doc = paperjam.open_pdf(args.input)
    basename = os.path.splitext(os.path.basename(args.input))[0]
    print(f"Opened: {args.input} ({doc.page_count} pages)")
    print(f"Has form: {doc.has_form}")

    # --- Create a field ---
    if args.create:
        field_name, field_type = args.create
        rect = tuple(args.rect)
        print(f"\nCreating field '{field_name}' (type={field_type}) on page {args.page}, rect={rect}")
        doc, create_result = doc.add_form_field(
            field_name,
            field_type,
            page=args.page,
            rect=rect,
        )
        print(f"  Created: {create_result.created}")

    # --- Fill fields ---
    if args.fill:
        values = {name: val for name, val in args.fill}
        mode = "with appearances" if args.generate_appearances else "default"
        print(f"\nFilling {len(values)} field(s) ({mode})...")
        doc, fill_result = doc.fill_form(
            values,
            generate_appearances=args.generate_appearances,
        )
        print(f"  Fields filled:     {fill_result.fields_filled}")
        print(f"  Fields not found:  {fill_result.fields_not_found}")
        if fill_result.not_found_names:
            print(f"  Not found names:   {', '.join(fill_result.not_found_names)}")

    # --- Modify a field ---
    if args.modify:
        print(f"\nModifying field '{args.modify}': read_only={args.read_only}, required={args.required}, max_length={args.max_length}")
        doc, mod_result = doc.modify_form_field(
            args.modify,
            read_only=args.read_only or None,
            required=args.required or None,
            max_length=args.max_length,
        )
        print(f"  Modified: {mod_result.modified}")

    # --- List fields ---
    if args.list or (not args.fill and not args.create and not args.modify):
        fields = doc.form_fields if doc.has_form else []
        print(f"\nForm fields: {len(fields)}")
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

    # --- Save if anything was modified ---
    if args.fill or args.create or args.modify:
        output_path = os.path.join(args.output, f"{basename}_forms.pdf")
        doc.save(output_path)
        print(f"\nSaved: {output_path}")


if __name__ == "__main__":
    main()
