# Exceptions

All exceptions are importable directly from `paperjam`:

```python
from paperjam import PdfError, ParseError, PasswordRequired, InvalidPassword, PageOutOfRange
```

---

## Exception hierarchy

```
PdfError (base)
├── ParseError
├── PasswordRequired
├── InvalidPassword
├── PageOutOfRange          (also subclasses IndexError)
├── UnsupportedFeature
├── TableExtractionError
├── AnnotationError
├── WatermarkError
├── OptimizationError
├── SanitizeError
├── EncryptionError
├── RedactError
├── FormError
├── RenderError
├── SignatureError
├── FormatError
└── PipelineError
```

---

## Base exception

### `PdfError`

```python
class paperjam.PdfError(Exception)
```

Base class for all paperjam exceptions. Catch this to handle any PDF-related error regardless of its specific cause:

```python
import paperjam
from paperjam import PdfError

try:
    doc = paperjam.open("file.pdf")
except PdfError as e:
    print(f"PDF error: {e}")
```

---

## Opening and parsing

### `ParseError`

```python
class paperjam.ParseError(PdfError)
```

Raised when a file cannot be parsed as a PDF. This typically means the file is corrupt, truncated, or not a PDF at all.

```python
try:
    doc = paperjam.open("not-a-pdf.txt")
except paperjam.ParseError as e:
    print(f"Not a valid PDF: {e}")
```

### `PasswordRequired`

```python
class paperjam.PasswordRequired(PdfError)
```

Raised when trying to open an encrypted PDF without providing a password.

```python
try:
    doc = paperjam.open("locked.pdf")
except paperjam.PasswordRequired:
    print("This PDF requires a password")
    doc = paperjam.open("locked.pdf", password=input("Password: "))
```

### `InvalidPassword`

```python
class paperjam.InvalidPassword(PdfError)
```

Raised when the provided password does not decrypt the PDF.

```python
try:
    doc = paperjam.open("locked.pdf", password="wrong")
except paperjam.InvalidPassword:
    print("Incorrect password")
```

### `PageOutOfRange`

```python
class paperjam.PageOutOfRange(PdfError, IndexError)
```

Raised when a 1-based page number does not exist in the document. Also subclasses `IndexError` so it is caught by bare `IndexError` handlers.

```python
try:
    doc.render_page(999)
except paperjam.PageOutOfRange as e:
    print(f"No such page: {e}")
```

### `UnsupportedFeature`

```python
class paperjam.UnsupportedFeature(PdfError)
```

Raised when the PDF uses a feature that paperjam does not support (e.g. a very old or experimental PDF extension).

---

## Extraction errors

### `TableExtractionError`

```python
class paperjam.TableExtractionError(PdfError)
```

Raised when table extraction fails for a specific page, for example due to malformed content streams.

---

## Annotation and watermark errors

### `AnnotationError`

```python
class paperjam.AnnotationError(PdfError)
```

Raised when adding or removing annotations fails.

### `WatermarkError`

```python
class paperjam.WatermarkError(PdfError)
```

Raised when adding a watermark fails (e.g. invalid font or color parameters).

---

## Security errors

### `OptimizationError`

```python
class paperjam.OptimizationError(PdfError)
```

Raised when PDF optimization fails.

### `SanitizeError`

```python
class paperjam.SanitizeError(PdfError)
```

Raised when sanitization fails.

### `EncryptionError`

```python
class paperjam.EncryptionError(PdfError)
```

Raised when encryption fails (e.g. unsupported algorithm or key length).

### `RedactError`

```python
class paperjam.RedactError(PdfError)
```

Raised when redaction fails, for example if the specified regions are outside the page bounds.

---

## Form errors

### `FormError`

```python
class paperjam.FormError(PdfError)
```

Raised for all form-related failures: reading, filling, creating, or modifying fields.

---

## Rendering errors

### `RenderError`

```python
class paperjam.RenderError(PdfError)
```

Raised when page rendering fails. Requires the `render` feature — if pdfium is not available this exception will be raised.

---

## Signature errors

### `SignatureError`

```python
class paperjam.SignatureError(PdfError)
```

Raised when signing fails (e.g. malformed private key or certificate) or when signature verification encounters an internal error.

---

## Format errors

### `FormatError`

```python
class paperjam.FormatError(PdfError)
```

Raised when a document format is unsupported or cannot be processed.

```python
try:
    doc = paperjam.open("archive.7z")
except paperjam.FormatError as e:
    print(f"Unsupported format: {e}")
```

---

## Pipeline errors

### `PipelineError`

```python
class paperjam.PipelineError(PdfError)
```

Raised when a pipeline definition is invalid or execution fails.

```python
try:
    paperjam.validate_pipeline(yaml_string)
except paperjam.PipelineError as e:
    print(f"Invalid pipeline: {e}")
```

---

## Catching all PDF errors

```python
import paperjam
from paperjam import PdfError

try:
    doc = paperjam.open(user_input)
    text = doc.pages[0].extract_text()
    doc.save("output.pdf")
except paperjam.PasswordRequired:
    return "This PDF is password-protected"
except paperjam.ParseError:
    return "This file is not a valid PDF"
except PdfError as e:
    return f"PDF processing error: {e}"
```
