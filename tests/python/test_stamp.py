"""Tests for page stamping/overlay (Phase 3)."""

import paperjam


def test_stamp_basic(simple_text_pdf, multi_page_pdf):
    """Stamp one PDF onto another — page count should be unchanged."""
    doc = paperjam.Document(multi_page_pdf)
    stamp = paperjam.Document(simple_text_pdf)
    result = doc.stamp(stamp)
    assert isinstance(result, paperjam.Document)
    assert result.page_count == doc.page_count


def test_stamp_specific_pages(simple_text_pdf, multi_page_pdf):
    """Stamp onto specific pages only."""
    doc = paperjam.Document(multi_page_pdf)
    stamp = paperjam.Document(simple_text_pdf)
    result = doc.stamp(stamp, target_pages=[1])
    assert result.page_count == doc.page_count


def test_stamp_with_scale(simple_text_pdf, multi_page_pdf):
    """Stamp with a scale factor."""
    doc = paperjam.Document(multi_page_pdf)
    stamp = paperjam.Document(simple_text_pdf)
    result = doc.stamp(stamp, scale=0.5, x=100, y=100)
    assert result.page_count == doc.page_count


def test_stamp_under_layer(simple_text_pdf, multi_page_pdf):
    """Stamp with under layer."""
    doc = paperjam.Document(multi_page_pdf)
    stamp = paperjam.Document(simple_text_pdf)
    result = doc.stamp(stamp, layer="under")
    assert result.page_count == doc.page_count


def test_stamp_with_opacity(simple_text_pdf, multi_page_pdf):
    """Stamp with opacity."""
    doc = paperjam.Document(multi_page_pdf)
    stamp = paperjam.Document(simple_text_pdf)
    result = doc.stamp(stamp, opacity=0.5)
    assert result.page_count == doc.page_count


def test_stamp_preserves_text(simple_text_pdf, multi_page_pdf):
    """Stamping should preserve the original document's text."""
    doc = paperjam.Document(multi_page_pdf)
    original_text = doc.pages[0].extract_text()
    stamp = paperjam.Document(simple_text_pdf)
    result = doc.stamp(stamp)
    # Original text should still be extractable
    result_text = result.pages[0].extract_text()
    assert len(result_text) >= len(original_text) * 0.5  # Allow some variance


def test_stamp_saveable(simple_text_pdf, multi_page_pdf):
    """Stamped document should be saveable to bytes."""
    doc = paperjam.Document(multi_page_pdf)
    stamp = paperjam.Document(simple_text_pdf)
    result = doc.stamp(stamp)
    data = result.save_bytes()
    assert len(data) > 0
    # Re-open should work
    reopened = paperjam.Document(data)
    assert reopened.page_count == doc.page_count
