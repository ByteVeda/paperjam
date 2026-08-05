"""Signature methods for Document: signatures, verify_signatures, sign."""

from __future__ import annotations

from typing import TYPE_CHECKING

from paperjam import _paperjam
from paperjam._types import (
    CertificateInfo,
    SignatureInfo,
    SignatureValidity,
)

if TYPE_CHECKING:
    from paperjam._protocols import DocumentBase

    _Base = DocumentBase
else:
    _Base = object


class SignatureMixin(_Base):
    __slots__ = ()

    @property
    def signatures(self) -> list[SignatureInfo]:
        """Extract all digital signatures from the document."""
        inner = self._ensure_open()
        raw_bytes = self._raw_bytes
        if raw_bytes is None:
            return []
        raw = _paperjam.extract_signatures(inner, raw_bytes)
        result = []
        for sig in raw:
            cert = None
            if sig["certificate"] is not None:
                cert = CertificateInfo(
                    subject=sig["certificate"]["subject"],
                    issuer=sig["certificate"]["issuer"],
                    serial_number=sig["certificate"]["serial_number"],
                    not_before=sig["certificate"]["not_before"],
                    not_after=sig["certificate"]["not_after"],
                    is_self_signed=sig["certificate"]["is_self_signed"],
                )
            result.append(
                SignatureInfo(
                    name=sig["name"],
                    signer=sig["signer"],
                    reason=sig["reason"],
                    location=sig["location"],
                    date=sig["date"],
                    contact_info=sig["contact_info"],
                    byte_range=tuple(sig["byte_range"]) if sig["byte_range"] else None,
                    certificate=cert,
                    covers_whole_document=sig["covers_whole_document"],
                    has_timestamp=sig.get("has_timestamp", False),
                    timestamp_date=sig.get("timestamp_date"),
                    has_ocsp=sig.get("has_ocsp", False),
                    has_crls=sig.get("has_crls", False),
                )
            )
        return result

    def verify_signatures(self) -> list[SignatureValidity]:
        """Verify all digital signatures in the document.

        For each signature, checks:
        - Integrity: the hash of the signed bytes matches the PKCS#7 signature
        - Certificate validity: basic date check

        Returns a list of SignatureValidity results.
        """
        inner = self._ensure_open()
        raw_bytes = self._raw_bytes
        if raw_bytes is None:
            return []
        raw = _paperjam.verify_signatures(inner, raw_bytes)
        return [
            SignatureValidity(
                name=r["name"],
                integrity_ok=r["integrity_ok"],
                certificate_valid=r["certificate_valid"],
                message=r["message"],
                signer=r["signer"],
                timestamp_valid=r.get("timestamp_valid"),
                revocation_ok=r.get("revocation_ok"),
                is_ltv=r.get("is_ltv", False),
            )
            for r in raw
        ]

    def sign(
        self,
        *,
        private_key: bytes,
        certificates: list[bytes],
        reason: str | None = None,
        location: str | None = None,
        contact_info: str | None = None,
        field_name: str = "Signature1",
        tsa_url: str | None = None,
        timestamp_token: bytes | None = None,
        ocsp_responses: list[bytes] | None = None,
        crls: list[bytes] | None = None,
    ) -> bytes:
        """Sign the document with a digital signature.

        Args:
            private_key: DER-encoded private key (PKCS#8 format).
            certificates: List of DER-encoded X.509 certificates.
                The first certificate should be the signing certificate.
            reason: Reason for signing.
            location: Location of signing.
            contact_info: Contact information.
            field_name: Signature field name (default: "Signature1").
            tsa_url: TSA server URL for RFC 3161 timestamps (LTV).
            timestamp_token: Pre-fetched timestamp token bytes (LTV).
            ocsp_responses: OCSP responses to embed (LTV).
            crls: CRLs to embed (LTV).

        Returns:
            The finalized signed PDF as bytes.
        """
        inner = self._ensure_open()
        return bytes(
            _paperjam.sign_document(
                inner,
                private_key,
                certificates,
                reason=reason,
                location=location,
                contact_info=contact_info,
                field_name=field_name,
                tsa_url=tsa_url,
                timestamp_token=timestamp_token,
                ocsp_responses=ocsp_responses,
                crls=crls,
            )
        )
