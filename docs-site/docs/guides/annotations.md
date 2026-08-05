# Annotations & Watermarks

paperjam supports reading existing annotations, adding new ones, removing annotations by type or index, and stamping pages with text watermarks.

## Reading annotations

Every `Page` exposes an `annotations` property returning a list of `Annotation` objects:

```python
import paperjam

doc = paperjam.open("reviewed.pdf")

for page in doc.pages:
    for annot in page.annotations:
        print(f"  [{annot.type}] {annot.contents!r} at {annot.rect}")
        if annot.author:
            print(f"    by {annot.author}")
        if annot.url:
            print(f"    url: {annot.url}")
```

`Annotation` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `type` | `str` | Annotation type string (see `AnnotationType` values) |
| `rect` | `tuple` | Bounding box `(x1, y1, x2, y2)` in PDF points |
| `contents` | `str \| None` | Annotation text / tooltip |
| `author` | `str \| None` | Author name |
| `color` | `tuple \| None` | RGB float tuple, each component 0.0–1.0 |
| `creation_date` | `str \| None` | ISO 8601 date string |
| `opacity` | `float \| None` | 0.0–1.0 |
| `url` | `str \| None` | URL for link annotations |
| `destination` | `dict \| None` | Internal page destination for link annotations |

## Adding annotations

`add_annotation()` returns a new `Document`:

```python
from paperjam import AnnotationType

# Highlight annotation
doc2 = doc.add_annotation(
    page=1,
    annotation_type=AnnotationType.HIGHLIGHT,
    rect=(100.0, 700.0, 350.0, 720.0),
    contents="Important passage",
    color=(1.0, 1.0, 0.0),   # yellow
    opacity=0.5,
)

# Sticky note
doc2 = doc.add_annotation(
    page=2,
    annotation_type=AnnotationType.TEXT,
    rect=(50.0, 600.0, 70.0, 620.0),
    contents="Review this section",
    author="Alice",
    color=(1.0, 0.0, 0.0),  # red
)

# Link to an external URL
doc2 = doc.add_annotation(
    page=1,
    annotation_type=AnnotationType.LINK,
    rect=(100.0, 100.0, 300.0, 120.0),
    url="https://example.com",
)

# Underline
doc2 = doc.add_annotation(
    page=1,
    annotation_type=AnnotationType.UNDERLINE,
    rect=(72.0, 500.0, 400.0, 512.0),
)

doc2.save("annotated.pdf")
```

You can also pass the type as a string:

```python
doc2 = doc.add_annotation(page=1, annotation_type="highlight", rect=(100, 700, 300, 720))
```

### Supported annotation types

| `AnnotationType` | String | Description |
|-----------------|--------|-------------|
| `TEXT` | `"text"` | Sticky note (comment) |
| `LINK` | `"link"` | Hyperlink |
| `FREE_TEXT` | `"free_text"` | Free-text callout |
| `HIGHLIGHT` | `"highlight"` | Text highlight |
| `UNDERLINE` | `"underline"` | Underline |
| `STRIKE_OUT` | `"strike_out"` | Strikethrough |
| `SQUARE` | `"square"` | Rectangle shape |
| `CIRCLE` | `"circle"` | Ellipse shape |
| `LINE` | `"line"` | Line annotation |
| `STAMP` | `"stamp"` | Rubber-stamp annotation |

## Removing annotations

`remove_annotations()` returns a tuple `(new_document, count_removed)`:

```python
# Remove all annotations from page 1
doc2, n = doc.remove_annotations(page=1)
print(f"Removed {n} annotations")

# Remove only highlights from page 2
doc2, n = doc.remove_annotations(
    page=2,
    annotation_types=[AnnotationType.HIGHLIGHT],
)

# Remove by index (0-based)
doc2, n = doc.remove_annotations(page=1, indices=[0, 2])
```

## Watermarks

`add_watermark()` stamps a text watermark onto pages. It returns a new `Document`:

```python
from paperjam import WatermarkPosition, WatermarkLayer

doc2 = doc.add_watermark(
    "CONFIDENTIAL",
    font_size=60.0,
    rotation=45.0,
    opacity=0.3,
    color=(0.5, 0.5, 0.5),   # grey
    position=WatermarkPosition.CENTER,
    layer=WatermarkLayer.OVER,
    pages=None,               # None = all pages
)
doc2.save("confidential.pdf")
```

Apply only to specific pages:

```python
doc2 = doc.add_watermark("DRAFT", pages=[1, 2, 3])
```

Use a custom position by supplying both `x` and `y` (in PDF points). When both are given the `position` parameter is ignored:

```python
doc2 = doc.add_watermark(
    "Internal use only",
    font_size=12.0,
    rotation=0.0,
    opacity=0.8,
    x=72.0,     # 1 inch from left
    y=36.0,     # 0.5 inch from bottom
)
```

### Watermark parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `text` | `str` | — | Watermark text |
| `font_size` | `float` | `60.0` | Font size in points |
| `rotation` | `float` | `45.0` | Rotation angle in degrees |
| `opacity` | `float` | `0.3` | Opacity (0.0–1.0) |
| `color` | `tuple` | `(0.5, 0.5, 0.5)` | RGB, each 0.0–1.0 |
| `font` | `str` | `"Helvetica"` | Font name |
| `position` | `WatermarkPosition` | `CENTER` | Preset position |
| `layer` | `WatermarkLayer` | `OVER` | Over or under content |
| `pages` | `list[int] \| None` | `None` | Pages to stamp; None = all |
| `x` | `float \| None` | `None` | Custom X position (overrides `position`) |
| `y` | `float \| None` | `None` | Custom Y position (overrides `position`) |
