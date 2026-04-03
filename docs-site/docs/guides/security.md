# Security: Sanitize, Redact & Encrypt

paperjam provides three distinct security operations: sanitization (removing active/dangerous content), redaction (permanently removing sensitive text from the content stream), and encryption (password-protecting documents).

## Sanitization

Sanitization removes potentially dangerous elements that could execute code, phone home, or embed malware:

```python
import paperjam

doc = paperjam.open("untrusted.pdf")
sanitized, result = doc.sanitize(
    remove_javascript=True,
    remove_embedded_files=True,
    remove_actions=True,
    remove_links=True,
)

print(f"JavaScript blocks removed: {result.javascript_removed}")
print(f"Embedded files removed:    {result.embedded_files_removed}")
print(f"Actions removed:           {result.actions_removed}")
print(f"Links removed:             {result.links_removed}")
print(f"Total removed:             {result.total_removed}")

# Inspect exactly what was removed
for item in result.items:
    page_info = f"(page {item.page})" if item.page else "(document-level)"
    print(f"  [{item.category}] {item.description} {page_info}")

sanitized.save("safe.pdf")
```

`SanitizeResult` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `javascript_removed` | `int` | Number of JavaScript actions removed |
| `embedded_files_removed` | `int` | Number of attached files removed |
| `actions_removed` | `int` | Number of other actions removed |
| `links_removed` | `int` | Number of links removed |
| `items` | `tuple[SanitizedItem, ...]` | Detailed list of removed items |
| `total_removed` | `int` | Sum of all four counters |

`SanitizedItem` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `category` | `str` | Category of the removed item |
| `description` | `str` | Human-readable description |
| `page` | `int \| None` | Page it was found on, if applicable |

## Redaction

paperjam's redaction is **true content-stream redaction** — it removes the underlying text operators from the PDF content stream rather than drawing a cosmetic black box on top. This means the redacted text cannot be recovered by editing the PDF or extracting text programmatically.

### Redact by search query

```python
# Redact all occurrences of a literal string
redacted, result = doc.redact_text(
    "John Smith",
    case_sensitive=False,
    use_regex=False,
    fill_color=(0.0, 0.0, 0.0),   # black fill box
)

# Redact using a regular expression
redacted, result = doc.redact_text(
    r"\b\d{3}-\d{2}-\d{4}\b",     # Social Security Numbers
    use_regex=True,
    fill_color=(0.0, 0.0, 0.0),
)

print(f"Pages modified: {result.pages_modified}")
print(f"Items redacted: {result.items_redacted}")
for item in result.items:
    print(f"  Page {item.page}: {item.text!r} at {item.rect}")

redacted.save("redacted.pdf")
```

### Redact by region

Redact specific rectangles (useful for images, signatures, or areas identified visually):

```python
from paperjam import RedactRegion

regions = [
    RedactRegion(page=1, rect=(100.0, 700.0, 250.0, 720.0)),
    RedactRegion(page=2, rect=(50.0,  600.0, 300.0, 620.0)),
]

redacted, result = doc.redact(
    regions=regions,
    fill_color=(0.0, 0.0, 0.0),   # black; omit for transparent
)
redacted.save("redacted.pdf")
```

`RedactResult` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `pages_modified` | `int` | Number of pages that had redactions applied |
| `items_redacted` | `int` | Total number of text items redacted |
| `items` | `tuple[RedactedItem, ...]` | Detail for each redacted item |

`RedactedItem` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `page` | `int` | 1-based page number |
| `text` | `str` | The text that was redacted |
| `rect` | `tuple` | Bounding box of the redacted area |

## Encryption

`encrypt()` returns the encrypted PDF as **bytes** (not a `Document`), along with metadata about the encryption applied:

```python
encrypted_bytes, result = doc.encrypt(
    user_password="open-me",
    owner_password="full-control",   # optional; defaults to user_password
    permissions=paperjam.Permissions(
        print=True,
        copy=False,
        modify=False,
        annotate=False,
        fill_forms=True,
        accessibility=True,
        assemble=False,
        print_high_quality=True,
    ),
    algorithm="aes128",   # "aes128" (default), "aes256", or "rc4"
)

print(f"Algorithm:  {result.algorithm}")
print(f"Key length: {result.key_length} bits")

with open("encrypted.pdf", "wb") as f:
    f.write(encrypted_bytes)
```

### Denying all permissions

```python
encrypted_bytes, _ = doc.encrypt(
    user_password="readonly",
    permissions=paperjam.Permissions.none(),   # all flags denied
)
```

`Permissions` attributes (all `bool`, default `True`):

