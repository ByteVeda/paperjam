"""Create, fill, and modify form fields on a PDF — full showcase."""

import argparse
import os

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Create multiple form fields on a PDF, fill them, and modify properties.",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o",
        "--output",
        default="./output",
        help="Output directory (default: ./output)",
    )
    args = parser.parse_args()

    os.makedirs(args.output, exist_ok=True)

    doc = paperjam.open_pdf(args.input)
    basename = os.path.splitext(os.path.basename(args.input))[0]
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    # 1. Create a text field
    print("\n--- Creating fields on page 1 ---")
    doc, r = doc.add_form_field(
        "full_name",
        "text",
        page=1,
        rect=(72, 700, 300, 720),
        font_size=12.0,
    )
    print(f"  Text field 'full_name': created={r.created}")

    # 2. Create a checkbox
    doc, r = doc.add_form_field(
        "agree_terms",
        "checkbox",
        page=1,
        rect=(72, 660, 92, 680),
    )
    print(f"  Checkbox 'agree_terms': created={r.created}")

    # 3. Create a combo box with options
    doc, r = doc.add_form_field(
        "language",
        "combo_box",
        page=1,
        rect=(72, 620, 250, 640),
        options=[
            paperjam.ChoiceOption(display="Python", export_value="py"),
            paperjam.ChoiceOption(display="Rust", export_value="rs"),
            paperjam.ChoiceOption(display="JavaScript", export_value="js"),
        ],
    )
    print(f"  Combo box 'language': created={r.created}")

    # 4. Fill all fields with appearance generation
    print("\n--- Filling fields (generate_appearances=True) ---")
    doc, result = doc.fill_form(
        {
            "full_name": "Jane Smith",
            "agree_terms": "Yes",
            "language": "py",
        },
        generate_appearances=True,
    )
    print(f"  Fields filled:    {result.fields_filled}")
    print(f"  Fields not found: {result.fields_not_found}")

    # 5. Modify a field to be read-only
    print("\n--- Modifying 'full_name' to read-only + required ---")
    doc, mod_r = doc.modify_form_field(
        "full_name",
        read_only=True,
        required=True,
        max_length=100,
    )
    print(f"  Modified: {mod_r.modified}")

    # 6. List all fields
    print("\n--- Final form fields ---")
    for f in doc.form_fields:
        flags = []
        if f.read_only:
            flags.append("read-only")
        if f.required:
            flags.append("required")
        flag_str = f" [{', '.join(flags)}]" if flags else ""
        value = f.value or "(empty)"
        print(f"  {f.name}: {f.field_type} = {value}{flag_str}")

    # 7. Save
    output_path = os.path.join(args.output, f"{basename}_with_forms.pdf")
    doc.save(output_path)
    size = os.path.getsize(output_path)
    print(f"\nSaved: {output_path} ({size:,} bytes)")


if __name__ == "__main__":
    main()
