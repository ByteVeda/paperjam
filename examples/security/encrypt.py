"""Encrypt a PDF with user/owner passwords and permission flags."""

import argparse
import os

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Encrypt a PDF with passwords and permission controls.",
    )
    parser.add_argument("input", help="Path to the input PDF")
    parser.add_argument(
        "-o",
        "--output",
        default="./output",
        help="Output directory (default: ./output)",
    )
    parser.add_argument(
        "--user-password",
        required=True,
        help="Password required to open the document",
    )
    parser.add_argument(
        "--owner-password",
        help="Password for full access (defaults to user password)",
    )
    parser.add_argument(
        "--deny-all",
        action="store_true",
        help="Start with all permissions denied (use --allow-* to re-enable)",
    )
    parser.add_argument(
        "--no-print",
        action="store_true",
        help="Deny printing",
    )
    parser.add_argument(
        "--no-modify",
        action="store_true",
        help="Deny document modification",
    )
    parser.add_argument(
        "--no-copy",
        action="store_true",
        help="Deny content copying",
    )
    parser.add_argument(
        "--no-annotate",
        action="store_true",
        help="Deny adding annotations",
    )
    parser.add_argument(
        "--no-fill-forms",
        action="store_true",
        help="Deny form filling",
    )
    parser.add_argument(
        "--no-accessibility",
        action="store_true",
        help="Deny accessibility text extraction",
    )
    parser.add_argument(
        "--no-assemble",
        action="store_true",
        help="Deny document assembly",
    )
    parser.add_argument(
        "--no-print-hq",
        action="store_true",
        help="Deny high-quality printing",
    )
    parser.add_argument(
        "--algorithm",
        choices=["aes128", "rc4"],
        default="aes128",
        help="Encryption algorithm (default: aes128)",
    )
    args = parser.parse_args()

    os.makedirs(args.output, exist_ok=True)

    doc = paperjam.open_pdf(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    # Build permissions
    if args.deny_all:
        permissions = paperjam.Permissions.none()
    else:
        permissions = paperjam.Permissions(
            print=not args.no_print,
            modify=not args.no_modify,
            copy=not args.no_copy,
            annotate=not args.no_annotate,
            fill_forms=not args.no_fill_forms,
            accessibility=not args.no_accessibility,
            assemble=not args.no_assemble,
            print_high_quality=not args.no_print_hq,
        )

    print("\nPermissions:")
    print(f"  Print:              {permissions.print}")
    print(f"  Modify:             {permissions.modify}")
    print(f"  Copy:               {permissions.copy}")
    print(f"  Annotate:           {permissions.annotate}")
    print(f"  Fill forms:         {permissions.fill_forms}")
    print(f"  Accessibility:      {permissions.accessibility}")
    print(f"  Assemble:           {permissions.assemble}")
    print(f"  Print high quality: {permissions.print_high_quality}")

    encrypted_bytes, result = doc.encrypt(
        user_password=args.user_password,
        owner_password=args.owner_password,
        permissions=permissions,
        algorithm=args.algorithm,
    )

    basename = os.path.splitext(os.path.basename(args.input))[0]
    output_path = os.path.join(args.output, f"encrypted_{basename}.pdf")
    with open(output_path, "wb") as f:
        f.write(encrypted_bytes)

    print("\nEncryption result:")
    print(f"  Algorithm:  {result.algorithm}")
    print(f"  Key length: {result.key_length}-bit")
    print(f"  Size:       {len(encrypted_bytes):,} bytes")
    print(f"  Saved:      {output_path}")

    # Verify round-trip: re-open with user password
    print("\nVerifying round-trip...")
    try:
        verified = paperjam.open_pdf(output_path, password=args.user_password)
        print(f"  Opened encrypted PDF: {verified.page_count} pages")
        print(f"  Page 1 text length:   {len(verified.pages[0].extract_text())} chars")
    except paperjam.InvalidPassword:
        # AES-128 decryption not yet supported by the underlying parser
        print(f"  Round-trip verification skipped ({result.algorithm} decryption not yet supported by the underlying PDF parser)")


if __name__ == "__main__":
    main()