| Attribute | Description |
|-----------|-------------|
| `print` | Allow low-quality printing |
| `modify` | Allow document modification |
| `copy` | Allow text/image copying |
| `annotate` | Allow adding annotations |
| `fill_forms` | Allow form filling |
| `accessibility` | Allow accessibility tools |
| `assemble` | Allow assembling pages |
| `print_high_quality` | Allow high-quality printing |

`EncryptResult` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `algorithm` | `str` | `"AES-128"`, `"AES-256"`, or `"RC4-128"` |
| `key_length` | `int` | Key length in bits (128 or 256) |

## PDF/A Validation

Validate whether a document conforms to the PDF/A archival standard:

```python
report = doc.validate_pdf_a(level="1b")   # "1b", "1a", or "2b"

print(f"PDF/A level: {report.level}")
print(f"Compliant:   {report.is_compliant}")
print(f"Pages checked: {report.pages_checked}")
print(f"Fonts checked: {report.fonts_checked}")

for issue in report.issues:
    print(f"  [{issue.severity}] rule={issue.rule}  {issue.message}")
    if issue.page:
        print(f"    on page {issue.page}")
```

`ValidationReport` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `level` | `str` | Validation level checked (`"1b"`, `"1a"`, `"2b"`) |
| `is_compliant` | `bool` | Whether the document passed |
| `issues` | `tuple[ValidationIssue, ...]` | List of problems found |
| `fonts_checked` | `int` | Number of fonts inspected |
| `pages_checked` | `int` | Number of pages inspected |

`ValidationIssue` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `severity` | `str` | `"error"`, `"warning"`, or `"info"` |
| `rule` | `str` | PDF/A rule identifier |
| `message` | `str` | Human-readable description |
| `page` | `int \| None` | Page number, if applicable |

## PDF/A Conversion

Convert a document to PDF/A conformance. This writes XMP metadata, embeds an sRGB ICC profile, removes JavaScript/actions, and strips transparency (PDF/A-1):

```python
new_doc, result = doc.convert_to_pdf_a(level="1b")

print(f"Success:  {result.success}")
print(f"Level:    {result.level}")
print(f"Actions taken: {len(result.actions_taken)}")
for action in result.actions_taken:
    print(f"  [{action.category}] {action.description}")

if result.remaining_issues:
    print("Remaining issues:")
    for issue in result.remaining_issues:
        print(f"  [{issue.severity}] {issue.message}")

new_doc.save("archival.pdf")
```

Font embedding is not performed automatically. Documents with unembedded fonts will fail unless `force=True` is passed:

```python
new_doc, result = doc.convert_to_pdf_a(level="1b", force=True)
```

`ConversionResult` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `level` | `str` | Target conformance level |
| `success` | `bool` | Whether all issues were resolved |
| `actions_taken` | `tuple[ConversionAction, ...]` | What was fixed |
| `remaining_issues` | `tuple[ValidationIssue, ...]` | Unresolved problems |

`ConversionAction` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `category` | `str` | Category: `"metadata"`, `"color"`, `"encryption"`, `"transparency"`, `"actions"` |
| `description` | `str` | Human-readable description |
| `page` | `int \| None` | Page number, if applicable |

## PDF/UA Validation

Validate accessibility compliance against PDF/UA (ISO 14289-1):

```python
report = doc.validate_pdf_ua()

print(f"PDF/UA level: {report.level}")
print(f"Compliant:    {report.is_compliant}")
print(f"Pages checked:     {report.pages_checked}")
print(f"Structure elements: {report.structure_elements_checked}")

for issue in report.issues:
    page_info = f" (page {issue.page})" if issue.page else ""
    print(f"  [{issue.severity}] {issue.rule}: {issue.message}{page_info}")
```

Checks performed:

- `/MarkInfo` dictionary with `/Marked true`
- Document language (`/Lang` in catalog)
- `/ViewerPreferences/DisplayDocTitle`
- `/StructTreeRoot` existence and structure
- Alt text on `/Figure` structure elements
- Heading hierarchy (H1-H6, no skipped levels)
- Tab order (`/Tabs /S` on pages)
- Annotation accessibility (`/Contents` or `/Alt`)
- Tagged content operators (BDC/BMC) in content streams

`PdfUaReport` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `level` | `str` | Validation level (`"1"`) |
| `is_compliant` | `bool` | Whether the document passed |
| `issues` | `tuple[ValidationIssue, ...]` | Problems found |
| `pages_checked` | `int` | Number of pages inspected |
| `structure_elements_checked` | `int` | Number of structure elements inspected |
