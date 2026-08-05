import { useState, useEffect } from "react";
import type { WasmDocument } from "../wasm-bridge";

interface UseDocumentExtractionResult {
  content: string;
  error: string;
  loading: boolean;
}

function extractContent(
  doc: WasmDocument,
  tab: string,
  name: string,
  format: string,
  size: number,
): string {
  switch (tab) {
    case "info": {
      const meta = doc.metadata();
      let pages: number | string = "N/A";
      try {
        pages = doc.pageCount();
      } catch {
        // not all formats support pageCount
      }
      return JSON.stringify({ name, format, size, pages, metadata: meta }, null, 2);
    }
    case "text":
      return doc.extractAllText();
    case "tables": {
      let pages: number;
      try {
        pages = doc.pageCount();
      } catch {
        pages = 1;
      }
      const allTables: unknown[] = [];
      for (let i = 0; i < pages; i++) {
        try {
          const t = doc.extractTables(i);
          if (t) allTables.push({ page: i, tables: t });
        } catch {
          // skip
        }
      }
      return JSON.stringify(allTables, null, 2);
    }
    case "structure":
      return JSON.stringify(doc.extractStructure(), null, 2);
    case "markdown":
      return doc.toMarkdown(true, true, true);
    default:
      return "";
  }
}

export function useDocumentExtraction(
  doc: WasmDocument | null,
  tab: string,
  meta: { name: string; format: string; size: number },
): UseDocumentExtractionResult {
  const [content, setContent] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!doc) {
      setContent("");
      setError(doc === null ? "Document not loaded in WASM." : "");
      return;
    }

    setLoading(true);
    setError("");

    try {
      const result = extractContent(doc, tab, meta.name, meta.format, meta.size);
      setContent(result);
    } catch (err) {
      setError(String(err));
      setContent("");
    } finally {
      setLoading(false);
    }
  }, [doc, tab, meta.name, meta.format, meta.size]);

  return { content, error, loading };
}
