# Metadata & Bookmarks

paperjam lets you read and write document-level metadata and manage the bookmark (outline) tree. All write operations follow the immutable pattern and return a new `Document`.

## Reading metadata

`doc.metadata` returns a frozen `Metadata` dataclass:

```python
import paperjam

doc = paperjam.open("report.pdf")
meta = doc.metadata

print(meta.title)
print(meta.author)
print(meta.subject)
print(meta.keywords)
print(meta.creator)       # application that created the document
print(meta.producer)      # PDF library used to write the file
print(meta.creation_date)
print(meta.modification_date)
print(meta.pdf_version)   # e.g. "1.7"
print(meta.page_count)
print(meta.is_encrypted)
print(meta.xmp_metadata)  # raw XMP XML string, or None
```

`Metadata` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `title` | `str \| None` | Document title |
| `author` | `str \| None` | Author field |
| `subject` | `str \| None` | Subject field |
| `keywords` | `str \| None` | Keywords field |
| `creator` | `str \| None` | Originating application |
| `producer` | `str \| None` | PDF-writing library |
| `creation_date` | `str \| None` | ISO 8601 date string |
| `modification_date` | `str \| None` | ISO 8601 date string |
| `pdf_version` | `str` | PDF specification version, e.g. `"1.7"` |
| `page_count` | `int` | Total number of pages |
| `is_encrypted` | `bool` | Whether the document is password-protected |
| `xmp_metadata` | `str \| None` | Full XMP XML string, if present |

## Writing metadata

`set_metadata()` returns a new `Document` with the updated fields. Pass a string to set a field, `None` to remove it, or omit it entirely to leave it unchanged:

```python
doc2 = doc.set_metadata(
    title="Annual Report 2024",
    author="Finance Team",
    subject="Financial Results",
    keywords="finance, annual, report",
    creator=None,      # remove the creator field
    producer=None,     # remove the producer field
)
doc2.save("updated.pdf")
```

## Reading bookmarks

`doc.bookmarks` returns a list of top-level `Bookmark` objects. Each `Bookmark` can have `children`, forming a nested tree:

```python
def print_toc(bookmarks, indent=0):
    for b in bookmarks:
        print("  " * indent + f"[p{b.page}] {b.title}")
        print_toc(b.children, indent + 1)

print_toc(doc.bookmarks)
```

`Bookmark` attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| `title` | `str` | Display text of the bookmark |
| `page` | `int` | 1-based destination page number |
| `level` | `int` | Nesting level (0 = top) |
| `children` | `tuple[Bookmark, ...]` | Nested child bookmarks |

## Writing bookmarks

`set_bookmarks()` replaces the entire outline tree:

```python
from paperjam import Bookmark

toc = [
    Bookmark(title="Introduction", page=1, level=0, children=(
        Bookmark(title="Background", page=2, level=1),
        Bookmark(title="Motivation", page=4, level=1),
    )),
    Bookmark(title="Methods", page=6, level=0, children=(
        Bookmark(title="Data Collection", page=7, level=1),
        Bookmark(title="Analysis", page=10, level=1),
    )),
    Bookmark(title="Results", page=14, level=0),
    Bookmark(title="Conclusion", page=20, level=0),
]

doc2 = doc.set_bookmarks(toc)
doc2.save("with-toc.pdf")
```

Pass an empty list to remove all bookmarks:

```python
doc2 = doc.set_bookmarks([])
```

## Auto-generating a TOC

`generate_toc()` analyses the document's heading structure and creates bookmarks automatically:

```python
doc2, bookmarks = doc.generate_toc(
    max_depth=3,               # only include headings up to H3
    heading_size_ratio=1.2,    # font size ratio to detect headings
    layout_aware=False,        # set True for multi-column docs
    replace_existing=True,     # overwrite existing bookmarks
)

print(f"Generated {len(bookmarks)} top-level bookmarks")
doc2.save("with-auto-toc.pdf")
```

The returned `bookmarks` list is the same tree that was written to the document, which you can inspect or reuse.
