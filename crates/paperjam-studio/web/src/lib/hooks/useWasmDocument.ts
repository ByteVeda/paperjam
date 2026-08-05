import { useState, useEffect } from "react";
import { getWasm } from "../wasm-bridge";
import type { WasmDocument } from "../wasm-bridge";
import { extensionToFormat } from "../format-utils";

interface UseWasmDocumentResult {
  doc: WasmDocument | null;
  format: string;
  error: string;
  loading: boolean;
}

export function useWasmDocument(
  bytes: Uint8Array | null,
  name: string,
): UseWasmDocumentResult {
  const [doc, setDoc] = useState<WasmDocument | null>(null);
  const [format, setFormat] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!bytes) {
      setDoc(null);
      setFormat("");
      setError("");
      return;
    }

    setLoading(true);
    setError("");

    const fmt = extensionToFormat(name);
    setFormat(fmt);

    const wasm = getWasm();
    if (!wasm) {
      setError("WASM not loaded");
      setLoading(false);
      return;
    }

    try {
      const opened =
        fmt === "pdf"
          ? new wasm.WasmDocument(bytes)
          : wasm.WasmDocument.openWithFormat(bytes, fmt);
      setDoc(opened);
    } catch (err) {
      setError(String(err));
      setDoc(null);
    } finally {
      setLoading(false);
    }

    return () => {
      // Cleanup handled by store when document is removed
    };
  }, [bytes, name]);

  return { doc, format, error, loading };
}
