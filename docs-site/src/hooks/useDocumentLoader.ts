import type { WasmDocument, WasmModule } from '@site/src/types/paperjam';
import { useCallback, useState } from 'react';

interface DocumentLoaderState {
  doc: WasmDocument | null;
  pageCount: number;
  error: string | null;
  fileName: string | null;
  loadFile: (data: Uint8Array, name: string) => void;
}

const MAX_FILE_SIZE = 100 * 1024 * 1024; // 100 MB

export function useDocumentLoader(wasm: WasmModule): DocumentLoaderState {
  const [doc, setDoc] = useState<WasmDocument | null>(null);
  const [pageCount, setPageCount] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);

  const loadFile = useCallback(
    (data: Uint8Array, name: string) => {
      if (data.byteLength > MAX_FILE_SIZE) {
        setError(
          `File too large (${(data.byteLength / 1024 / 1024).toFixed(1)} MB). Maximum is 100 MB.`,
        );
        return;
      }
      try {
        const d = new wasm.WasmDocument(data);
        setDoc(d);
        setPageCount(d.pageCount());
        setFileName(name);
        setError(null);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
        setDoc(null);
        setPageCount(0);
      }
    },
    [wasm],
  );

  return { doc, pageCount, error, fileName, loadFile };
}
