import { useCallback } from "react";
import DropZone from "../components/ui/DropZone";
import DocumentList from "../components/documents/DocumentList";
import DocumentViewer from "../components/documents/DocumentViewer";
import { useStore } from "../lib/store";
import { getWasm } from "../lib/wasm-bridge";
import type { WasmDocument } from "../lib/wasm-bridge";
import { extensionToFormat } from "../lib/format-utils";

interface Props {
  wasmReady: boolean;
}

export default function DocumentsPage({ wasmReady }: Props) {
  const [state, dispatch] = useStore();

  const handleFileLoaded = useCallback(
    (name: string, bytes: Uint8Array) => {
      const wasm = getWasm();
      const format = extensionToFormat(name);
      let doc: WasmDocument | null = null;

      if (wasm) {
        try {
          doc =
            format === "pdf"
              ? new wasm.WasmDocument(bytes)
              : wasm.WasmDocument.openWithFormat(bytes, format);
        } catch (err) {
          console.error("Failed to open document:", err);
        }
      }

      dispatch({
        type: "ADD_DOCUMENT",
        payload: { id: crypto.randomUUID(), name, bytes, doc, format, size: bytes.length },
      });
    },
    [dispatch],
  );

  const activeDoc = state.activeDocumentId
    ? state.documents.get(state.activeDocumentId) ?? null
    : null;

  return (
    <div className="flex flex-col gap-20">
      <DropZone onFileLoaded={handleFileLoaded} />

      <DocumentList
        documents={state.documents}
        activeId={state.activeDocumentId}
        onSelect={(id) => dispatch({ type: "SET_ACTIVE", payload: id })}
        onRemove={(id) => dispatch({ type: "REMOVE_DOCUMENT", payload: id })}
      />

      {activeDoc && (
        <DocumentViewer entry={activeDoc} wasmReady={wasmReady} />
      )}
    </div>
  );
}
