import React, { useState, useCallback, useMemo } from 'react';
import type { WasmModule, MetadataResult, PageInfo, StructureBlock } from '@site/src/types/paperjam';
import { useDocumentLoader } from '@site/src/hooks/useDocumentLoader';
import WasmLoader from './WasmLoader';
import PdfUploader from './PdfUploader';
import ErrorAlert from './ui/ErrorAlert';
import Tabs from './ui/Tabs';
import styles from './playground.module.css';

const TABS = [
  { id: 'metadata', label: 'Metadata' },
  { id: 'pages', label: 'Pages' },
  { id: 'structure', label: 'Structure' },
];

function DocumentInfoInner({ wasm }: { wasm: WasmModule }) {
  const { doc, pageCount, error, loadFile } = useDocumentLoader(wasm);
  const [metadata, setMetadata] = useState<MetadataResult | null>(null);
  const [pages, setPages] = useState<PageInfo[]>([]);
  const [structure, setStructure] = useState<StructureBlock[]>([]);
  const [activeTab, setActiveTab] = useState('metadata');

  const handleFileLoaded = useCallback(
    (data: Uint8Array, name: string) => {
      loadFile(data, name);
      try {
        const d = new wasm.WasmDocument(data);
        const count = d.pageCount();

        try { setMetadata(d.metadata()); } catch { setMetadata(null); }

        const pageInfos: PageInfo[] = [];
        for (let i = 1; i <= count; i++) {
          try { pageInfos.push(d.pageInfo(i)); } catch { pageInfos.push({ number: i, width: 0, height: 0, rotation: 0 }); }
        }
        setPages(pageInfos);

        try { setStructure(d.extractStructure()); } catch { setStructure([]); }

        setActiveTab('metadata');
      } catch {
        // useDocumentLoader handles errors
      }
    },
    [wasm, loadFile],
  );

  const metadataEntries = useMemo(() => {
    if (!metadata) return [];
    return Object.entries(metadata).filter(([, v]) => v != null && v !== '');
  }, [metadata]);

  return (
    <div>
      <PdfUploader onFileLoaded={handleFileLoaded} />
      <ErrorAlert error={error} />
      {doc && (
        <>
          <Tabs
            tabs={TABS.map((t) => t.id === 'pages' ? { ...t, label: `Pages (${pageCount})` } : t)}
            active={activeTab}
            onChange={setActiveTab}
          />

          {activeTab === 'metadata' && (
            metadataEntries.length === 0 ? (
              <div className={styles.emptyState}>No metadata available.</div>
            ) : (
              <table className={styles.metadataTable}>
                <thead><tr><th>Property</th><th>Value</th></tr></thead>
                <tbody>
                  {metadataEntries.map(([key, value]) => (
                    <tr key={key}>
                      <th>{key}</th>
                      <td>{typeof value === 'object' ? JSON.stringify(value) : String(value)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )
          )}

          {activeTab === 'pages' && (
            <div>
              <p><strong>{pageCount}</strong> page{pageCount !== 1 ? 's' : ''} in this document.</p>
              <div className={styles.pageDimensions}>
                {pages.map((p, i) => (
                  <div key={i} className={styles.pageCard}>
                    <strong>Page {p.number}</strong>
                    <span>{p.width.toFixed(1)} x {p.height.toFixed(1)} pt</span>
                    {p.rotation !== 0 && <span> | Rotation: {p.rotation}&deg;</span>}
                  </div>
                ))}
              </div>
            </div>
          )}

          {activeTab === 'structure' && (
            structure.length === 0 ? (
              <div className={styles.emptyState}>No structure elements found.</div>
            ) : (
              <div className={styles.resultPanel} style={{ maxHeight: '600px' }}>
                <ul className={styles.structureList}>
                  {structure.map((item, i) => {
                    const isHeading = item.block_type.toLowerCase().includes('heading');
                    const indent = item.level ? (item.level - 1) * 1.25 : 0;
                    return (
                      <li key={i} className={styles.structureItem} style={{ paddingLeft: `${indent + 0.5}rem` }}>
                        <span className={`${styles.structureBadge} ${isHeading ? styles.structureBadgeHeading : ''}`}>
                          {item.block_type}
                        </span>
                        <span className={styles.structureText}>
                          {item.text}
                          {item.page != null && (
                            <span style={{ color: 'var(--ifm-color-emphasis-500)', fontSize: '0.8rem', marginLeft: '0.5rem' }}>
                              (p.{item.page})
                            </span>
                          )}
                        </span>
                      </li>
                    );
                  })}
                </ul>
              </div>
            )
          )}
        </>
      )}
    </div>
  );
}

export default function DocumentInfo() {
  return <WasmLoader>{(wasm) => <DocumentInfoInner wasm={wasm} />}</WasmLoader>;
}
