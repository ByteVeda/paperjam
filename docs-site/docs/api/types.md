# Data Types

All result types are frozen dataclasses (immutable). They are importable from `paperjam`:

```python
from paperjam import TextLine, TextSpan, Table, Metadata, ...
```

---

## Text types

### `TextSpan`

A positioned piece of text on a page.

| Attribute | Type | Description |
|-----------|------|-------------|
| `text` | `str` | The text content |
| `x` | `float` | Left edge in PDF points |
| `y` | `float` | Baseline y in PDF points |
| `width` | `float` | Width of the span in points |
| `font_size` | `float` | Font size in points |
| `font_name` | `str` | Font name as embedded in the PDF |

### `TextLine`

A line of text composed of one or more spans.

| Attribute | Type | Description |
|-----------|------|-------------|
| `text` | `str` | Full text of the line |
| `spans` | `tuple[TextSpan, ...]` | Individual spans making up the line |
| `bbox` | `tuple[float, float, float, float]` | Bounding box `(x1, y1, x2, y2)` |

---

## Table types

### `Table`

An extracted table from a PDF page.

| Attribute | Type | Description |
|-----------|------|-------------|
| `rows` | `tuple[Row, ...]` | All rows |
| `col_count` | `int` | Number of columns |
| `bbox` | `tuple[float, float, float, float]` | Bounding box |
| `strategy` | `str` | Extraction strategy used |
| `row_count` | `int` *(property)* | Number of rows |

**Methods**

| Method | Returns | Description |
|--------|---------|-------------|
| `cell(row, col)` | `Cell \| None` | Get cell by 0-based row and column |
| `to_list()` | `list[list[str]]` | Convert to 2D list of strings |
| `to_csv(delimiter=",")` | `str` | Convert to CSV string |
| `to_dataframe()` | `DataFrame` | Convert to pandas DataFrame (pandas extra required) |

### `Row`

A row in a table.

| Attribute | Type | Description |
|-----------|------|-------------|
| `cells` | `tuple[Cell, ...]` | Cells in the row |
| `y_min` | `float` | Top y coordinate |
| `y_max` | `float` | Bottom y coordinate |

### `Cell`

A single cell in a table.

| Attribute | Type | Description |
|-----------|------|-------------|
| `text` | `str` | Cell text content |
| `bbox` | `tuple[float, float, float, float]` | Bounding box |
| `col_span` | `int` | Column span (merged cells) |
| `row_span` | `int` | Row span (merged cells) |

---

## Document types

### `Metadata`

PDF document metadata.

| Attribute | Type | Description |
|-----------|------|-------------|
| `title` | `str \| None` | Document title |
| `author` | `str \| None` | Author |
| `subject` | `str \| None` | Subject |
| `keywords` | `str \| None` | Keywords |
| `creator` | `str \| None` | Creating application |
| `producer` | `str \| None` | PDF-writing library |
| `creation_date` | `str \| None` | ISO 8601 date |
| `modification_date` | `str \| None` | ISO 8601 date |
| `pdf_version` | `str` | PDF spec version, e.g. `"1.7"` |
| `page_count` | `int` | Total pages |
| `is_encrypted` | `bool` | Whether password-protected |
| `xmp_metadata` | `str \| None` | Raw XMP XML string |

### `PageInfo`

Basic page dimensions and orientation.

| Attribute | Type | Description |
|-----------|------|-------------|
| `number` | `int` | 1-based page number |
| `width` | `float` | Width in points |
| `height` | `float` | Height in points |
| `rotation` | `int` | Rotation: `0`, `90`, `180`, or `270` |

### `Bookmark`

A bookmark/outline entry.

| Attribute | Type | Description |
|-----------|------|-------------|
| `title` | `str` | Display text |
| `page` | `int` | 1-based destination page |
| `level` | `int` | Nesting level (0 = top) |
| `children` | `tuple[Bookmark, ...]` | Nested child bookmarks |

---

## Content types

### `ContentBlock`

A structured content block extracted from a page.

| Attribute | Type | Description |
|-----------|------|-------------|
| `type` | `str` | `"heading"`, `"paragraph"`, `"list_item"`, `"table"` |
| `page` | `int` | 1-based page number |
| `text` | `str \| None` | Text content (None for tables) |
| `level` | `int \| None` | Heading level 1–6 |
| `indent_level` | `int \| None` | List item nesting depth |
| `bbox` | `tuple \| None` | Bounding box |
| `table` | `Table \| None` | Table object (only for `type="table"`) |

