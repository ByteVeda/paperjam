import { useState } from "react";
import type { DocumentEntry } from "../../lib/store";
import { useDocumentExtraction } from "../../lib/hooks/useDocumentExtraction";
import Tabs from "../ui/Tabs";
import InfoViewer from "./viewers/InfoViewer";
import TextViewer from "./viewers/TextViewer";
import TableViewer from "./viewers/TableViewer";
import StructureViewer from "./viewers/StructureViewer";
import MarkdownViewer from "./viewers/MarkdownViewer";

const VIEWER_TABS = [
  { id: "info", label: "Info" },
  { id: "text", label: "Text" },
  { id: "tables", label: "Tables" },
  { id: "structure", label: "Structure" },
  { id: "markdown", label: "Markdown" },
];

interface DocumentViewerProps {
  entry: DocumentEntry;
  wasmReady: boolean;
}

function ViewerContent({ entry, tab }: { entry: DocumentEntry; tab: string }) {
  const meta = { name: entry.name, format: entry.format, size: entry.size };
  const { content, error } = useDocumentExtraction(entry.doc, tab, meta);

  if (error) return <p className="text-error">{error}</p>;

  switch (tab) {
    case "info":
      return <InfoViewer content={content} />;
    case "text":
      return <TextViewer content={content} />;
    case "tables":
      return <TableViewer content={content} />;
    case "structure":
      return <StructureViewer content={content} />;
    case "markdown":
      return <MarkdownViewer content={content} />;
    default:
      return null;
  }
}

export default function DocumentViewer({ entry, wasmReady }: DocumentViewerProps) {
  const [activeTab, setActiveTab] = useState("info");

  return (
    <div className="panel">
      <Tabs tabs={VIEWER_TABS} activeTab={activeTab} onChange={setActiveTab} />
      <div className="panel-body">
        {!wasmReady ? (
          <p className="text-muted">WASM not loaded yet.</p>
        ) : (
          <ViewerContent entry={entry} tab={activeTab} />
        )}
      </div>
    </div>
  );
}
