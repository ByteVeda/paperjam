# Table Extraction

paperjam includes a table extraction engine that can detect both ruled (lattice) tables and borderless (stream) tables. The engine is written in Rust for performance, which matters when processing large PDFs with dozens of tables.

## Basic usage

```python
import paperjam
from paperjam import TableStrategy

doc = paperjam.open("financial-report.pdf")
tables = doc.pages[0].extract_tables()

for table in tables:
    print(f"Found table: {table.row_count} rows × {table.col_count} cols")
    print(table.to_list())
```

## Extraction strategies

Three strategies are available:

| Strategy | When to use |
|----------|-------------|
| `TableStrategy.AUTO` | Let paperjam decide (default) |
| `TableStrategy.LATTICE` | Ruled tables with visible cell borders |
| `TableStrategy.STREAM` | Borderless tables with whitespace-aligned columns |

```python
from paperjam import TableStrategy

# Force lattice strategy (ruled tables)
tables = page.extract_tables(strategy=TableStrategy.LATTICE)

# Force stream strategy (whitespace-aligned)
tables = page.extract_tables(strategy=TableStrategy.STREAM)

# String shorthand also works
tables = page.extract_tables(strategy="lattice")
```

## Fine-tuning extraction parameters

All parameters are keyword-only:

```python
tables = page.extract_tables(
    strategy=TableStrategy.AUTO,
    min_rows=2,           # ignore tables smaller than this
    min_cols=2,           # ignore tables narrower than this
    snap_tolerance=3.0,   # how closely cell edges must align (pts)
    row_tolerance=0.5,    # vertical snap tolerance
    min_col_gap=10.0,     # minimum whitespace to split columns (stream)
)
```

## Working with Table objects

### Getting a cell

`cell(row, col)` is 0-indexed and returns `None` if out of range:

```python
cell = table.cell(0, 0)   # top-left cell
if cell:
    print(cell.text, cell.bbox)
    print(cell.col_span, cell.row_span)  # merged cells
```

### Converting to a 2D list

```python
data = table.to_list()   # list[list[str]]
headers = data[0]
rows = data[1:]
```

### CSV export

```python
csv_text = table.to_csv()          # comma-delimited
tsv_text = table.to_csv(delimiter="\t")
with open("table.csv", "w") as f:
    f.write(csv_text)
```

### pandas DataFrame

Requires the `pandas` extra (`pip install "paperjam[pandas]"`).
The first row is used as column headers:

```python
df = table.to_dataframe()
print(df.dtypes)
df.to_excel("table.xlsx", index=False)
```

### Iterating rows

```python
for row in table.rows:
    texts = [cell.text for cell in row.cells]
    print(texts, "y:", row.y_min, "–", row.y_max)
```

## Document-level extraction

To extract tables from all pages at once:

```python
all_tables = doc.extract_tables(strategy=TableStrategy.AUTO)
print(f"Found {len(all_tables)} tables across {doc.page_count} pages")
```

## Checking table boundaries

The `bbox` attribute gives the bounding box of the entire table in PDF points `(x1, y1, x2, y2)`:

```python
for table in tables:
    x1, y1, x2, y2 = table.bbox
    print(f"Table spans {x2 - x1:.0f}pt wide, {y2 - y1:.0f}pt tall")
```

## Which strategy was used?

After extraction, `table.strategy` tells you which algorithm was actually applied:

```python
for table in tables:
    print(f"Used strategy: {table.strategy}")  # "lattice", "stream", or "auto"
```
