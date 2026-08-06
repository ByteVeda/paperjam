# Document Manipulation

paperjam uses an **immutable document** pattern. Every manipulation method returns a *new* `Document` object — the original is never modified. This makes it easy to reason about transformations and chain operations safely.

```python
# Original is unchanged; doc2 is the rotated version
doc2 = doc.rotate([(1, 90)])
doc.save("original.pdf")
doc2.save("rotated.pdf")
```

## Splitting

### Split by page ranges

`split()` takes a list of `(start, end)` tuples (1-indexed, inclusive on both ends):

```python
import paperjam

doc = paperjam.open("big-report.pdf")

# Split into two parts: pages 1-5 and pages 6-end
parts = doc.split([(1, 5), (6, doc.page_count)])

parts[0].save("part1.pdf")
parts[1].save("part2.pdf")
```

### Split into individual pages

```python
singles = doc.split_pages()
for i, page_doc in enumerate(singles, 1):
    page_doc.save(f"page_{i:03d}.pdf")
```

## Merging

### Merge Document objects

```python
doc_a = paperjam.open("cover.pdf")
doc_b = paperjam.open("body.pdf")
doc_c = paperjam.open("appendix.pdf")

merged = paperjam.merge([doc_a, doc_b, doc_c])
merged.save("complete.pdf")
```

### Merge files by path

Convenient when you just have file paths and don't need to inspect the documents first:

```python
merged = paperjam.merge_files(["cover.pdf", "body.pdf", "appendix.pdf"])
merged.save("complete.pdf")
```

## Reordering pages

`reorder()` takes a list of 1-indexed page numbers in the desired order. You can repeat page numbers to duplicate pages, or omit them to delete pages:

```python
# Reverse a 3-page document
doc = doc.reorder([3, 2, 1])

# Duplicate page 1 at the beginning and end
doc = doc.reorder([1, 2, 3, 4, 5, 1])

# Keep only pages 1, 3, 5 (drop even pages)
doc = doc.reorder([1, 3, 5])
```

## Rotating pages

`rotate()` takes a list of `(page_number, angle)` tuples. The angle can be an integer (degrees) or a `Rotation` enum value. Rotation is additive — it rotates by the given amount from the current orientation:

```python
from paperjam import Rotation

# Rotate page 1 clockwise 90 degrees
doc = doc.rotate([(1, Rotation.CW_90)])

# Rotate multiple pages
doc = doc.rotate([
    (1, 90),
    (3, 180),
    (5, 270),
])

# Rotate all pages
rotations = [(i, 90) for i in range(1, doc.page_count + 1)]
doc = doc.rotate(rotations)
```

`Rotation` enum values:

| Value | Degrees |
|-------|---------|
| `Rotation.NONE` | 0 |
| `Rotation.CW_90` | 90 |
| `Rotation.CW_180` | 180 |
| `Rotation.CW_270` | 270 |

## Deleting pages

Pass a list of 1-indexed page numbers. At least one page must remain:

```python
# Delete pages 3 and 5
doc = doc.delete_pages([3, 5])

# Delete the first page
doc = doc.delete_pages([1])
```

## Inserting blank pages

`insert_blank_pages()` takes a list of `(after_page, width, height)` tuples. Dimensions are in PDF points (72 pt = 1 inch). Use `after_page=0` to insert before page 1:

```python
# Insert a US Letter blank page before page 1
doc = doc.insert_blank_pages([(0, 612.0, 792.0)])

# Insert an A4 blank page after page 5
doc = doc.insert_blank_pages([(5, 595.0, 842.0)])

# Insert multiple blank pages at once
doc = doc.insert_blank_pages([
    (0, 612.0, 792.0),   # before page 1
    (5, 612.0, 792.0),   # after page 5
])
```

Common page sizes in points:

| Format | Width | Height |
|--------|-------|--------|
| US Letter | 612 | 792 |
| US Legal | 612 | 1008 |
| A4 | 595 | 842 |
| A3 | 842 | 1191 |

## Optimization

`optimize()` applies lossless compression and cleanup to reduce file size:

```python
optimized_doc, result = doc.optimize(
    compress_streams=True,    # compress uncompressed streams
    remove_unused=True,       # remove unreferenced objects
    remove_duplicates=True,   # deduplicate identical stream data
    strip_metadata=False,     # keep document metadata
)

print(f"Original: {result.original_size:,} bytes")
print(f"Optimized: {result.optimized_size:,} bytes")
print(f"Reduced by {result.reduction_percent:.1f}%")
print(f"Objects removed: {result.objects_removed}")
print(f"Streams compressed: {result.streams_compressed}")

optimized_doc.save("smaller.pdf")
```

## Stamping

Overlay a page from another PDF (a stamp or letterhead) onto pages of the current document:

```python
letterhead = paperjam.open("letterhead.pdf")
stamped = doc.stamp(
    letterhead,
    source_page=1,          # which page of the stamp doc to use
    target_pages=None,      # None = all pages
    x=0.0,                  # x offset in points
    y=0.0,                  # y offset in points
    scale=1.0,              # scale factor
    opacity=1.0,            # 0.0 = invisible, 1.0 = opaque
    layer="over",           # "over" or "under"
)
stamped.save("stamped.pdf")
```

To add a background watermark-style stamp under the content:

```python
background = paperjam.open("background.pdf")
doc = doc.stamp(background, layer="under", opacity=0.3)
```