### `Image`

An image extracted from a page.

| Attribute | Type | Description |
|-----------|------|-------------|
| `width` | `int` | Width in pixels |
| `height` | `int` | Height in pixels |
| `color_space` | `str \| None` | e.g. `"DeviceRGB"` |
| `bits_per_component` | `int \| None` | Bit depth |
| `filters` | `list[str]` | Applied PDF stream filters |
| `data` | `bytes` | Raw image data |

**Methods**: `save(path)` — write raw bytes to a file.

### `SearchResult`

A text search match.

| Attribute | Type | Description |
|-----------|------|-------------|
| `page` | `int` | 1-based page number |
| `text` | `str` | Full line text containing the match |
| `line_number` | `int` | 1-based line index on the page |
| `bbox` | `tuple \| None` | Bounding box of the matching line |

### `Link`

A hyperlink extracted from a page.

| Attribute | Type | Description |
|-----------|------|-------------|
| `page` | `int` | 1-based page number |
| `rect` | `tuple` | Clickable area `(x1, y1, x2, y2)` |
| `url` | `str \| None` | External URL |
| `destination` | `dict \| None` | Internal page destination |
| `contents` | `str \| None` | Alternative text |

### `Annotation`

A PDF annotation.

| Attribute | Type | Description |
|-----------|------|-------------|
| `type` | `str` | Annotation type string |
| `rect` | `tuple` | Bounding box |
| `contents` | `str \| None` | Text / tooltip |
| `author` | `str \| None` | Author name |
| `color` | `tuple \| None` | RGB float (0.0–1.0 each) |
| `creation_date` | `str \| None` | ISO 8601 date |
| `opacity` | `float \| None` | 0.0–1.0 |
| `url` | `str \| None` | URL (link annotations) |
| `destination` | `dict \| None` | Internal destination (link annotations) |

---

## Layout types

### `PageLayout`

Layout analysis result for a single page.

| Attribute | Type | Description |
|-----------|------|-------------|
| `page_width` | `float` | Page width in points |
| `page_height` | `float` | Page height in points |
| `column_count` | `int` | Detected column count |
| `gutters` | `tuple[float, ...]` | X positions of gutter centres |
| `regions` | `tuple[LayoutRegion, ...]` | Classified regions |
| `is_multi_column` | `bool` *(property)* | `column_count > 1` |

**Methods**: `text()` — all text in reading order.

### `LayoutRegion`

A classified rectangular region of the page.

| Attribute | Type | Description |
|-----------|------|-------------|
| `kind` | `str` | `"header"`, `"footer"`, `"body_column"`, `"full_width"` |
| `column_index` | `int \| None` | 0-based column index |
| `bbox` | `tuple` | Bounding box |
| `lines` | `tuple[TextLine, ...]` | Lines in reading order |

---

## Security and optimization types

### `OptimizeResult`

| Attribute | Type | Description |
|-----------|------|-------------|
| `original_size` | `int` | Size before optimization (bytes) |
| `optimized_size` | `int` | Size after optimization (bytes) |
| `objects_removed` | `int` | Number of objects removed |
| `streams_compressed` | `int` | Number of streams compressed |
| `reduction_percent` | `float` *(property)* | Percentage size reduction |

### `SanitizeResult`

| Attribute | Type | Description |
|-----------|------|-------------|
| `javascript_removed` | `int` | JS actions removed |
| `embedded_files_removed` | `int` | Attachments removed |
| `actions_removed` | `int` | Other actions removed |
| `links_removed` | `int` | Links removed |
| `items` | `tuple[SanitizedItem, ...]` | Detailed list |
| `total_removed` | `int` *(property)* | Sum of all four counters |

### `SanitizedItem`

| Attribute | Type | Description |
|-----------|------|-------------|
| `category` | `str` | Category of removed item |
| `description` | `str` | Human-readable description |
| `page` | `int \| None` | Page number if applicable |

### `RedactResult`

| Attribute | Type | Description |
|-----------|------|-------------|
| `pages_modified` | `int` | Number of pages with redactions |
| `items_redacted` | `int` | Total items redacted |
| `items` | `tuple[RedactedItem, ...]` | Details of each redaction |

### `RedactedItem`

| Attribute | Type | Description |
|-----------|------|-------------|
| `page` | `int` | 1-based page number |
| `text` | `str` | The redacted text |
| `rect` | `tuple` | Bounding box of the redacted area |

