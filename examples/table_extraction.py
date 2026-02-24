"""Table extraction examples: auto, lattice, stream strategies."""

import time
from pathlib import Path

import paperjam

PDF_PATH = Path(__file__).resolve().parent.parent / "file" / "Learning_Python.pdf"
OUTPUT_DIR = Path(__file__).resolve().parent.parent / "results" / "tables"


def timed(label: str):
    class Timer:
        def __enter__(self):
            self.t0 = time.perf_counter()
            return self

        def __exit__(self, *_):
            self.elapsed = time.perf_counter() - self.t0
            print(f"  [{label}: {self.elapsed:.4f}s]")

    return Timer()


def section(title: str) -> None:
    print(f"\n{'=' * 60}")
    print(f"  {title}")
    print(f"{'=' * 60}")


def main() -> None:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    t_start = time.perf_counter()

    with timed("open"):
        doc = paperjam.open(str(PDF_PATH))
    print(f"  Document: {PDF_PATH.name} ({doc.page_count} pages)")

    # Try multiple pages that might have tables (TOC, appendix, etc.)
    test_pages = [5, 10, 20, 50, 100]

    # ── Auto strategy on several pages ───────────────────────────
    section("Auto strategy — scanning pages")
    tables_found = 0
    for page_num in test_pages:
        if page_num > doc.page_count:
            continue
        with timed(f"extract_tables page {page_num} (auto)"):
            tables = doc.pages[page_num - 1].extract_tables(strategy="auto")
        if tables:
            tables_found += len(tables)
            for i, table in enumerate(tables):
                print(
                    f"    Page {page_num}, Table {i}: "
                    f"{table.row_count}x{table.col_count} "
                    f"strategy={table.strategy}"
                )
                # Save as CSV
                csv_path = OUTPUT_DIR / f"page_{page_num}_auto_table_{i}.csv"
                csv_path.write_text(table.to_csv())
                print(f"    Saved: {csv_path}")

                # Show first row
                if table.row_count > 0:
                    header = [c.text for c in table.rows[0].cells]
                    print(f"    First row: {header}")
        else:
            print(f"    Page {page_num}: no tables found")
    print(f"  Total tables found: {tables_found}")

    # ── Compare strategies on a single page ──────────────────────
    section("Strategy comparison — page 5")
    for strategy in ["auto", "lattice", "stream"]:
        with timed(f"extract_tables ({strategy})"):
            tables = doc.pages[4].extract_tables(strategy=strategy)
        print(f"  {strategy:8s}: {len(tables)} tables")
        for i, table in enumerate(tables):
            print(f"    Table {i}: {table.row_count}x{table.col_count}")
            csv_path = OUTPUT_DIR / f"page_5_{strategy}_table_{i}.csv"
            csv_path.write_text(table.to_csv())
            print(f"    Saved: {csv_path}")

    # ── Table structure details ──────────────────────────────────
    section("Table structure details")
    # Find a page with tables by scanning more pages
    for page_num in range(1, min(201, doc.page_count + 1)):
        tables = doc.pages[page_num - 1].extract_tables()
        if tables and tables[0].row_count >= 3:
            table = tables[0]
            print(f"  Found table on page {page_num}")
            print(f"  Rows: {table.row_count}, Cols: {table.col_count}")
            print(f"  Strategy: {table.strategy}")
            print("  as list (first 3 rows):")
            for row in table.to_list()[:3]:
                print(f"    {row}")
            csv_path = OUTPUT_DIR / f"detailed_table_page_{page_num}.csv"
            csv_path.write_text(table.to_csv())
            print(f"  Saved: {csv_path}")
            break
    else:
        print("  No tables with 3+ rows found in first 200 pages")

    # ── Summary ──────────────────────────────────────────────────
    total_time = time.perf_counter() - t_start
    print(f"\n{'=' * 60}")
    print(f"  Total: {total_time:.3f}s")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    main()
