# Layout Analysis

Many real-world PDFs use multi-column layouts — academic papers, newspapers, brochures. Naively extracting text from such documents produces garbled output because text fragments from different columns are interleaved. paperjam's layout engine detects columns and reads them in the correct visual order.

## Analysing page layout

`page.analyze_layout()` returns a `PageLayout` describing the column structure and any detected header/footer zones:

```python
import paperjam

doc = paperjam.open("paper.pdf")
layout = doc.pages[0].analyze_layout(
    min_gutter_width=20.0,          # minimum whitespace between columns (pts)
    max_columns=4,                  # cap on number of columns detected
    detect_headers_footers=True,    # identify header and footer bands
    header_zone_fraction=0.08,      # top 8% of page = header zone
    footer_zone_fraction=0.08,      # bottom 8% of page = footer zone
)

print(f"Columns: {layout.column_count}")
print(f"Multi-column: {layout.is_multi_column}")
print(f"Page size: {layout.page_width:.0f} × {layout.page_height:.0f} pt")
print(f"Gutter positions: {layout.gutters}")
```

`PageLayout` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `page_width` | `float` | Page width in points |
| `page_height` | `float` | Page height in points |
| `column_count` | `int` | Number of body columns detected |
| `gutters` | `tuple[float, ...]` | X positions of gutter centres |
| `regions` | `tuple[LayoutRegion, ...]` | All classified regions |
| `is_multi_column` | `bool` | True if `column_count > 1` |

## Regions

Each `LayoutRegion` covers a rectangular area of the page with a `kind` label:

```python
for region in layout.regions:
    print(f"  {region.kind} col={region.column_index} bbox={region.bbox}")
    for line in region.lines:
        print(f"    {line.text}")
```

Region kinds:

| `kind` | Description |
|--------|-------------|
| `"header"` | Top-of-page header band |
| `"footer"` | Bottom-of-page footer band |
| `"body_column"` | A body text column |
| `"full_width"` | Full-width section (e.g. a large heading) |

`LayoutRegion` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `kind` | `str` | Region classification |
| `column_index` | `int \| None` | 0-based column index (None for header/footer) |
| `bbox` | `tuple` | Bounding box `(x1, y1, x2, y2)` |
| `lines` | `tuple[TextLine, ...]` | Text lines in reading order |

## Layout-aware text extraction

`extract_text_layout()` combines layout analysis and text extraction into one call, returning a plain string in correct reading order:

```python
text = doc.pages[0].extract_text_layout(
    min_gutter_width=20.0,
)
print(text)   # left column first, then right column
```

## Layout-aware structure and Markdown

Pass `layout_aware=True` to `extract_structure()` and `to_markdown()` to respect multi-column layout:

```python
# Structure extraction respecting columns
blocks = doc.extract_structure(layout_aware=True)

# Markdown conversion respecting columns
md = doc.to_markdown(layout_aware=True)
```

## Practical example: two-column academic paper

```python
import paperjam

doc = paperjam.open("ieee-paper.pdf")

for page in doc.pages:
    layout = page.analyze_layout(min_gutter_width=15.0)

    if layout.is_multi_column:
        print(f"Page {page.number}: {layout.column_count}-column layout")
    else:
        print(f"Page {page.number}: single-column")

    # Get body text in correct reading order, skipping headers/footers
    body_text = []
    for region in layout.regions:
        if region.kind in ("body_column", "full_width"):
            for line in region.lines:
                body_text.append(line.text)

    print("\n".join(body_text))
```

## Getting all text in reading order

The `PageLayout.text()` helper returns all text in the order the regions were detected:

```python
layout = page.analyze_layout()
full_text = layout.text()
```
