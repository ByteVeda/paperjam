import { Trash2 } from "lucide-react";
import Button from "../ui/Button";
import { getFormatIcon, getFormatLabel, formatFileSize } from "../../lib/format-utils";

interface DocumentCardProps {
  name: string;
  format: string;
  size: number;
  active: boolean;
  onSelect: () => void;
  onRemove: () => void;
}

export default function DocumentCard({
  name,
  format,
  size,
  active,
  onSelect,
  onRemove,
}: DocumentCardProps) {
  return (
    <div
      className={`card card-clickable${active ? " card-active" : ""}`}
      onClick={onSelect}
    >
      <div className="doc-card">
        <span className="doc-icon">{getFormatIcon(format)}</span>
        <div className="doc-info">
          <div className="doc-name" title={name}>{name}</div>
          <div className="doc-meta">
            {getFormatLabel(format)} &middot; {formatFileSize(size)}
          </div>
        </div>
        <Button
          variant="ghost"
          size="sm"
          icon={<Trash2 size={14} />}
          className="doc-card-remove"
          onClick={(e) => {
            e.stopPropagation();
            onRemove();
          }}
          title="Remove"
        />
      </div>
    </div>
  );
}
