import { useDocumentLoader } from '@site/src/hooks/useDocumentLoader';
import type {
  SearchMatch,
  TextLine,
  WasmModule,
} from '@site/src/types/paperjam';
import type React from 'react';
import { useCallback, useMemo, useState } from 'react';
import PdfUploader from './PdfUploader';
import styles from './playground.module.css';
import ErrorAlert from './ui/ErrorAlert';
import PageSelector from './ui/PageSelector';
import ResultPanel from './ui/ResultPanel';
import Tabs from './ui/Tabs';
import WasmLoader from './WasmLoader';

const TABS = [
  { id: 'plain', label: 'Plain Text' },
  { id: 'lines', label: 'Text Lines' },
];

function highlightText(text: string, query: string): React.ReactNode {
  if (!query) return text;
  const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const parts = text.split(new RegExp(`(${escaped})`, 'gi'));
  return parts.map((part, i) =>
    part.toLowerCase() === query.toLowerCase() ? (
      <span key={i} className={styles.highlight}>
        {part}
      </span>
    ) : (
      part
    ),
  );
}

function TextExtractionInner({ wasm }: { wasm: WasmModule }) {
  const { doc, pageCount, error, loadFile } = useDocumentLoader(wasm);
  const [page, setPage] = useState(1);
  const [activeTab, setActiveTab] = useState('plain');
  const [plainText, setPlainText] = useState('');
  const [lines, setLines] = useState<TextLine[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<SearchMatch[]>([]);

  const handleFileLoaded = useCallback(
    (data: Uint8Array, name: string) => {
      loadFile(data, name);
      try {
        const d = new wasm.WasmDocument(data);
        setPage(1);
        setPlainText(d.extractText(1));
        setLines(d.extractTextLines(1));
        setSearchQuery('');
        setSearchResults([]);
      } catch {
        // useDocumentLoader handles errors
      }
    },
    [wasm, loadFile],
  );

  const handlePageChange = useCallback(
    (newPage: number) => {
      if (!doc) return;
      setPage(newPage);
      try {
        setPlainText(doc.extractText(newPage));
        setLines(doc.extractTextLines(newPage));
      } catch {
        // silently handle
      }
    },
    [doc],
  );

  const handleSearch = useCallback(() => {
    if (!doc || !searchQuery.trim()) {
      setSearchResults([]);
      return;
    }
    try {
      setSearchResults(doc.searchText(searchQuery, false));
    } catch {
      // silently handle
    }
  }, [doc, searchQuery]);

  const filteredLines = useMemo(() => {
    if (!searchQuery.trim()) return lines;
    return lines.filter((line) =>
      line.text.toLowerCase().includes(searchQuery.toLowerCase()),
    );
  }, [lines, searchQuery]);

  return (
    <div>
      <PdfUploader onFileLoaded={handleFileLoaded} />
      <ErrorAlert error={error} />
      {doc && (
        <>
          <div className={styles.toolbar}>
            <PageSelector
              page={page}
              pageCount={pageCount}
              onChange={handlePageChange}
            />
            <input
              type="text"
              className={styles.searchInput}
              placeholder="Search text..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleSearch();
              }}
              aria-label="Search text"
            />
            <button
              type="button"
              className={`${styles.btn} ${styles.btnPrimary}`}
              onClick={handleSearch}
            >
              Search All Pages
            </button>
          </div>

          {searchResults.length > 0 && (
            <div className={styles.searchResults}>
              <strong>
                {searchResults.length} result
                {searchResults.length !== 1 ? 's' : ''} across all pages:
              </strong>
              <div
                className={styles.resultPanel}
                style={{
                  maxHeight: '200px',
                  marginTop: '0.5rem',
                  marginBottom: '1rem',
                }}
              >
                {searchResults.map((r, i) => (
                  <div key={i} style={{ marginBottom: '0.25rem' }}>
                    <strong>
                      Page {r.page}, Line {r.line_number}:
                    </strong>{' '}
                    {highlightText(r.text, searchQuery)}
                  </div>
                ))}
              </div>
            </div>
          )}

          <Tabs tabs={TABS} active={activeTab} onChange={setActiveTab} />

          {activeTab === 'plain' && (
            <ResultPanel copyText={plainText}>
              <pre style={{ margin: 0, whiteSpace: 'pre-wrap' }}>
                {searchQuery
                  ? highlightText(plainText, searchQuery)
                  : plainText}
              </pre>
            </ResultPanel>
          )}

          {activeTab === 'lines' && (
            <div>
              {filteredLines.length === 0 ? (
                <div className={styles.emptyState}>No lines found.</div>
              ) : (
                <div style={{ overflowX: 'auto' }}>
                  <table className={styles.table}>
                    <thead>
                      <tr>
                        <th>#</th>
                        <th>Text</th>
                        <th>Bounding Box</th>
                        <th>Spans</th>
                      </tr>
                    </thead>
                    <tbody>
                      {filteredLines.map((line, i) => (
                        <tr key={i}>
                          <td>{i + 1}</td>
                          <td>
                            {searchQuery
                              ? highlightText(line.text, searchQuery)
                              : line.text}
                          </td>
                          <td
                            style={{
                              fontFamily: 'monospace',
                              fontSize: '0.8rem',
                              whiteSpace: 'nowrap',
                            }}
                          >
                            [{line.bbox.map((v) => v.toFixed(1)).join(', ')}]
                          </td>
                          <td>{line.spans?.length ?? 0}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </div>
          )}
        </>
      )}
    </div>
  );
}

export default function TextExtraction() {
  return (
    <WasmLoader>{(wasm) => <TextExtractionInner wasm={wasm} />}</WasmLoader>
  );
}
