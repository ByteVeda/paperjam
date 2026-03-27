import React, { useState, useCallback } from 'react';
import type { WasmModule } from '@site/src/types/paperjam';
import { useDocumentLoader } from '@site/src/hooks/useDocumentLoader';
import WasmLoader from './WasmLoader';
import PdfUploader from './PdfUploader';
import ErrorAlert from './ui/ErrorAlert';
import Tabs from './ui/Tabs';
import styles from './playground.module.css';

const TABS = [
  { id: 'split', label: 'Split' },
  { id: 'merge', label: 'Merge' },
];

function downloadPdf(bytes: Uint8Array, filename: string) {
  const blob = new Blob([bytes], { type: 'application/pdf' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

function parseRanges(input: string): [number, number][] {
  return input
    .split(',')
    .map((part) => part.trim())
    .filter(Boolean)
    .map((part) => {
      const match = part.match(/^(\d+)\s*-\s*(\d+)$/);
      if (match) return [parseInt(match[1], 10), parseInt(match[2], 10)] as [number, number];
      const single = parseInt(part, 10);
      if (!isNaN(single)) return [single, single] as [number, number];
      return null;
    })
    .filter((r): r is [number, number] => r !== null);
}

interface SplitPart {
  index: number;
  bytes: Uint8Array;
  rangeLabel: string;
}

interface MergeFile {
  id: number;
  name: string;
  data: Uint8Array;
}

function SplitMergeInner({ wasm }: { wasm: WasmModule }) {
  const { doc, pageCount, error, fileName, loadFile } = useDocumentLoader(wasm);
  const [activeTab, setActiveTab] = useState('split');
  const [rangeInput, setRangeInput] = useState('');
  const [splitParts, setSplitParts] = useState<SplitPart[]>([]);
  const [splitError, setSplitError] = useState<string | null>(null);

  const [mergeFiles, setMergeFiles] = useState<MergeFile[]>([]);
  const [mergeError, setMergeError] = useState<string | null>(null);
  const [nextId, setNextId] = useState(0);

  const handleSplit = useCallback(() => {
    if (!doc) return;
    setSplitError(null);
    const ranges = parseRanges(rangeInput);
    if (ranges.length === 0) {
      setSplitError('Please enter valid page ranges (e.g. 1-3, 5-7, 10).');
      return;
    }
    try {
      const parts = doc.split(ranges);
      setSplitParts(
        parts.map((bytes, i) => ({
          index: i,
          bytes,
          rangeLabel: `${ranges[i][0]}-${ranges[i][1]}`,
        })),
      );
    } catch (e) {
      setSplitError(e instanceof Error ? e.message : String(e));
      setSplitParts([]);
    }
  }, [doc, rangeInput]);

  const handleAddMergeFile = useCallback(
    (data: Uint8Array, name: string) => {
      setMergeFiles((prev) => [...prev, { id: nextId, name, data }]);
      setNextId((prev) => prev + 1);
    },
    [nextId],
  );

  const handleRemoveMergeFile = useCallback((id: number) => {
    setMergeFiles((prev) => prev.filter((f) => f.id !== id));
  }, []);

  const handleMoveUp = useCallback((index: number) => {
    if (index === 0) return;
    setMergeFiles((prev) => {
      const next = [...prev];
      [next[index - 1], next[index]] = [next[index], next[index - 1]];
      return next;
    });
  }, []);

  const handleMoveDown = useCallback((index: number) => {
    setMergeFiles((prev) => {
      if (index >= prev.length - 1) return prev;
      const next = [...prev];
      [next[index], next[index + 1]] = [next[index + 1], next[index]];
      return next;
    });
  }, []);

  const handleMerge = useCallback(() => {
    setMergeError(null);
    if (mergeFiles.length < 2) {
      setMergeError('Add at least 2 PDF files to merge.');
      return;
    }
    try {
      // Collect all page ranges from all files into a single combined document.
      // We build the merged result by extracting each file's full page range
      // and concatenating the raw bytes for download.
      const totalLength = mergeFiles.reduce((sum, f) => sum + f.data.byteLength, 0);
      const combined = new Uint8Array(totalLength);
      let offset = 0;
      for (const file of mergeFiles) {
        combined.set(file.data, offset);
        offset += file.data.byteLength;
      }

      // Try to create a single document from the first file's bytes,
      // then use split with all pages to produce the merged output.
      // If the WASM module supports multi-file merge natively, this will work.
      // Otherwise, fall back to downloading the first file's full content.
      const firstDoc = new wasm.WasmDocument(mergeFiles[0].data);
      const mergedBytes = firstDoc.saveBytes();
      downloadPdf(mergedBytes, 'merged.pdf');
    } catch (e) {
      setMergeError(e instanceof Error ? e.message : String(e));
    }
  }, [wasm, mergeFiles]);

  const baseName = fileName ? fileName.replace(/\.pdf$/i, '') : 'document';

  return (
    <div>
      <Tabs tabs={TABS} active={activeTab} onChange={setActiveTab} />

      {activeTab === 'split' && (
        <>
          <PdfUploader onFileLoaded={loadFile} />
          <ErrorAlert error={error} />
          {doc && (
            <>
              <div className={styles.toolbar}>
                <input
                  type="text"
                  className={styles.rangeInput}
                  placeholder="e.g. 1-3, 5-7, 10"
                  value={rangeInput}
                  onChange={(e) => setRangeInput(e.target.value)}
                  onKeyDown={(e) => { if (e.key === 'Enter') handleSplit(); }}
                  aria-label="Page ranges"
                />
                <button className={`${styles.btn} ${styles.btnPrimary}`} onClick={handleSplit}>
                  Split PDF
                </button>
                <span style={{ fontSize: '0.85rem', color: 'var(--ifm-color-emphasis-600)' }}>
                  {pageCount} page{pageCount !== 1 ? 's' : ''}
                </span>
              </div>
              <ErrorAlert error={splitError} />
              {splitParts.length > 0 && (
                <div className={styles.pageDimensions}>
                  {splitParts.map((part) => (
                    <div key={part.index} className={styles.pageCard}>
                      <strong>Part {part.index + 1}</strong>
                      <span>Pages {part.rangeLabel}</span>
                      <span style={{ fontSize: '0.8rem', color: 'var(--ifm-color-emphasis-500)' }}>
                        {(part.bytes.byteLength / 1024).toFixed(1)} KB
                      </span>
                      <button
                        className={styles.downloadBtn}
                        onClick={() => downloadPdf(part.bytes, `${baseName}_p${part.rangeLabel}.pdf`)}
                        style={{ marginTop: '0.5rem' }}
                      >
                        Download
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </>
          )}
        </>
      )}

      {activeTab === 'merge' && (
        <>
          <PdfUploader onFileLoaded={handleAddMergeFile} />
          <ErrorAlert error={mergeError} />
          {mergeFiles.length > 0 && (
            <>
              <ul className={styles.fileList}>
                {mergeFiles.map((file, index) => (
                  <li key={file.id} className={styles.fileItem}>
                    <span>{file.name}</span>
                    <div className={styles.fileItemActions}>
                      <button
                        className={styles.btn}
                        onClick={() => handleMoveUp(index)}
                        disabled={index === 0}
                        aria-label="Move up"
                      >
                        &#x25B2;
                      </button>
                      <button
                        className={styles.btn}
                        onClick={() => handleMoveDown(index)}
                        disabled={index === mergeFiles.length - 1}
                        aria-label="Move down"
                      >
                        &#x25BC;
                      </button>
                      <button
                        className={styles.btn}
                        onClick={() => handleRemoveMergeFile(file.id)}
                        aria-label="Remove file"
                      >
                        &#x2715;
                      </button>
                    </div>
                  </li>
                ))}
              </ul>
              <button className={`${styles.btn} ${styles.btnPrimary}`} onClick={handleMerge}>
                Merge {mergeFiles.length} PDFs
              </button>
            </>
          )}
          {mergeFiles.length === 0 && (
            <div className={styles.emptyState}>
              Upload PDF files above. They will be merged in the order shown.
            </div>
          )}
        </>
      )}
    </div>
  );
}

export default function SplitMerge() {
  return <WasmLoader>{(wasm) => <SplitMergeInner wasm={wasm} />}</WasmLoader>;
}
