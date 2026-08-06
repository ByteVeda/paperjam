import { useDocumentLoader } from '@site/src/hooks/useDocumentLoader';
import type { TableResult, WasmModule } from '@site/src/types/paperjam';
import { useCallback, useState } from 'react';
import PdfUploader from './PdfUploader';
import styles from './playground.module.css';
import CopyButton from './ui/CopyButton';
import ErrorAlert from './ui/ErrorAlert';
import PageSelector from './ui/PageSelector';
import WasmLoader from './WasmLoader';

function tableToCsv(table: TableResult): string {
  return table.rows
    .map((row) =>
      row
        .map((cell) => {
          const escaped = String(cell).replace(/"/g, '""');
          return escaped.includes(',') ||
            escaped.includes('"') ||
            escaped.includes('\n')
            ? `"${escaped}"`
            : escaped;
        })
        .join(','),
    )
    .join('\n');
}

function TableExtractionInner({ wasm }: { wasm: WasmModule }) {
  const { doc, pageCount, error, loadFile } = useDocumentLoader(wasm);
  const [page, setPage] = useState(1);
  const [tables, setTables] = useState<TableResult[]>([]);
  const [extracted, setExtracted] = useState(false);

  const handlePageChange = useCallback((newPage: number) => {
    setPage(newPage);
    setTables([]);
    setExtracted(false);
  }, []);

  const handleExtract = useCallback(() => {
    if (!doc) return;
    try {
      setTables(doc.extractTables(page));
      setExtracted(true);
    } catch {
      // useDocumentLoader handles errors
    }
  }, [doc, page]);

  return (
    <div>
      <PdfUploader onFileLoaded={loadFile} />
      <ErrorAlert error={error} />
      {doc && (
        <>
          <div className={styles.toolbar}>
            <PageSelector
              page={page}
              pageCount={pageCount}
              onChange={handlePageChange}
            />
            <button
              type="button"
              className={`${styles.btn} ${styles.btnPrimary}`}
              onClick={handleExtract}
            >
              Extract Tables
            </button>
          </div>

          {extracted && tables.length === 0 && (
            <div className={styles.emptyState}>
              No tables found on page {page}.
            </div>
          )}

          {tables.map((table, i) => (
            <div key={i} className={styles.tableCard}>
              <div className={styles.tableCardHeader}>
                <span>
                  <strong>Table {i + 1}</strong> &mdash; {table.row_count} rows
                  x {table.col_count} cols
                  {table.strategy && ` (${table.strategy})`}
                </span>
                <CopyButton text={tableToCsv(table)} label="Copy as CSV" />
              </div>
              <div className={styles.tableOverflow}>
                <table className={styles.table}>
                  <tbody>
                    {table.rows.map((row, ri) => (
                      <tr key={ri}>
                        {row.map((cell, ci) => (
                          <td key={ci}>{String(cell)}</td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              {table.bbox && (
                <div
                  style={{
                    fontSize: '0.8rem',
                    color: 'var(--ifm-color-emphasis-500)',
                    marginTop: '0.5rem',
                  }}
                >
                  Bounding box: [
                  {table.bbox.map((v) => v.toFixed(1)).join(', ')}]
                </div>
              )}
            </div>
          ))}
        </>
      )}
    </div>
  );
}

export default function TableExtraction() {
  return (
    <WasmLoader>{(wasm) => <TableExtractionInner wasm={wasm} />}</WasmLoader>
  );
}
