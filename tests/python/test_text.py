import paperjam


def test_extract_text(simple_text_pdf):
    doc = paperjam.open(simple_text_pdf)
    text = doc.pages[0].extract_text()
    assert "Hello World" in text
    assert "Second line" in text
    assert "Third line" in text


def test_extract_text_multipage(multi_page_pdf):
    doc = paperjam.open(multi_page_pdf)
    for i, page in enumerate(doc.pages, 1):
        text = page.extract_text()
        assert f"Page {i}" in text


def test_extract_text_spans(simple_text_pdf):
    doc = paperjam.open(simple_text_pdf)
    spans = doc.pages[0].extract_text_spans()
    assert len(spans) >= 3
    texts = [s.text for s in spans]
    assert any("Hello World" in t for t in texts)


def test_text_span_properties(simple_text_pdf):
    doc = paperjam.open(simple_text_pdf)
    spans = doc.pages[0].extract_text_spans()
    for span in spans:
        assert isinstance(span.text, str)
        assert isinstance(span.x, float)
        assert isinstance(span.y, float)
        assert isinstance(span.width, float)
        assert isinstance(span.font_size, float)
        assert isinstance(span.font_name, str)
        assert span.font_size > 0


def test_extract_text_lines(simple_text_pdf):
    doc = paperjam.open(simple_text_pdf)
    lines = doc.pages[0].extract_text_lines()
    assert len(lines) >= 3
    texts = [line.text for line in lines]
    assert any("Hello World" in t for t in texts)


def test_text_line_has_bbox(simple_text_pdf):
    doc = paperjam.open(simple_text_pdf)
    lines = doc.pages[0].extract_text_lines()
    for line in lines:
        assert hasattr(line, "bbox")
        assert len(line.bbox) == 4
        x_min, y_min, x_max, y_max = line.bbox
        assert x_max >= x_min
        assert y_max >= y_min


def test_text_line_has_spans(simple_text_pdf):
    doc = paperjam.open(simple_text_pdf)
    lines = doc.pages[0].extract_text_lines()
    for line in lines:
        assert len(line.spans) >= 1
        for span in line.spans:
            assert isinstance(span, paperjam.TextSpan)
