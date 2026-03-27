import { useDocumentLoader } from '@site/src/hooks/useDocumentLoader';
import type { SearchMatch, WasmModule } from '@site/src/types/paperjam';
import type React from 'react';
import { useCallback, useMemo, useState } from 'react';
import PdfUploader from './PdfUploader';
import styles from './playground.module.css';
import ErrorAlert from './ui/ErrorAlert';
import WasmLoader from './WasmLoader';

function highlightText(
  text: string,
  query: string,
  caseSensitive: boolean,
): React.ReactNode {
  if (!query) return text;
  const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const flags = caseSensitive ? 'g' : 'gi';
  const parts = text.split(new RegExp(`(${escaped})`, flags));
  return parts.map((part, i) => {
    const isMatch = caseSensitive
      ? part === query
      : part.toLowerCase() === query.toLowerCase();
    return isMatch ? (
      <span key={i} className={styles.highlight}>
        {part}
      </span>
    ) : (
      part
    );
  });
}

function SearchInner({ wasm }: { wasm: WasmModule }) {
  const { doc, error, loadFile } = useDocumentLoader(wasm);
  const [query, setQuery] = useState('');
  const [caseSensitive, setCaseSensitive] = useState(false);
  const [results, setResults] = useState<SearchMatch[]>([]);
  const [searched, setSearched] = useState(false);

  const handleFileLoaded = useCallback(
    (data: Uint8Array, name: string) => {
      loadFile(data, name);
      setResults([]);
      setSearched(false);
      setQuery('');
    },
    [loadFile],
  );

  const handleSearch = useCallback(() => {
    if (!doc || !query.trim()) {
      setResults([]);
      return;
    }
    try {
      const matches = doc.searchText(query, caseSensitive);
      setResults(matches);
      setSearched(true);
    } catch {
      setResults([]);
      setSearched(true);
    }
  }, [doc, query, caseSensitive]);

  const groupedResults = useMemo(() => {
    const groups = new Map<number, SearchMatch[]>();
    for (const match of results) {
      const existing = groups.get(match.page);
      if (existing) {
        existing.push(match);
      } else {
        groups.set(match.page, [match]);
      }
    }
    return groups;
  }, [results]);

  return (
    <div>
      <PdfUploader onFileLoaded={handleFileLoaded} />
      <ErrorAlert error={error} />
      {doc && (
        <>
          <div className={styles.toolbar}>
            <input
              type="text"
              className={styles.searchInput}
              placeholder="Search text..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleSearch();
              }}
              aria-label="Search query"
            />
            <div className={styles.checkboxGroup}>
              <label>
                <input
                  type="checkbox"
                  checked={caseSensitive}
                  onChange={(e) => setCaseSensitive(e.target.checked)}
                />
                Case sensitive
              </label>
            </div>
            <button
              type="button"
              className={`${styles.btn} ${styles.btnPrimary}`}
              onClick={handleSearch}
            >
              Search
            </button>
          </div>

          {searched && (
            <div className={styles.searchResults}>
              <strong>
                {results.length} result{results.length !== 1 ? 's' : ''} found
                {results.length > 0 &&
                  ` across ${groupedResults.size} page${groupedResults.size !== 1 ? 's' : ''}`}
              </strong>
            </div>
          )}

          {results.length > 0 && (
            <div
              className={styles.resultPanel}
              style={{ maxHeight: '600px', marginTop: '0.75rem' }}
            >
              {Array.from(groupedResults.entries()).map(([page, matches]) => (
                <div key={page} style={{ marginBottom: '1rem' }}>
                  <div
                    style={{
                      fontWeight: 600,
                      marginBottom: '0.25rem',
                      fontSize: '0.9rem',
                    }}
                  >
                    Page {page} ({matches.length} match
                    {matches.length !== 1 ? 'es' : ''})
                  </div>
                  {matches.map((match, i) => (
                    <div
                      key={i}
                      style={{ marginBottom: '0.25rem', paddingLeft: '1rem' }}
                    >
                      <span
                        style={{
                          color: 'var(--ifm-color-emphasis-500)',
                          fontSize: '0.8rem',
                          marginRight: '0.5rem',
                        }}
                      >
                        Line {match.line_number}:
                      </span>
                      {highlightText(match.text, query, caseSensitive)}
                    </div>
                  ))}
                </div>
              ))}
            </div>
          )}

          {searched && results.length === 0 && (
            <div className={styles.emptyState}>No results found.</div>
          )}
        </>
      )}
    </div>
  );
}

export default function Search() {
  return <WasmLoader>{(wasm) => <SearchInner wasm={wasm} />}</WasmLoader>;
}
