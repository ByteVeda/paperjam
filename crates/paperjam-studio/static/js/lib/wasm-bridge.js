/**
 * WASM bridge: loads the paperjam-wasm module and exposes helper functions.
 *
 * Expects the WASM build output at /wasm/paperjam_wasm.js (the wasm-pack
 * --target web output).  If the module is not found a warning is logged and
 * all operations return graceful errors so the UI can still render.
 */

import { setWasmReady } from './store.js';

let _wasm = null;
let _initPromise = null;

/** Initialise the WASM module (idempotent, returns a promise). */
export function initWasm() {
  if (_initPromise) return _initPromise;
  _initPromise = _doInit();
  return _initPromise;
}

async function _doInit() {
  try {
    const mod = await import('/wasm/paperjam_wasm.js');
    if (typeof mod.default === 'function') {
      await mod.default();
    }
    _wasm = mod;
    setWasmReady(true);
    console.info('[wasm-bridge] WASM module loaded');
    return true;
  } catch (err) {
    console.warn('[wasm-bridge] Failed to load WASM module:', err.message);
    console.warn('[wasm-bridge] The app will run in demo mode (no document processing).');
    setWasmReady(false);
    return false;
  }
}

/** Returns true when the WASM module has been loaded successfully. */
export function isReady() {
  return _wasm !== null;
}

/**
 * Open a document from bytes.  Returns a WasmDocument handle.
 * @param {Uint8Array} bytes
 * @returns {any} WasmDocument
 */
export function openDocument(bytes) {
  if (!_wasm) throw new Error('WASM not loaded');
  return new _wasm.WasmDocument(bytes);
}

/**
 * Open a document with an explicit format hint.
 * @param {Uint8Array} bytes
 * @param {string} formatStr  e.g. "docx", "xlsx"
 * @returns {any} WasmDocument
 */
export function openDocumentWithFormat(bytes, formatStr) {
  if (!_wasm) throw new Error('WASM not loaded');
  return _wasm.WasmDocument.openWithFormat(bytes, formatStr);
}

/**
 * Convert document bytes from one format to another.
 * @param {Uint8Array} data
 * @param {string} fromFormat
 * @param {string} toFormat
 * @returns {Uint8Array}
 */
export function convertDocument(data, fromFormat, toFormat) {
  if (!_wasm) throw new Error('WASM not loaded');
  return _wasm.convertDocument(data, fromFormat, toFormat);
}

/**
 * Convert a WasmDocument to a different format and return the bytes.
 * @param {any} wasmDoc  WasmDocument handle
 * @param {string} targetFormat  e.g. "pdf", "docx", "md"
 * @returns {Uint8Array}
 */
export function convertTo(wasmDoc, targetFormat) {
  if (!_wasm) throw new Error('WASM not loaded');
  return wasmDoc.convertTo(targetFormat);
}

/**
 * Merge multiple PDFs.
 * @param {Array<Uint8Array>} pdfArrays
 * @returns {Uint8Array}
 */
export function mergePdfs(pdfArrays) {
  if (!_wasm) throw new Error('WASM not loaded');
  return _wasm.mergePdfs(pdfArrays);
}
