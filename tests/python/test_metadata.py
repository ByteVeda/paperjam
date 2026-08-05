import paperjam


def test_metadata_version(simple_text_pdf):
    doc = paperjam.open(simple_text_pdf)
    meta = doc.metadata
    assert isinstance(meta.pdf_version, str)
    assert "." in meta.pdf_version


def test_metadata_page_count(multi_page_pdf):
    doc = paperjam.open(multi_page_pdf)
    meta = doc.metadata
    assert meta.page_count == 3


def test_metadata_title(metadata_pdf):
    doc = paperjam.open(metadata_pdf)
    meta = doc.metadata
    assert meta.title == "Test Document Title"


def test_metadata_author(metadata_pdf):
    doc = paperjam.open(metadata_pdf)
    meta = doc.metadata
    assert meta.author == "Test Author"


def test_metadata_subject(metadata_pdf):
    doc = paperjam.open(metadata_pdf)
    meta = doc.metadata
    assert meta.subject == "Test Subject"


def test_metadata_creator(metadata_pdf):
    doc = paperjam.open(metadata_pdf)
    meta = doc.metadata
    assert meta.creator == "Test Creator"


def test_metadata_is_frozen(simple_text_pdf):
    doc = paperjam.open(simple_text_pdf)
    meta = doc.metadata
    assert isinstance(meta, paperjam.Metadata)
