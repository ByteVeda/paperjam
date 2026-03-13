# Digital Signatures

paperjam can inspect, verify, and apply digital signatures. Signature support requires the `signatures` feature, which is included in all pre-built PyPI wheels.

## Inspecting signatures

`doc.signatures` returns a list of `SignatureInfo` objects:

```python
import paperjam

doc = paperjam.open("signed-contract.pdf")

for sig in doc.signatures:
    print(f"Signature: {sig.name}")
    print(f"  Signer:   {sig.signer}")
    print(f"  Reason:   {sig.reason}")
    print(f"  Location: {sig.location}")
    print(f"  Date:     {sig.date}")
    print(f"  Covers whole document: {sig.covers_whole_document}")
    if sig.certificate:
        cert = sig.certificate
        print(f"  Certificate subject:   {cert.subject}")
        print(f"  Certificate issuer:    {cert.issuer}")
        print(f"  Serial number:         {cert.serial_number}")
        print(f"  Valid from:            {cert.not_before}")
        print(f"  Valid until:           {cert.not_after}")
        print(f"  Self-signed:           {cert.is_self_signed}")
```

`SignatureInfo` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `name` | `str` | Signature field name in the PDF |
| `signer` | `str \| None` | Signer's name from the signature dictionary |
| `reason` | `str \| None` | Stated reason for signing |
| `location` | `str \| None` | Location where signing took place |
| `date` | `str \| None` | Signing date/time |
| `contact_info` | `str \| None` | Contact information |
| `byte_range` | `tuple \| None` | `(offset_a, len_a, offset_b, len_b)` of signed bytes |
| `certificate` | `CertificateInfo \| None` | Embedded certificate details |
| `covers_whole_document` | `bool` | Whether the signature covers the entire document |

`CertificateInfo` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `subject` | `str` | Certificate subject DN |
| `issuer` | `str` | Certificate issuer DN |
| `serial_number` | `str` | Certificate serial number (hex) |
| `not_before` | `str` | Validity start (ISO 8601) |
| `not_after` | `str` | Validity end (ISO 8601) |
| `is_self_signed` | `bool` | Whether subject equals issuer |

## Verifying signatures

`verify_signatures()` checks each signature's integrity and certificate validity:

```python
validity_list = doc.verify_signatures()

for v in validity_list:
    status = "OK" if (v.integrity_ok and v.certificate_valid) else "FAILED"
    print(f"[{status}] {v.name} — {v.message}")
    if v.signer:
        print(f"  Signer: {v.signer}")
```

`SignatureValidity` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `name` | `str` | Signature field name |
| `integrity_ok` | `bool` | Whether the signed bytes hash matches the PKCS#7 signature |
| `certificate_valid` | `bool` | Whether the certificate date range is valid |
| `message` | `str` | Human-readable status message |
| `signer` | `str \| None` | Signer name, if available |

### What is checked

- **Integrity**: the SHA-256 hash of the byte ranges specified in the signature is compared against the hash stored inside the PKCS#7 envelope. If the PDF was modified after signing, this check fails.
- **Certificate validity**: the current date is checked against the certificate's `not_before`/`not_after` range. Full certificate chain validation against a trust store is not performed.

## Signing a document

`sign()` appends a digital signature to the document and returns the signed PDF as **bytes**:

```python
# Load your DER-encoded private key and certificate chain
with open("private_key.der", "rb") as f:
    private_key = f.read()

with open("certificate.der", "rb") as f:
    signing_cert = f.read()

signed_bytes = doc.sign(
    private_key=private_key,
    certificates=[signing_cert],    # first cert = signing cert
    reason="Approved by legal",
    location="London, UK",
    contact_info="legal@example.com",
    field_name="Signature1",        # signature field to fill
)

with open("signed.pdf", "wb") as f:
    f.write(signed_bytes)
```

### Generating a test key pair

For testing purposes you can generate a self-signed certificate using OpenSSL:

```bash
# Generate private key
openssl genpkey -algorithm RSA -out key.pem -pkeyopt rsa_keygen_bits:2048

# Generate self-signed certificate
openssl req -new -x509 -key key.pem -out cert.pem -days 365 \
    -subj "/CN=Test Signer/O=Test Org"

# Convert to DER format for paperjam
openssl pkey  -in key.pem  -outform DER -out key.der
openssl x509  -in cert.pem -outform DER -out cert.der
```

```python
with open("key.der",  "rb") as f:
    private_key = f.read()
with open("cert.der", "rb") as f:
    cert = f.read()

signed_bytes = doc.sign(private_key=private_key, certificates=[cert])
```

### Sign parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `private_key` | `bytes` | DER-encoded PKCS#8 private key |
| `certificates` | `list[bytes]` | DER-encoded X.509 certificates; first = signing cert |
| `reason` | `str \| None` | Reason for signing |
| `location` | `str \| None` | Geographic location |
| `contact_info` | `str \| None` | Contact information |
| `field_name` | `str` | Signature field name (default: `"Signature1"`) |