### `RedactRegion`

A region to redact.

| Attribute | Type | Description |
|-----------|------|-------------|
| `page` | `int` | 1-based page number |
| `rect` | `tuple` | `(x1, y1, x2, y2)` in PDF points |

### `EncryptResult`

| Attribute | Type | Description |
|-----------|------|-------------|
| `algorithm` | `str` | `"aes128"` or `"rc4"` |
| `key_length` | `int` | Key length in bits |

### `Permissions`

Permission flags for an encrypted PDF.

All attributes are `bool`, default `True`:

| Attribute | Description |
|-----------|-------------|
| `print` | Low-quality printing |
| `modify` | Document modification |
| `copy` | Text/image copying |
| `annotate` | Adding annotations |
| `fill_forms` | Filling form fields |
| `accessibility` | Accessibility tool access |
| `assemble` | Page assembly |
| `print_high_quality` | High-quality printing |

**Class method**: `Permissions.none()` — returns a `Permissions` with all flags set to `False`.

---

## Form types

### `FormField`

| Attribute | Type | Description |
|-----------|------|-------------|
| `name` | `str` | Fully-qualified field name |
| `field_type` | `str` | `"text"`, `"checkbox"`, etc. |
| `value` | `str \| None` | Current value |
| `default_value` | `str \| None` | Default value |
| `page` | `int \| None` | 1-based page |
| `rect` | `tuple \| None` | Position |
| `read_only` | `bool` | Read-only flag |
| `required` | `bool` | Required flag |
| `max_length` | `int` | Max characters (0 = no limit) |
| `options` | `tuple[ChoiceOption, ...]` | Options for combo/list boxes |

### `ChoiceOption`

An option in a combo or list box.

| Attribute | Type | Description |
|-----------|------|-------------|
| `display` | `str` | Display text shown to the user |
| `export_value` | `str` | Value written to the field when selected |

### `FillFormResult`

| Attribute | Type | Description |
|-----------|------|-------------|
| `fields_filled` | `int` | Fields successfully filled |
| `fields_not_found` | `int` | Field names not found |
| `not_found_names` | `tuple[str, ...]` | Names of missing fields |

### `CreateFieldResult`

| Attribute | Type | Description |
|-----------|------|-------------|
| `field_name` | `str` | Name of the created field |
| `created` | `bool` | Whether creation succeeded |

### `ModifyFieldResult`

| Attribute | Type | Description |
|-----------|------|-------------|
| `field_name` | `str` | Name of the targeted field |
| `modified` | `bool` | Whether modification succeeded |

---

## Diff types

### `DiffResult`

| Attribute | Type | Description |
|-----------|------|-------------|
| `page_diffs` | `tuple[PageDiff, ...]` | Per-page results |
| `summary` | `DiffSummary` | Aggregate statistics |

### `DiffSummary`

| Attribute | Type | Description |
|-----------|------|-------------|
| `pages_changed` | `int` | Pages with text differences |
| `pages_added` | `int` | Pages in doc_b only |
| `pages_removed` | `int` | Pages in doc_a only |
| `total_additions` | `int` | Lines added |
| `total_removals` | `int` | Lines removed |
| `total_changes` | `int` | Lines changed |

### `PageDiff`

| Attribute | Type | Description |
|-----------|------|-------------|
| `page` | `int` | 1-based page number |
| `ops` | `tuple[DiffOp, ...]` | Change operations |

### `DiffOp`

| Attribute | Type | Description |
|-----------|------|-------------|
| `kind` | `str` | `"added"`, `"removed"`, `"changed"` |
| `page` | `int` | 1-based page number |
| `text_a` | `str \| None` | Text in document A |
| `text_b` | `str \| None` | Text in document B |
| `bbox_a` | `tuple \| None` | Bbox in document A |
| `bbox_b` | `tuple \| None` | Bbox in document B |
| `line_index_a` | `int \| None` | Line index in document A |
| `line_index_b` | `int \| None` | Line index in document B |

---

## Visual diff types

### `VisualDiffResult`

| Attribute | Type | Description |
|-----------|------|-------------|
| `pages` | `tuple[VisualDiffPage, ...]` | Per-page visual results |
| `overall_similarity` | `float` | Average similarity (0.0–1.0) |
| `text_diff_summary` | `DiffSummary` | Text-level summary |

### `VisualDiffPage`

