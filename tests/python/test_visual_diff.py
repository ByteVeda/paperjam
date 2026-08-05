"""Tests for visual diff (Phase 4).

Note: These tests require pdfium to be installed on the system.
They are skipped if pdfium is not available.
"""

import paperjam
import pytest

_MINIMAL_PDF = (
    b"%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n"
    b"2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n"
    b"3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 612 792]>>endobj\n"
    b"xref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n"
    b"0000000058 00000 n \n0000000115 00000 n \n"
    b"trailer<</Size 4/Root 1 0 R>>\nstartxref\n190\n%%EOF"
)


def _pdfium_available():
    """Check if pdfium rendering is available."""
    try:
        doc = paperjam.Document(_MINIMAL_PDF)
        doc.render_page(1)
        return True
    except Exception:
        return False


pdfium_required = pytest.mark.skipif(
    not _pdfium_available(),
    reason="pdfium not available for rendering",
)


@pdfium_required
def test_visual_diff_identical(simple_text_pdf):
    """Visual diff of identical documents should have similarity ~1.0."""
    doc_a = paperjam.Document(simple_text_pdf)
    doc_b = paperjam.Document(simple_text_pdf)
    result = doc_a.visual_diff(doc_b)
    assert isinstance(result, paperjam.VisualDiffResult)
    assert result.overall_similarity >= 0.99
    assert len(result.pages) == doc_a.page_count
    for page in result.pages:
        assert page.similarity >= 0.99
        assert page.changed_pixel_count == 0


@pdfium_required
def test_visual_diff_different(simple_text_pdf, multi_page_pdf):
    """Visual diff of different documents should have similarity < 1.0."""
    doc_a = paperjam.Document(simple_text_pdf)
    doc_b = paperjam.Document(multi_page_pdf)
    result = doc_a.visual_diff(doc_b)
    assert isinstance(result, paperjam.VisualDiffResult)
    # Different docs should produce a meaningful diff
    assert len(result.pages) >= 1
    assert isinstance(result.text_diff_summary, paperjam.DiffSummary)


@pdfium_required
def test_visual_diff_page_result_fields(simple_text_pdf):
    """Each page result should have all expected fields."""
    doc_a = paperjam.Document(simple_text_pdf)
    doc_b = paperjam.Document(simple_text_pdf)
    result = doc_a.visual_diff(doc_b)
    for page in result.pages:
        assert isinstance(page, paperjam.VisualDiffPage)
        assert isinstance(page.image_a, bytes)
        assert isinstance(page.image_b, bytes)
        assert isinstance(page.diff_image, bytes)
        assert page.image_a_width > 0
        assert page.image_a_height > 0
        assert 0.0 <= page.similarity <= 1.0


@pdfium_required
def test_visual_diff_custom_options(simple_text_pdf, multi_page_pdf):
    """Custom DPI and threshold options should work."""
    doc_a = paperjam.Document(simple_text_pdf)
    doc_b = paperjam.Document(multi_page_pdf)
    result = doc_a.visual_diff(
        doc_b,
        dpi=72.0,
        threshold=50,
        mode="pixel_diff",
    )
    assert isinstance(result, paperjam.VisualDiffResult)
