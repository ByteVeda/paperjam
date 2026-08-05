import { useDocumentLoader } from '@site/src/hooks/useDocumentLoader';
import type { WasmModule } from '@site/src/types/paperjam';
import { useCallback, useState } from 'react';
import PdfUploader from './PdfUploader';
import styles from './playground.module.css';
import ErrorAlert from './ui/ErrorAlert';
import ResultPanel from './ui/ResultPanel';
import WasmLoader from './WasmLoader';

function MarkdownConversionInner({ wasm }: { wasm: WasmModule }) {
  const { doc, error, loadFile } = useDocumentLoader(wasm);
  const [layoutAware, setLayoutAware] = useState(false);
  const [includePageNumbers, setIncludePageNumbers] = useState(false);
  const [htmlTables, setHtmlTables] = useState(false);
  const [markdown, setMarkdown] = useState('');
  const [converted, setConverted] = useState(false);

  const handleConvert = useCallback(() => {
    if (!doc) return;
    try {
      setMarkdown(doc.toMarkdown(layoutAware, includePageNumbers, htmlTables));
      setConverted(true);
    } catch {
      // useDocumentLoader handles errors
    }
  }, [doc, layoutAware, includePageNumbers, htmlTables]);

  return (
    <div>
      <PdfUploader onFileLoaded={loadFile} />
      <ErrorAlert error={error} />
      {doc && (
        <>
          <div className={styles.toolbar}>
            <div className={styles.checkboxGroup}>
              <label>
                <input
                  type="checkbox"
                  checked={layoutAware}
                  onChange={(e) => setLayoutAware(e.target.checked)}
                />
                Layout-aware
              </label>
              <label>
                <input
                  type="checkbox"
                  checked={includePageNumbers}
                  onChange={(e) => setIncludePageNumbers(e.target.checked)}
                />
                Page numbers
              </label>
              <label>
                <input
                  type="checkbox"
                  checked={htmlTables}
                  onChange={(e) => setHtmlTables(e.target.checked)}
                />
                HTML tables
              </label>
            </div>
            <button
              type="button"
              className={`${styles.btn} ${styles.btnPrimary}`}
              onClick={handleConvert}
            >
              Convert to Markdown
            </button>
          </div>

          {converted && !markdown && (
            <div className={styles.emptyState}>No content extracted.</div>
          )}

          {markdown && (
            <ResultPanel copyText={markdown} copyLabel="Copy Markdown">
              <pre style={{ margin: 0, whiteSpace: 'pre-wrap' }}>
                {markdown}
              </pre>
            </ResultPanel>
          )}
        </>
      )}
    </div>
  );
}

export default function MarkdownConversion() {
  return (
    <WasmLoader>{(wasm) => <MarkdownConversionInner wasm={wasm} />}</WasmLoader>
  );
}
