import React, { useState, useCallback } from 'react';
import type { WasmModule, StructureBlock, LayoutResult } from '@site/src/types/paperjam';
import { useDocumentLoader } from '@site/src/hooks/useDocumentLoader';
import WasmLoader from './WasmLoader';
import PdfUploader from './PdfUploader';
import ErrorAlert from './ui/ErrorAlert';
import Tabs from './ui/Tabs';
import PageSelector from './ui/PageSelector';
import styles from './playground.module.css';

const TABS = [
  { id: 'structure', label: 'Structure' },
  { id: 'layout', label: 'Layout' },
];

function StructureLayoutInner({ wasm }: { wasm: WasmModule }) {
  const { doc, pageCount, error, loadFile } = useDocumentLoader(wasm);
  const [activeTab, setActiveTab] = useState('structure');

  // Structure state
  const [structure, setStructure] = useState<StructureBlock[]>([]);
  const [structureError, setStructureError] = useState<string | null>(null);

  // Layout state
  const [page, setPage] = useState(1);
  const [layout, setLayout] = useState<LayoutResult | null>(null);
  const [layoutError, setLayoutError] = useState<string | null>(null);

  const handleFileLoaded = useCallback(
    (data: Uint8Array, name: string) => {
      loadFile(data, name);
      setStructureError(null);
      setLayoutError(null);
      setLayout(null);
      setPage(1);
      try {
        const d = new wasm.WasmDocument(data);
        try {
          setStructure(d.extractStructure());
        } catch (e) {
          setStructure([]);
          setStructureError(e instanceof Error ? e.message : String(e));
        }
        try {
          setLayout(d.analyzeLayout(1));
        } catch (e) {
          setLayout(null);
          setLayoutError(e instanceof Error ? e.message : String(e));
        }
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
      setLayoutError(null);
      try {
        setLayout(doc.analyzeLayout(newPage));
      } catch (e) {
        setLayout(null);
        setLayoutError(e instanceof Error ? e.message : String(e));
      }
    },
    [doc],
  );

  return (
    <div>
      <PdfUploader onFileLoaded={handleFileLoaded} />
      <ErrorAlert error={error} />
      {doc && (
        <>
          <Tabs tabs={TABS} active={activeTab} onChange={setActiveTab} />

          {activeTab === 'structure' && (
            <>
              <ErrorAlert error={structureError} />
              {structure.length === 0 ? (
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
              )}
            </>
          )}

          {activeTab === 'layout' && (
            <>
              <div className={styles.toolbar}>
                <PageSelector page={page} pageCount={pageCount} onChange={handlePageChange} />
              </div>
              <ErrorAlert error={layoutError} />
              {layout && (
                <>
                  <div className={styles.layoutInfo}>
                    <div className={styles.statCard}>
                      Columns <strong>{layout.num_columns}</strong>
                    </div>
                    <div className={styles.statCard}>
                      Header <strong>{layout.has_header ? 'Yes' : 'No'}</strong>
                    </div>
                    <div className={styles.statCard}>
                      Footer <strong>{layout.has_footer ? 'Yes' : 'No'}</strong>
                    </div>
                    <div className={styles.statCard}>
                      Regions <strong>{layout.regions.length}</strong>
                    </div>
                  </div>
                  {layout.num_columns > 1 && (
                    <p style={{ fontStyle: 'italic', color: 'var(--ifm-color-primary)', marginBottom: '1rem' }}>
                      {layout.num_columns}-column layout detected
                    </p>
                  )}
                  {layout.regions.length === 0 ? (
                    <div className={styles.emptyState}>No regions detected on this page.</div>
                  ) : (
                    layout.regions.map((region, i) => (
                      <div key={i} className={styles.regionCard}>
                        <div style={{ marginBottom: '0.25rem' }}>
                          <strong>{region.region_type}</strong>
                          <span style={{ fontSize: '0.8rem', color: 'var(--ifm-color-emphasis-500)', marginLeft: '0.75rem' }}>
                            [{region.bbox.map((v) => v.toFixed(1)).join(', ')}]
                          </span>
                        </div>
                        {region.text && (
                          <div style={{ fontSize: '0.85rem', color: 'var(--ifm-color-emphasis-700)' }}>
                            {region.text.length > 100 ? region.text.slice(0, 100) + '...' : region.text}
                          </div>
                        )}
                      </div>
                    ))
                  )}
                </>
              )}
              {!layout && !layoutError && (
                <div className={styles.emptyState}>Select a page to analyze its layout.</div>
              )}
            </>
          )}
        </>
      )}
    </div>
  );
}

export default function StructureLayout() {
  return <WasmLoader>{(wasm) => <StructureLayoutInner wasm={wasm} />}</WasmLoader>;
}
