# Page

```python
class paperjam.Page
```

A single page in a PDF document. Pages are lazily parsed — content is only decoded when you call an extraction method.

Pages cannot be created directly. Access them through `doc.pages`:

```python
page  = doc.pages[0]     # 0-indexed
pages = doc.pages[1:5]   # slice
for page in doc.pages:   # iterate
    ...
```

---

## Properties

### `number`

```python
page.number: int
```

1-based page number.

### `width`

```python
page.width: float
```

Page width in PDF points (72 pt = 1 inch).

### `height`

```python
page.height: float
```

Page height in PDF points.

### `rotation`

```python
page.rotation: int
```

Page rotation in degrees: `0`, `90`, `180`, or `270`.

### `info`

```python
page.info: PageInfo
```

All four properties as a frozen `PageInfo` dataclass.

### `annotations`

```python
page.annotations: list[Annotation]
```

All annotations on this page.

---

## Text extraction

### `extract_text`

```python
page.extract_text() -> str
```

Extract all text from the page as a plain string.

### `extract_text_lines`

```python
page.extract_text_lines() -> list[TextLine]
```

Extract text grouped into lines. Each `TextLine` has `.text`, `.bbox`, and `.spans`.

### `extract_text_spans`

```python
page.extract_text_spans() -> list[TextSpan]
```

Extract all text as individual positioned fragments. Each `TextSpan` has `.text`, `.x`, `.y`, `.width`, `.font_name`, `.font_size`.

### `extract_text_layout`

```python
page.extract_text_layout(
    *,
    min_gutter_width: float = 20.0,
    max_columns: int = 4,
    detect_headers_footers: bool = True,
    header_zone_fraction: float = 0.08,
    footer_zone_fraction: float = 0.08,
) -> str
```

Extract text in layout-aware reading order (respects multi-column layout).

---

## Table extraction

### `extract_tables`

```python
page.extract_tables(
    *,
    strategy: TableStrategy | str = TableStrategy.AUTO,
    min_rows: int = 2,
    min_cols: int = 2,
    snap_tolerance: float = 3.0,
    row_tolerance: float = 0.5,
    min_col_gap: float = 10.0,
) -> list[Table]
```

Extract tables from this page.

---

## Image extraction

### `extract_images`

```python
page.extract_images() -> list[Image]
```

Extract all images embedded in this page.

---

## Searching

### `search`

```python
page.search(
    query: str,
    *,
    case_sensitive: bool = True,
    use_regex: bool = False,
) -> list[SearchResult]
```

Search for text on this page.

---

## Links

### `extract_links`

```python
page.extract_links() -> list[Link]
```

Extract all hyperlinks from this page.

---

## Structure and Markdown

### `extract_structure`

```python
page.extract_structure(
    *,
    heading_size_ratio: float = 1.2,
    detect_lists: bool = True,
    include_tables: bool = True,
    layout_aware: bool = False,
) -> list[ContentBlock]
```

Extract structured content blocks (headings, paragraphs, list items, tables).

### `to_markdown`

```python
page.to_markdown(
    *,
    heading_offset: int = 0,
    html_tables: bool = False,
    layout_aware: bool = False,
    # ... additional formatting options
) -> str
```

Convert this page to Markdown.

---

## Layout analysis

### `analyze_layout`

```python
page.analyze_layout(
    *,
    min_gutter_width: float = 20.0,
    max_columns: int = 4,
    detect_headers_footers: bool = True,
    header_zone_fraction: float = 0.08,
    footer_zone_fraction: float = 0.08,
) -> PageLayout
```

Analyse the page layout. Returns a `PageLayout` describing columns, gutters, and classified regions.

---

## Rendering

### `render`

```python
page.render(
    *,
    dpi: float = 150,
    format: str = "png",
    quality: int = 85,
    background_color: tuple[int, int, int] | None = None,
    scale_to_width: int | None = None,
    scale_to_height: int | None = None,
) -> RenderedImage
```

Render this page to an image. Requires the `render` feature.

---

## Async methods

| Async method | Equivalent sync |
|-------------|----------------|
| `await page.aextract_text()` | `page.extract_text()` |
| `await page.aextract_tables()` | `page.extract_tables()` |
| `await page.ato_markdown()` | `page.to_markdown()` |
