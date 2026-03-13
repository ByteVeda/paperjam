"""Inspect, verify, and apply digital signatures on a PDF."""

import argparse
import os

import paperjam


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Inspect, verify, and apply digital signatures on a PDF.",
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
        help="List all digital signatures in the document",
    )
    parser.add_argument(
        "--verify",
        action="store_true",
        help="Verify all signatures (integrity + certificate dates)",
    )
    parser.add_argument(
        "--sign",
        action="store_true",
        help="Sign the document (requires --key and --cert)",
    )
    parser.add_argument(
        "--key",
        help="Path to DER-encoded private key (PKCS#8 format)",
    )
    parser.add_argument(
        "--cert",
        help="Path to DER-encoded X.509 certificate",
    )
    parser.add_argument(
        "--reason",
        help="Reason for signing",
    )
    parser.add_argument(
        "--location",
        help="Location of signing",
    )
    parser.add_argument(
        "--field-name",
        default="Signature1",
        help="Signature field name (default: Signature1)",
    )
    args = parser.parse_args()

    os.makedirs(args.output, exist_ok=True)

    doc = paperjam.open(args.input)
    print(f"Opened: {args.input} ({doc.page_count} pages)")

    if args.list or (not args.verify and not args.sign):
        sigs = doc.signatures
        print(f"\nDigital signatures: {len(sigs)}")
        for sig in sigs:
            print(f"\n  Field: {sig.name}")
            if sig.signer:
                print(f"  Signer: {sig.signer}")
            if sig.reason:
                print(f"  Reason: {sig.reason}")
            if sig.location:
                print(f"  Location: {sig.location}")
            if sig.date:
                print(f"  Date: {sig.date}")
            if sig.contact_info:
                print(f"  Contact: {sig.contact_info}")
            print(f"  Covers whole document: {sig.covers_whole_document}")
            if sig.byte_range:
                print(f"  Byte range: {sig.byte_range}")
            if sig.certificate:
                cert = sig.certificate
                print("  Certificate:")
                print(f"    Subject: {cert.subject}")
                print(f"    Issuer: {cert.issuer}")
                print(f"    Serial: {cert.serial_number}")
                print(f"    Valid from: {cert.not_before}")
                print(f"    Valid until: {cert.not_after}")
                print(f"    Self-signed: {cert.is_self_signed}")

    if args.verify:
        print("\nVerifying signatures...")
        results = doc.verify_signatures()
        if not results:
            print("  No signatures to verify.")
        for r in results:
            status = "VALID" if r.integrity_ok and r.certificate_valid else "INVALID"
            print(f"\n  [{status}] {r.name}")
            print(f"    Integrity: {'OK' if r.integrity_ok else 'FAILED'}")
            print(f"    Certificate: {'valid' if r.certificate_valid else 'invalid/expired'}")
            if r.signer:
                print(f"    Signer: {r.signer}")
            print(f"    Message: {r.message}")

    if args.sign:
        if not args.key or not args.cert:
            parser.error("--sign requires --key and --cert")

        with open(args.key, "rb") as f:
            private_key = f.read()
        with open(args.cert, "rb") as f:
            cert_der = f.read()

        print("\nSigning document...")
        signed_bytes = doc.sign(
            private_key=private_key,
            certificates=[cert_der],
            reason=args.reason,
            location=args.location,
            field_name=args.field_name,
        )

        basename = os.path.splitext(os.path.basename(args.input))[0]
        output_path = os.path.join(args.output, f"{basename}_signed.pdf")
        with open(output_path, "wb") as f:
            f.write(signed_bytes)
        print(f"  Signed PDF: {len(signed_bytes):,} bytes")
        print(f"  Saved: {output_path}")

        # Verify the signature we just created
        signed_doc = paperjam.Document(signed_bytes)
        results = signed_doc.verify_signatures()
        for r in results:
            status = "VALID" if r.integrity_ok and r.certificate_valid else "INVALID"
            print(f"  Verification: [{status}] {r.message}")


if __name__ == "__main__":
    main()
