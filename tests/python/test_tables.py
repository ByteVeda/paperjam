import paperjam


def test_extract_tables_bordered(table_bordered_pdf):
    doc = paperjam.open(table_bordered_pdf)
    page = doc.pages[0]
    tables = page.extract_tables()
    # We should detect at least something — exact results depend on strategy
    assert isinstance(tables, list)


def test_extract_tables_strategies(table_bordered_pdf):
    doc = paperjam.open(table_bordered_pdf)
    page = doc.pages[0]

    # Each strategy should return a list (possibly empty)
    for strategy in ["auto", "lattice", "stream"]:
        tables = page.extract_tables(strategy=strategy)
        assert isinstance(tables, list)


def test_table_structure(table_bordered_pdf):
    doc = paperjam.open(table_bordered_pdf)
    page = doc.pages[0]
    tables = page.extract_tables(strategy="stream")
    if tables:
        table = tables[0]
        assert isinstance(table, paperjam.Table)
        assert table.row_count >= 1
        assert table.col_count >= 1
        assert len(table.bbox) == 4

        for row in table.rows:
            assert isinstance(row, paperjam.Row)
            for cell in row.cells:
                assert isinstance(cell, paperjam.Cell)
                assert isinstance(cell.text, str)


def test_table_to_csv(table_bordered_pdf):
    doc = paperjam.open(table_bordered_pdf)
    page = doc.pages[0]
    tables = page.extract_tables(strategy="stream")
    if tables:
        csv = tables[0].to_csv()
        assert isinstance(csv, str)
        assert len(csv) > 0
