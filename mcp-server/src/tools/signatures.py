"""Signature tools: get_signatures, verify_signatures, sign_document."""

from __future__ import annotations

import json

from paperjam_mcp.serializers import serialize
from paperjam_mcp.server import handle_errors, mcp, resolve_path, session_manager


@mcp.tool()
@handle_errors
def get_signatures(session_id: str) -> str:
    """Extract all digital signature information from a PDF.

    Returns signer names, dates, certificate details, and coverage info.
    """
    _session, doc = session_manager.get_pdf(session_id)
    sigs = doc.signatures
    return json.dumps({"signatures": serialize(sigs), "count": len(sigs)})


@mcp.tool()
@handle_errors
def verify_signatures(session_id: str) -> str:
    """Verify all digital signatures in a PDF.

    Checks integrity (hash verification) and basic certificate validity.
    """
    _session, doc = session_manager.get_pdf(session_id)
    results = doc.verify_signatures()
    return json.dumps({"results": serialize(results), "count": len(results)})


@mcp.tool()
@handle_errors
def sign_document(
    session_id: str,
    private_key_path: str,
    certificate_paths: list[str],
    reason: str | None = None,
    location: str | None = None,
    contact_info: str | None = None,
    field_name: str = "Signature1",
    tsa_url: str | None = None,
) -> str:
    """Digitally sign a PDF document. Creates a new session with the signed document.

    private_key_path: path to DER-encoded private key (PKCS#8).
    certificate_paths: paths to DER-encoded X.509 certificates (signing cert first).
    tsa_url: optional TSA server URL for RFC 3161 timestamps.
    """
    _session, doc = session_manager.get_pdf(session_id)

    key_path = resolve_path(private_key_path)
    with open(key_path, "rb") as f:
        private_key = f.read()

    certificates = []
    for cp in certificate_paths:
        cert_path = resolve_path(cp)
        with open(cert_path, "rb") as f:
            certificates.append(f.read())

    signed_bytes = doc.sign(
        private_key=private_key,
        certificates=certificates,
        reason=reason,
        location=location,
        contact_info=contact_info,
        field_name=field_name,
        tsa_url=tsa_url,
    )

    import paperjam

    new_doc = paperjam.Document(signed_bytes)
    new_id = session_manager.register(new_doc, fmt="pdf")
    return json.dumps({"session_id": new_id, "signed": True, "field_name": field_name})
