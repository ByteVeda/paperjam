import tempfile

import paperjam


def test_split_pages(multi_page_pdf):
    doc = paperjam.open(multi_page_pdf)
    pages = doc.split_pages()
    assert len(pages) == 3
    for _, page_doc in enumerate(pages, 1):
        assert page_doc.page_count == 1


def test_split_range(multi_page_pdf):
    doc = paperjam.open(multi_page_pdf)
    parts = doc.split([(1, 2), (3, 3)])
    assert len(parts) == 2
    assert parts[0].page_count == 2
    assert parts[1].page_count == 1


def test_merge(simple_text_pdf, multi_page_pdf):
    doc1 = paperjam.open(simple_text_pdf)
    doc2 = paperjam.open(multi_page_pdf)
    merged = paperjam.merge([doc1, doc2])
    assert merged.page_count == 4


def test_merge_files(simple_text_pdf, multi_page_pdf):
    merged = paperjam.merge_files([simple_text_pdf, multi_page_pdf])
    assert merged.page_count == 4


def test_save_and_reopen(simple_text_pdf):
    doc = paperjam.open(simple_text_pdf)
    data = doc.save_bytes()
    assert len(data) > 0

    doc2 = paperjam.open(data)
    assert doc2.page_count == doc.page_count
    text = doc2.pages[0].extract_text()
    assert "Hello World" in text


def test_save_to_file(simple_text_pdf):
    doc = paperjam.open(simple_text_pdf)
    with tempfile.NamedTemporaryFile(suffix=".pdf", delete=False) as f:
        path = f.name
    doc.save(path)
    doc2 = paperjam.open(path)
    assert doc2.page_count == 1
