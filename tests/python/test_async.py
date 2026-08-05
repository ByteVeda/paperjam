"""Tests for async wrappers."""

from __future__ import annotations

import asyncio

import paperjam
import pytest
import pytest_asyncio

pytestmark = pytest.mark.asyncio


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest_asyncio.fixture
async def async_doc(simple_text_pdf):
    return await paperjam.aopen(simple_text_pdf)


@pytest_asyncio.fixture
async def async_multi_doc(multi_page_pdf):
    return await paperjam.aopen(multi_page_pdf)


# ---------------------------------------------------------------------------
# Document async tests
# ---------------------------------------------------------------------------


async def test_aopen(simple_text_pdf):
    doc = await paperjam.aopen(simple_text_pdf)
    assert doc.page_count == 1


async def test_aopen_bytes(simple_text_pdf):
    with open(simple_text_pdf, "rb") as f:
        data = f.read()
    doc = await paperjam.aopen(data)
    assert doc.page_count == 1


async def test_asave_bytes(async_doc):
    data = await async_doc.asave_bytes()
    assert isinstance(data, bytes)
    assert len(data) > 0


async def test_asave(async_doc, tmp_path):
    out = tmp_path / "output.pdf"
    await async_doc.asave(str(out))
    assert out.exists()
    assert out.stat().st_size > 0


async def test_ato_markdown(async_doc):
    sync_md = async_doc.to_markdown()
    async_md = await async_doc.ato_markdown()
    assert async_md == sync_md


async def test_asearch(async_doc):
    sync_results = async_doc.search("Hello")
    async_results = await async_doc.asearch("Hello")
    assert len(async_results) == len(sync_results)
    for s, a in zip(sync_results, async_results, strict=True):
        assert s.text == a.text
        assert s.page == a.page


async def test_adiff(simple_text_pdf):
    doc_a = await paperjam.aopen(simple_text_pdf)
    doc_b = await paperjam.aopen(simple_text_pdf)
    result = await doc_a.adiff(doc_b)
    assert result.summary.total_changes == 0
    assert result.summary.total_additions == 0
    assert result.summary.total_removals == 0


# ---------------------------------------------------------------------------
# Page async tests
# ---------------------------------------------------------------------------


async def test_page_aextract_text(async_doc):
    page = async_doc.pages[0]
    sync_text = page.extract_text()
    async_text = await page.aextract_text()
    assert async_text == sync_text


async def test_page_ato_markdown(async_doc):
    page = async_doc.pages[0]
    sync_md = page.to_markdown()
    async_md = await page.ato_markdown()
    assert async_md == sync_md


# ---------------------------------------------------------------------------
# Top-level async functions
# ---------------------------------------------------------------------------


async def test_ato_markdown_toplevel(simple_text_pdf):
    sync_md = paperjam.to_markdown(simple_text_pdf)
    async_md = await paperjam.ato_markdown(simple_text_pdf)
    assert async_md == sync_md


async def test_amerge(simple_text_pdf):
    doc1 = await paperjam.aopen(simple_text_pdf)
    doc2 = await paperjam.aopen(simple_text_pdf)
    merged = await paperjam.amerge([doc1, doc2])
    assert merged.page_count == 2


# ---------------------------------------------------------------------------
# Concurrency
# ---------------------------------------------------------------------------


async def test_concurrent_operations(simple_text_pdf):
    """Multiple async operations should run concurrently without errors."""
    doc = await paperjam.aopen(simple_text_pdf)
    results = await asyncio.gather(
        doc.ato_markdown(),
        doc.asearch("Hello"),
        doc.asave_bytes(),
    )
    assert isinstance(results[0], str)
    assert isinstance(results[1], list)
    assert isinstance(results[2], bytes)