| Attribute | Type | Description |
|-----------|------|-------------|
| `page` | `int` | 1-based page number |
| `image_a` | `bytes` | Rendered image from document A |
| `image_b` | `bytes` | Rendered image from document B |
| `diff_image` | `bytes` | Diff visualization |
| `similarity` | `float` | Pixel similarity for this page |
| `changed_pixel_count` | `int` | Number of differing pixels |

---

## Rendering types

### `RenderedImage`

| Attribute | Type | Description |
|-----------|------|-------------|
| `data` | `bytes` | Raw image bytes |
| `width` | `int` | Width in pixels |
| `height` | `int` | Height in pixels |
| `format` | `str` | `"png"`, `"jpeg"`, or `"bmp"` |
| `page` | `int` | 1-based source page |

**Methods**: `save(path)` — write image bytes to a file.

---

## Signature types

### `SignatureInfo`

| Attribute | Type | Description |
|-----------|------|-------------|
| `name` | `str` | Signature field name |
| `signer` | `str \| None` | Signer's name |
| `reason` | `str \| None` | Reason for signing |
| `location` | `str \| None` | Signing location |
| `date` | `str \| None` | Signing date |
| `contact_info` | `str \| None` | Contact info |
| `byte_range` | `tuple \| None` | Signed byte ranges |
| `certificate` | `CertificateInfo \| None` | Embedded certificate |
| `covers_whole_document` | `bool` | Whether it covers the full file |
| `has_timestamp` | `bool` | Whether an RFC 3161 timestamp is present |
| `timestamp_date` | `str \| None` | Timestamp date from the TSA |
| `has_ocsp` | `bool` | Whether OCSP responses are embedded |
| `has_crls` | `bool` | Whether CRLs are embedded |

### `CertificateInfo`

| Attribute | Type | Description |
|-----------|------|-------------|
| `subject` | `str` | Certificate subject DN |
| `issuer` | `str` | Certificate issuer DN |
| `serial_number` | `str` | Serial number (hex) |
| `not_before` | `str` | Validity start |
| `not_after` | `str` | Validity end |
| `is_self_signed` | `bool` | Subject equals issuer |

### `SignatureValidity`

| Attribute | Type | Description |
|-----------|------|-------------|
| `name` | `str` | Signature field name |
| `integrity_ok` | `bool` | Hash verification passed |
| `certificate_valid` | `bool` | Certificate date range valid |
| `message` | `str` | Status message |
| `signer` | `str \| None` | Signer name |
| `timestamp_valid` | `bool \| None` | Timestamp token validity |
| `revocation_ok` | `bool \| None` | Revocation info validity |
| `is_ltv` | `bool` | Has long-term validation info |

---

## Validation types

### `ValidationReport`

| Attribute | Type | Description |
|-----------|------|-------------|
| `level` | `str` | Validation level: `"1b"`, `"1a"`, `"2b"` |
| `is_compliant` | `bool` | Whether document passed |
| `issues` | `tuple[ValidationIssue, ...]` | Problems found |
| `fonts_checked` | `int` | Number of fonts inspected |
| `pages_checked` | `int` | Number of pages inspected |

### `ValidationIssue`

| Attribute | Type | Description |
|-----------|------|-------------|
| `severity` | `str` | `"error"`, `"warning"`, `"info"` |
| `rule` | `str` | Rule identifier |
| `message` | `str` | Description |
| `page` | `int \| None` | Page number if applicable |

### `ConversionResult`

| Attribute | Type | Description |
|-----------|------|-------------|
| `level` | `str` | Target conformance level |
| `success` | `bool` | Whether all issues were resolved |
| `actions_taken` | `tuple[ConversionAction, ...]` | Actions performed |
| `remaining_issues` | `tuple[ValidationIssue, ...]` | Unresolved problems |

### `ConversionAction`

| Attribute | Type | Description |
|-----------|------|-------------|
| `category` | `str` | Action category |
| `description` | `str` | Human-readable description |
| `page` | `int \| None` | Page number if applicable |

### `PdfUaReport`

| Attribute | Type | Description |
|-----------|------|-------------|
| `level` | `str` | Validation level (`"1"`) |
| `is_compliant` | `bool` | Whether the document passed |
| `issues` | `tuple[ValidationIssue, ...]` | Problems found |
| `pages_checked` | `int` | Number of pages inspected |
| `structure_elements_checked` | `int` | Structure elements inspected |
