import type { DocumentEntry } from "../../lib/store";
import DocumentCard from "./DocumentCard";

interface DocumentListProps {
  documents: Map<string, DocumentEntry>;
  activeId: string | null;
  onSelect: (id: string) => void;
  onRemove: (id: string) => void;
}

export default function DocumentList({
  documents,
  activeId,
  onSelect,
  onRemove,
}: DocumentListProps) {
  if (documents.size === 0) return null;

  return (
    <div className="doc-list">
      {Array.from(documents.values()).map((entry) => (
        <DocumentCard
          key={entry.id}
          name={entry.name}
          format={entry.format}
          size={entry.size}
          active={activeId === entry.id}
          onSelect={() => onSelect(entry.id)}
          onRemove={() => onRemove(entry.id)}
        />
      ))}
    </div>
  );
}
