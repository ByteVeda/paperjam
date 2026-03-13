# Enumerations

All enums are importable directly from `paperjam`:

```python
from paperjam import TableStrategy, Rotation, AnnotationType, WatermarkPosition, WatermarkLayer, ImageFormat, FormFieldType
```

---

## `TableStrategy`

Controls which algorithm is used for table extraction.

| Member | Value | Description |
|--------|-------|-------------|
| `AUTO` | `"auto"` | Let paperjam choose based on page content |
| `LATTICE` | `"lattice"` | Ruled tables with visible cell borders |
| `STREAM` | `"stream"` | Borderless tables using whitespace alignment |

```python
from paperjam import TableStrategy

tables = page.extract_tables(strategy=TableStrategy.LATTICE)
tables = page.extract_tables(strategy="lattice")   # string shorthand
```

---

## `Rotation`

Page rotation angles.

| Member | Value | Description |
|--------|-------|-------------|
| `NONE` | `0` | No rotation |
| `CW_90` | `90` | 90° clockwise |
| `CW_180` | `180` | 180° (upside-down) |
| `CW_270` | `270` | 270° clockwise (90° counter-clockwise) |

```python
from paperjam import Rotation

doc = doc.rotate([(1, Rotation.CW_90), (2, Rotation.CW_180)])
doc = doc.rotate([(1, 90)])    # integer degrees also accepted
```

---

## `AnnotationType`

Type of PDF annotation to add or filter.

| Member | Value | Description |
|--------|-------|-------------|
| `TEXT` | `"text"` | Sticky note (pop-up comment) |
| `LINK` | `"link"` | Hyperlink |
| `FREE_TEXT` | `"free_text"` | Free-text callout |
| `HIGHLIGHT` | `"highlight"` | Text highlight |
| `UNDERLINE` | `"underline"` | Underline mark |
| `STRIKE_OUT` | `"strike_out"` | Strikethrough mark |
| `SQUARE` | `"square"` | Rectangle shape |
| `CIRCLE` | `"circle"` | Ellipse shape |
| `LINE` | `"line"` | Line annotation |
| `STAMP` | `"stamp"` | Rubber-stamp annotation |

```python
from paperjam import AnnotationType

# Add a highlight
doc = doc.add_annotation(
    page=1,
    annotation_type=AnnotationType.HIGHLIGHT,
    rect=(100, 700, 300, 720),
)

# Remove only highlights from page 2
doc, n = doc.remove_annotations(page=2, annotation_types=[AnnotationType.HIGHLIGHT])
```

---

## `WatermarkPosition`

Preset positions for text watermarks.

| Member | Value | Description |
|--------|-------|-------------|
| `CENTER` | `"center"` | Page centre |
| `TOP_LEFT` | `"top_left"` | Top-left corner |
| `TOP_RIGHT` | `"top_right"` | Top-right corner |
| `BOTTOM_LEFT` | `"bottom_left"` | Bottom-left corner |
| `BOTTOM_RIGHT` | `"bottom_right"` | Bottom-right corner |

```python
from paperjam import WatermarkPosition

doc = doc.add_watermark("DRAFT", position=WatermarkPosition.CENTER)
```

When both `x` and `y` are provided to `add_watermark()`, the `position` parameter is ignored.

---

## `WatermarkLayer`

Whether the watermark appears over or under the page content.

| Member | Value | Description |
|--------|-------|-------------|
| `OVER` | `"over"` | Rendered on top of page content |
| `UNDER` | `"under"` | Rendered behind page content |

```python
from paperjam import WatermarkLayer

# Background watermark visible through content
doc = doc.add_watermark("CONFIDENTIAL", layer=WatermarkLayer.UNDER, opacity=0.15)
```

---

## `ImageFormat`

Output image format for page rendering.

| Member | Value | Description |
|--------|-------|-------------|
| `PNG` | `"png"` | Lossless PNG |
| `JPEG` | `"jpeg"` | Lossy JPEG |
| `BMP` | `"bmp"` | Uncompressed BMP |

This enum is provided for convenience; render methods also accept the string values directly:

```python
from paperjam import ImageFormat

img = doc.render_page(1, format=ImageFormat.PNG.value)
img = doc.render_page(1, format="png")   # equivalent
```

---

## `FormFieldType`

Type of interactive form field.

| Member | Value | Description |
|--------|-------|-------------|
| `TEXT` | `"text"` | Single or multi-line text input |
| `CHECKBOX` | `"checkbox"` | Boolean checkbox |
| `RADIO_BUTTON` | `"radio_button"` | One-of-N radio button |
| `COMBO_BOX` | `"combo_box"` | Dropdown selector |
| `LIST_BOX` | `"list_box"` | Scrollable list |
| `PUSH_BUTTON` | `"push_button"` | Clickable button |
| `SIGNATURE` | `"signature"` | Digital signature placeholder |

`FormField.field_type` returns the string value (e.g. `"text"`). When calling `add_form_field()`, pass the string value directly:

```python
doc, result = doc.add_form_field("email", "text", page=1, rect=(72, 680, 300, 700))
doc, result = doc.add_form_field("agree", "checkbox", page=1, rect=(72, 650, 90, 668))
```
