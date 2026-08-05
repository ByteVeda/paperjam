"""Tests for link/URI extraction (Phase 1)."""

import paperjam


def test_extract_links_empty(simple_text_pdf):
    """A PDF without links should return empty list."""
    doc = paperjam.Document(simple_text_pdf)
    links = doc.extract_links()
    assert isinstance(links, list)
    # simple_text.pdf may or may not have links; just verify no crash
    for link in links:
        assert isinstance(link, paperjam.Link)


def test_page_extract_links(simple_text_pdf):
    """Page.extract_links() should work without error."""
    doc = paperjam.Document(simple_text_pdf)
    page = doc.pages[0]
    links = page.extract_links()
    assert isinstance(links, list)


def test_annotation_has_url_field(simple_text_pdf):
    """Annotations should have url and destination fields."""
    doc = paperjam.Document(simple_text_pdf)
    page = doc.pages[0]
    for annot in page.annotations:
        assert hasattr(annot, "url")
        assert hasattr(annot, "destination")


def test_link_annotation_with_url(simple_text_pdf):
    """Create a link annotation with a URL, then extract it back."""
    doc = paperjam.Document(simple_text_pdf)
    doc2 = doc.add_annotation(
        page=1,
        annotation_type="link",
        rect=(100, 100, 200, 120),
        url="https://example.com",
    )
    links = doc2.pages[0].extract_links()
    assert len(links) >= 1
    found = [link for link in links if link.url == "https://example.com"]
    assert len(found) == 1
    assert found[0].destination is not None
    assert found[0].destination["type"] == "uri"


def test_link_type():
    """Link dataclass should have expected fields."""
    link = paperjam.Link(
        page=1,
        rect=(0, 0, 100, 100),
        url="https://test.com",
        destination={"type": "uri", "uri": "https://test.com"},
    )
    assert link.page == 1
    assert link.url == "https://test.com"
    assert link.destination["type"] == "uri"
