import pathlib

import pytest

FIXTURES = pathlib.Path(__file__).resolve().parent.parent / "fixtures"


@pytest.fixture
def fixtures_dir():
    return FIXTURES


@pytest.fixture
def simple_text_pdf():
    return str(FIXTURES / "simple_text.pdf")


@pytest.fixture
def multi_page_pdf():
    return str(FIXTURES / "multi_page.pdf")


@pytest.fixture
def table_bordered_pdf():
    return str(FIXTURES / "table_bordered.pdf")


@pytest.fixture
def metadata_pdf():
    return str(FIXTURES / "with_metadata.pdf")
