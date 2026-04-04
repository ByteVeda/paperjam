/**
 * Minimal reactive store for paperjam-studio.
 *
 * Holds application state (loaded documents, active doc, theme) and provides
 * subscribe/dispatch so Preact components can react to changes.
 */

let _state = {
  /** @type {Array<{id: string, name: string, format: string, bytes: Uint8Array, wasmDoc: any}>} */
  documents: [],
  /** @type {string|null} */
  activeDocumentId: null,
  /** @type {'light'|'dark'} */
  theme: window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light',
  /** @type {boolean} */
  wasmReady: false,
};

/** @type {Set<Function>} */
const _listeners = new Set();

/** Return a read-only snapshot of state. */
export function getState() {
  return _state;
}

/** Subscribe to state changes. Returns an unsubscribe function. */
export function subscribe(fn) {
  _listeners.add(fn);
  return () => _listeners.delete(fn);
}

function _notify() {
  for (const fn of _listeners) {
    try { fn(_state); } catch (e) { console.error('[store] listener error', e); }
  }
}

function _setState(partial) {
  _state = { ..._state, ...partial };
  _notify();
}

/** Generate a short random id. */
function _uid() {
  return Math.random().toString(36).slice(2, 10);
}

// ---- Actions ----

/** Mark WASM as ready. */
export function setWasmReady(ready) {
  _setState({ wasmReady: ready });
}

/** Toggle dark/light theme. */
export function toggleTheme() {
  const next = _state.theme === 'dark' ? 'light' : 'dark';
  document.documentElement.setAttribute('data-theme', next);
  _setState({ theme: next });
}

/** Add a document from bytes and its WasmDocument handle. */
export function addDocument(name, format, bytes, wasmDoc) {
  const id = _uid();
  const doc = { id, name, format, bytes, wasmDoc };
  _setState({
    documents: [..._state.documents, doc],
    activeDocumentId: id,
  });
  return id;
}

/** Remove a document by id. */
export function removeDocument(id) {
  const documents = _state.documents.filter(d => d.id !== id);
  const activeDocumentId = _state.activeDocumentId === id
    ? (documents.length > 0 ? documents[0].id : null)
    : _state.activeDocumentId;
  _setState({ documents, activeDocumentId });
}

/** Set the active document. */
export function setActiveDocument(id) {
  _setState({ activeDocumentId: id });
}

/** Get the active document entry, or null. */
export function getActiveDocument() {
  if (!_state.activeDocumentId) return null;
  return _state.documents.find(d => d.id === _state.activeDocumentId) || null;
}

// Apply initial theme attribute
document.documentElement.setAttribute('data-theme', _state.theme);
