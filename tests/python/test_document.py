import paperjam
import pytest


def test_open_file(simple_text_pdf):
    doc = paperjam.open(simple_text_pdf)
    assert doc.page_count == 1


def test_open_nonexistent_file():
    with pytest.raises(FileNotFoundError):
        paperjam.open("/tmp/nonexistent_file.pdf")


def test_open_bytes(simple_text_pdf):
    with open(simple_text_pdf, "rb") as f:
        data = f.read()
    doc = paperjam.open(data)
    assert doc.page_count == 1


def test_context_manager(simple_text_pdf):
    with paperjam.open(simple_text_pdf) as doc:
        assert doc.page_count == 1
        text = doc.pages[0].extract_text()
        assert "Hello World" in text


def test_page_count(multi_page_pdf):
    doc = paperjam.open(multi_page_pdf)
    assert doc.page_count == 3


def test_page_indexing(multi_page_pdf):
    doc = paperjam.open(multi_page_pdf)
    page = doc.pages[0]
    assert page.number == 1

    page = doc.pages[-1]
    assert page.number == 3


def test_page_slicing(multi_page_pdf):
    doc = paperjam.open(multi_page_pdf)
    pages = doc.pages[0:2]
    assert len(pages) == 2
    assert pages[0].number == 1
    assert pages[1].number == 2


def test_page_out_of_range(simple_text_pdf):
    doc = paperjam.open(simple_text_pdf)
    with pytest.raises(IndexError):
        _ = doc.pages[5]


def test_page_iteration(multi_page_pdf):
    doc = paperjam.open(multi_page_pdf)
    page_numbers = [p.number for p in doc.pages]
    assert page_numbers == [1, 2, 3]


def test_page_dimensions(simple_text_pdf):
    doc = paperjam.open(simple_text_pdf)
    page = doc.pages[0]
    # A4 dimensions (approximately)
    assert 590 < page.width < 600
    assert 840 < page.height < 845
