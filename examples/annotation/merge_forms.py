"""Demonstrate form-aware PDF merging with automatic field name prefixing."""

import argparse
import os

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Create form fields on a PDF, merge two copies, and show that fields from both are preserved with prefixed names.",
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

    basename = os.path.splitext(os.path.basename(args.input))[0]

    # --- Prepare doc A: text field + checkbox ---
    print("--- Preparing document A ---")
    doc_a = paperjam.open(args.input)
    doc_a, _ = doc_a.add_form_field(
        "username",
        "text",
        page=1,
        rect=(72, 700, 250, 720),
    )
    doc_a, _ = doc_a.add_form_field(
        "subscribe",
        "checkbox",
        page=1,
        rect=(72, 660, 92, 680),
    )
    doc_a, result = doc_a.fill_form(
        {"username": "Alice", "subscribe": "Yes"},
        generate_appearances=True,
    )
    print(f"  Fields: {[f.name for f in doc_a.form_fields]}")
    print(f"  Filled: {result.fields_filled}")

    # --- Prepare doc B: different fields ---
    print("\n--- Preparing document B ---")
    doc_b = paperjam.open(args.input)
    doc_b, _ = doc_b.add_form_field(
        "email",
        "text",
        page=1,
        rect=(72, 700, 300, 720),
    )
    doc_b, _ = doc_b.add_form_field(
        "plan",
        "combo_box",
        page=1,
        rect=(72, 660, 250, 680),
        options=[
            paperjam.ChoiceOption(display="Free", export_value="free"),
            paperjam.ChoiceOption(display="Pro", export_value="pro"),
        ],
    )
    doc_b, result = doc_b.fill_form(
        {"email": "bob@example.com", "plan": "pro"},
        generate_appearances=True,
    )
    print(f"  Fields: {[f.name for f in doc_b.form_fields]}")
    print(f"  Filled: {result.fields_filled}")

    # --- Merge ---
    print("\n--- Merging documents ---")
    merged = paperjam.merge([doc_a, doc_b])
    print(f"  Total pages: {merged.page_count}")
    print(f"  Has form: {merged.has_form}")

    # --- Show merged fields (should have prefixed names) ---
    fields = merged.form_fields
    print(f"  Total fields: {len(fields)}")
    for f in fields:
        value = f.value or "(empty)"
        print(f"    {f.name}: {f.field_type} = {value}")

    # --- Save ---
    output_path = os.path.join(args.output, f"{basename}_merged_forms.pdf")
    merged.save(output_path)
    size = os.path.getsize(output_path)
    print(f"\nSaved: {output_path} ({size:,} bytes)")


if __name__ == "__main__":
    main()
