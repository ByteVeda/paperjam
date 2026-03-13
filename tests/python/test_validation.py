"""Tests for PDF/A validation (Phase 5)."""

import paperjam


def test_validate_simple_pdf(simple_text_pdf):
    """A basic PDF should not be PDF/A compliant (no XMP, no embedded fonts)."""
    doc = paperjam.Document(simple_text_pdf)
    report = doc.validate_pdf_a()
    assert isinstance(report, paperjam.ValidationReport)
    assert report.level == "1b"
    assert report.pages_checked >= 1
    # Simple text PDF is unlikely to be PDF/A compliant
    assert not report.is_compliant
    assert len(report.issues) > 0


def test_validate_returns_issues(simple_text_pdf):
    """Validation issues should have correct structure."""
    doc = paperjam.Document(simple_text_pdf)
    report = doc.validate_pdf_a()
    for issue in report.issues:
        assert isinstance(issue, paperjam.ValidationIssue)
        assert issue.severity in ("error", "warning", "info")
        assert len(issue.rule) > 0
        assert len(issue.message) > 0


def test_validate_level_2b(simple_text_pdf):
    """Validation with level 2b should work."""
    doc = paperjam.Document(simple_text_pdf)
    report = doc.validate_pdf_a(level="2b")
    assert report.level == "2b"
    assert isinstance(report.is_compliant, bool)


def test_validate_level_1a(simple_text_pdf):
    """Validation with level 1a should work."""
    doc = paperjam.Document(simple_text_pdf)
    report = doc.validate_pdf_a(level="1a")
    assert report.level == "1a"


def test_validate_multi_page(multi_page_pdf):
    """Validation should check all pages."""
    doc = paperjam.Document(multi_page_pdf)
    report = doc.validate_pdf_a()
    assert report.pages_checked == doc.page_count


def test_validate_checks_encryption(simple_text_pdf):
    """A non-encrypted PDF should not have encryption issues."""
    doc = paperjam.Document(simple_text_pdf)
    report = doc.validate_pdf_a()
    encryption_issues = [i for i in report.issues if i.rule == "encryption"]
    # simple_text.pdf is not encrypted
    assert len(encryption_issues) == 0


def test_validate_checks_xmp(simple_text_pdf):
    """Validation should check for XMP metadata."""
    doc = paperjam.Document(simple_text_pdf)
    report = doc.validate_pdf_a()
    xmp_issues = [i for i in report.issues if i.rule.startswith("xmp.")]
    # Most simple PDFs won't have proper XMP
    assert len(xmp_issues) > 0
