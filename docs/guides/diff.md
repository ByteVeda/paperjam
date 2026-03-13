# Comparing Documents

paperjam can compare two PDF documents at both the text level and the visual (pixel) level. Text diff is fast and works without any optional dependencies. Visual diff requires the `render` feature (pdfium).

## Text diff

`diff()` is available as both a module-level function and a `Document` method:

```python
import paperjam

doc_a = paperjam.open("v1.pdf")
doc_b = paperjam.open("v2.pdf")

# Both forms are equivalent
result = paperjam.diff(doc_a, doc_b)
result = doc_a.diff(doc_b)
```

### Reading the summary

```python
s = result.summary
print(f"Pages changed:  {s.pages_changed}")
print(f"Pages added:    {s.pages_added}")
print(f"Pages removed:  {s.pages_removed}")
print(f"Total additions:{s.total_additions}")
print(f"Total removals: {s.total_removals}")
print(f"Total changes:  {s.total_changes}")
```

`DiffSummary` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `pages_changed` | `int` | Pages present in both docs but with different text |
| `pages_added` | `int` | Pages in doc_b but not doc_a |
| `pages_removed` | `int` | Pages in doc_a but not doc_b |
| `total_additions` | `int` | Lines added across all pages |
| `total_removals` | `int` | Lines removed across all pages |
| `total_changes` | `int` | Lines changed across all pages |

### Per-page diff operations

```python
for page_diff in result.page_diffs:
    if not page_diff.ops:
        continue
    print(f"\n--- Page {page_diff.page} ---")
    for op in page_diff.ops:
        if op.kind == "added":
            print(f"  + {op.text_b!r}")
        elif op.kind == "removed":
            print(f"  - {op.text_a!r}")
        elif op.kind == "changed":
            print(f"  ~ {op.text_a!r}  →  {op.text_b!r}")
```

`DiffOp` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `kind` | `str` | `"added"`, `"removed"`, or `"changed"` |
| `page` | `int` | 1-based page number |
| `text_a` | `str \| None` | Text in doc_a (None for "added") |
| `text_b` | `str \| None` | Text in doc_b (None for "removed") |
| `bbox_a` | `tuple \| None` | Bounding box in doc_a |
| `bbox_b` | `tuple \| None` | Bounding box in doc_b |
| `line_index_a` | `int \| None` | 0-based line index in doc_a |
| `line_index_b` | `int \| None` | 0-based line index in doc_b |

## Visual diff

Visual diff renders both documents and computes a pixel-level comparison. It requires the `render` feature:

```python
result = doc_a.visual_diff(
    doc_b,
    dpi=150,                           # render resolution
    highlight_color=(255, 0, 0, 128),  # RGBA, semi-transparent red
    mode="both",                       # "pixel_diff", "bbox_overlay", or "both"
    threshold=10,                      # per-channel difference to count as changed
)
```

### Modes

| Mode | Description |
|------|-------------|
| `"pixel_diff"` | Highlights every changed pixel |
| `"bbox_overlay"` | Draws bounding boxes around changed regions |
| `"both"` | Combines pixel diff and bounding boxes |

### Reading visual diff results

```python
print(f"Overall similarity: {result.overall_similarity:.1%}")

for vp in result.pages:
    print(f"\nPage {vp.page}:")
    print(f"  Similarity:      {vp.similarity:.1%}")
    print(f"  Changed pixels:  {vp.changed_pixel_count}")

    # Save the side-by-side diff image
    with open(f"diff_page_{vp.page}.png", "wb") as f:
        f.write(vp.diff_image)

    # Save the individual rendered pages
    with open(f"a_page_{vp.page}.png", "wb") as f:
        f.write(vp.image_a)
    with open(f"b_page_{vp.page}.png", "wb") as f:
        f.write(vp.image_b)
```

`VisualDiffResult` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `pages` | `tuple[VisualDiffPage, ...]` | Per-page results |
| `overall_similarity` | `float` | Average similarity across all pages (0.0–1.0) |
| `text_diff_summary` | `DiffSummary` | Text-level diff summary |

`VisualDiffPage` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `page` | `int` | 1-based page number |
| `image_a` | `bytes` | Rendered page from doc_a (PNG) |
| `image_b` | `bytes` | Rendered page from doc_b (PNG) |
| `diff_image` | `bytes` | Diff visualization (PNG) |
| `similarity` | `float` | Pixel similarity ratio for this page |
| `changed_pixel_count` | `int` | Number of pixels that differ |

### Async visual diff

```python
result = await doc_a.adiff(doc_b)   # text diff
```
