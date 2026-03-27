import React, { useState, useCallback } from 'react';
import type { WasmModule, SanitizeResult, RedactResult } from '@site/src/types/paperjam';
import { useDocumentLoader } from '@site/src/hooks/useDocumentLoader';
import WasmLoader from './WasmLoader';
import PdfUploader from './PdfUploader';
import ErrorAlert from './ui/ErrorAlert';
import Tabs from './ui/Tabs';
import styles from './playground.module.css';

const TABS = [
  { id: 'sanitize', label: 'Sanitize' },
  { id: 'redact', label: 'Redact' },
  { id: 'encrypt', label: 'Encrypt' },
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

function SecurityInner({ wasm }: { wasm: WasmModule }) {
  const { doc, error, fileName, loadFile } = useDocumentLoader(wasm);
  const [activeTab, setActiveTab] = useState('sanitize');

  // Sanitize state
  const [removeJs, setRemoveJs] = useState(true);
  const [removeFiles, setRemoveFiles] = useState(true);
  const [removeActions, setRemoveActions] = useState(true);
  const [removeLinks, setRemoveLinks] = useState(false);
  const [sanitizeResult, setSanitizeResult] = useState<SanitizeResult | null>(null);
  const [sanitizeBytes, setSanitizeBytes] = useState<Uint8Array | null>(null);
  const [sanitizeError, setSanitizeError] = useState<string | null>(null);

  // Redact state
  const [redactQuery, setRedactQuery] = useState('');
  const [redactCaseSensitive, setRedactCaseSensitive] = useState(false);
  const [redactResult, setRedactResult] = useState<RedactResult | null>(null);
  const [redactBytes, setRedactBytes] = useState<Uint8Array | null>(null);
  const [redactError, setRedactError] = useState<string | null>(null);

  // Encrypt state
  const [userPassword, setUserPassword] = useState('');
  const [ownerPassword, setOwnerPassword] = useState('');
  const [encryptBytes, setEncryptBytes] = useState<Uint8Array | null>(null);
  const [encryptError, setEncryptError] = useState<string | null>(null);

  const handleFileLoaded = useCallback(
    (data: Uint8Array, name: string) => {
      loadFile(data, name);
      setSanitizeResult(null);
      setSanitizeBytes(null);
      setSanitizeError(null);
      setRedactResult(null);
      setRedactBytes(null);
      setRedactError(null);
      setEncryptBytes(null);
      setEncryptError(null);
    },
    [loadFile],
  );

  const handleSanitize = useCallback(() => {
    if (!doc) return;
    setSanitizeError(null);
    try {
      const output = doc.sanitize(removeJs, removeFiles, removeActions, removeLinks);
      setSanitizeResult(output.result);
      setSanitizeBytes(output.doc_bytes);
    } catch (e) {
      setSanitizeError(e instanceof Error ? e.message : String(e));
      setSanitizeResult(null);
      setSanitizeBytes(null);
    }
  }, [doc, removeJs, removeFiles, removeActions, removeLinks]);

  const handleRedact = useCallback(() => {
    if (!doc || !redactQuery.trim()) return;
    setRedactError(null);
    try {
      const output = doc.redactText(redactQuery, redactCaseSensitive);
      setRedactResult(output.result);
      setRedactBytes(output.doc_bytes);
    } catch (e) {
      setRedactError(e instanceof Error ? e.message : String(e));
      setRedactResult(null);
      setRedactBytes(null);
    }
  }, [doc, redactQuery, redactCaseSensitive]);

  const handleEncrypt = useCallback(() => {
    if (!doc || !userPassword.trim()) return;
    setEncryptError(null);
    try {
      const bytes = doc.encrypt(userPassword, ownerPassword || undefined);
      setEncryptBytes(bytes);
    } catch (e) {
      setEncryptError(e instanceof Error ? e.message : String(e));
      setEncryptBytes(null);
    }
  }, [doc, userPassword, ownerPassword]);

  const baseName = fileName ? fileName.replace(/\.pdf$/i, '') : 'document';

  return (
    <div>
      <PdfUploader onFileLoaded={handleFileLoaded} />
      <ErrorAlert error={error} />
      {doc && (
        <>
          <Tabs tabs={TABS} active={activeTab} onChange={setActiveTab} />

          {activeTab === 'sanitize' && (
            <>
              <div className={styles.checkboxGroup} style={{ marginBottom: '1rem' }}>
                <label>
                  <input type="checkbox" checked={removeJs} onChange={(e) => setRemoveJs(e.target.checked)} />
                  Remove JavaScript
                </label>
                <label>
                  <input type="checkbox" checked={removeFiles} onChange={(e) => setRemoveFiles(e.target.checked)} />
                  Remove Embedded Files
                </label>
                <label>
                  <input type="checkbox" checked={removeActions} onChange={(e) => setRemoveActions(e.target.checked)} />
                  Remove Actions
                </label>
                <label>
                  <input type="checkbox" checked={removeLinks} onChange={(e) => setRemoveLinks(e.target.checked)} />
                  Remove Links
                </label>
              </div>
              <div className={styles.toolbar}>
                <button className={`${styles.btn} ${styles.btnPrimary}`} onClick={handleSanitize}>
                  Sanitize PDF
                </button>
              </div>
              <ErrorAlert error={sanitizeError} />
              {sanitizeResult && (
                <>
                  <div className={styles.layoutInfo}>
                    <div className={styles.statCard}>
                      JavaScript <strong>{sanitizeResult.javascript_removed}</strong>
                    </div>
                    <div className={styles.statCard}>
                      Embedded Files <strong>{sanitizeResult.embedded_files_removed}</strong>
                    </div>
                    <div className={styles.statCard}>
                      Actions <strong>{sanitizeResult.actions_removed}</strong>
                    </div>
                    <div className={styles.statCard}>
                      Links <strong>{sanitizeResult.links_removed}</strong>
                    </div>
                  </div>
                  {sanitizeBytes && (
                    <button
                      className={styles.downloadBtn}
                      onClick={() => downloadPdf(sanitizeBytes, `${baseName}_sanitized.pdf`)}
                    >
                      Download Sanitized PDF
                    </button>
                  )}
                </>
              )}
            </>
          )}

          {activeTab === 'redact' && (
            <>
              <div className={styles.toolbar}>
                <input
                  type="text"
                  className={styles.searchInput}
                  placeholder="Text to redact..."
                  value={redactQuery}
                  onChange={(e) => setRedactQuery(e.target.value)}
                  onKeyDown={(e) => { if (e.key === 'Enter') handleRedact(); }}
                  aria-label="Redact query"
                />
                <div className={styles.checkboxGroup}>
                  <label>
                    <input
                      type="checkbox"
                      checked={redactCaseSensitive}
                      onChange={(e) => setRedactCaseSensitive(e.target.checked)}
                    />
                    Case sensitive
                  </label>
                </div>
                <button className={`${styles.btn} ${styles.btnPrimary}`} onClick={handleRedact}>
                  Redact
                </button>
              </div>
              <ErrorAlert error={redactError} />
              {redactResult && (
                <>
                  <div className={styles.layoutInfo}>
                    <div className={styles.statCard}>
                      Pages Modified <strong>{redactResult.pages_modified}</strong>
                    </div>
                    <div className={styles.statCard}>
                      Items Redacted <strong>{redactResult.items_redacted}</strong>
                    </div>
                  </div>
                  {redactResult.items.length > 0 && (
                    <div className={styles.resultPanel} style={{ maxHeight: '300px', marginBottom: '1rem' }}>
                      {redactResult.items.map((item, i) => (
                        <div key={i} style={{ marginBottom: '0.25rem' }}>
                          <strong>Page {item.page}:</strong> {item.text}
                        </div>
                      ))}
                    </div>
                  )}
                  {redactBytes && (
                    <button
                      className={styles.downloadBtn}
                      onClick={() => downloadPdf(redactBytes, `${baseName}_redacted.pdf`)}
                    >
                      Download Redacted PDF
                    </button>
                  )}
                </>
              )}
            </>
          )}

          {activeTab === 'encrypt' && (
            <>
              <div className={styles.toolbar} style={{ flexDirection: 'column', alignItems: 'flex-start', gap: '0.5rem' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                  <label style={{ fontSize: '0.9rem', minWidth: '120px' }}>User Password</label>
                  <input
                    type="password"
                    className={styles.passwordInput}
                    placeholder="Required"
                    value={userPassword}
                    onChange={(e) => setUserPassword(e.target.value)}
                    aria-label="User password"
                  />
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                  <label style={{ fontSize: '0.9rem', minWidth: '120px' }}>Owner Password</label>
                  <input
                    type="password"
                    className={styles.passwordInput}
                    placeholder="Optional"
                    value={ownerPassword}
                    onChange={(e) => setOwnerPassword(e.target.value)}
                    aria-label="Owner password"
                  />
                </div>
              </div>
              <div className={styles.toolbar}>
                <button
                  className={`${styles.btn} ${styles.btnPrimary}`}
                  onClick={handleEncrypt}
                  disabled={!userPassword.trim()}
                >
                  Encrypt PDF
                </button>
              </div>
              <ErrorAlert error={encryptError} />
              {encryptBytes && (
                <button
                  className={styles.downloadBtn}
                  onClick={() => downloadPdf(encryptBytes, `${baseName}_encrypted.pdf`)}
                >
                  Download Encrypted PDF
                </button>
              )}
            </>
          )}
        </>
      )}
    </div>
  );
}

export default function Security() {
  return <WasmLoader>{(wasm) => <SecurityInner wasm={wasm} />}</WasmLoader>;
}
