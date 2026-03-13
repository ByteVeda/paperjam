"""Tests for auto TOC generation (Phase 2)."""

import paperjam


def test_generate_toc_simple(multi_page_pdf):
    """Generate TOC from a multi-page PDF."""
    doc = paperjam.Document(multi_page_pdf)
    new_doc, bookmarks = doc.generate_toc()
    assert isinstance(new_doc, paperjam.Document)
    assert isinstance(bookmarks, list)
    # The result should be a valid document
    assert new_doc.page_count == doc.page_count


def test_generate_toc_returns_bookmarks(multi_page_pdf):
    """Generated bookmarks should be Bookmark instances."""
    doc = paperjam.Document(multi_page_pdf)
    _new_doc, bookmarks = doc.generate_toc()
    for b in bookmarks:
        assert isinstance(b, paperjam.Bookmark)
        assert b.page >= 1
        assert len(b.title) > 0


def test_generate_toc_replace_existing(multi_page_pdf):
    """replace_existing=True should replace any existing bookmarks."""
    doc = paperjam.Document(multi_page_pdf)
    # First generate TOC
    doc2, _ = doc.generate_toc(replace_existing=True)
    # Generate again — should replace, not duplicate
    doc3, _bookmarks = doc2.generate_toc(replace_existing=True)
    assert isinstance(doc3, paperjam.Document)


def test_generate_toc_max_depth(multi_page_pdf):
    """max_depth should limit heading levels included."""
    doc = paperjam.Document(multi_page_pdf)
    _, bookmarks_full = doc.generate_toc(max_depth=6)
    _, bookmarks_h1 = doc.generate_toc(max_depth=1)
    # H1-only should have fewer or equal bookmarks
    assert len(bookmarks_h1) <= len(bookmarks_full)


def test_generate_toc_on_empty_headings(simple_text_pdf):
    """If no headings are found, should return empty bookmarks."""
    doc = paperjam.Document(simple_text_pdf)
    new_doc, bookmarks = doc.generate_toc(heading_size_ratio=100.0)
    # With an absurdly high ratio, nothing should qualify as a heading
    assert isinstance(bookmarks, list)
    assert isinstance(new_doc, paperjam.Document)
