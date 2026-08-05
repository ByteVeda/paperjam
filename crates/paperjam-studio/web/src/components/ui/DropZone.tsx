import { useCallback, useRef, useState, type DragEvent } from "react";
import { Upload } from "lucide-react";
import { SUPPORTED_EXTENSIONS } from "../../lib/format-utils";

interface DropZoneProps {
  onFileLoaded: (name: string, bytes: Uint8Array) => void;
}

export default function DropZone({ onFileLoaded }: DropZoneProps) {
  const [dragOver, setDragOver] = useState(false);
  const [loading, setLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const acceptStr = SUPPORTED_EXTENSIONS.join(",");

  const processFile = useCallback(
    async (file: File) => {
      setLoading(true);
      try {
        const buffer = await file.arrayBuffer();
        onFileLoaded(file.name, new Uint8Array(buffer));
      } finally {
        setLoading(false);
      }
    },
    [onFileLoaded],
  );

  const handleDrop = useCallback(
    (e: DragEvent) => {
      e.preventDefault();
      setDragOver(false);
      const file = e.dataTransfer.files[0];
      if (file) processFile(file);
    },
    [processFile],
  );

  const handleDragOver = useCallback((e: DragEvent) => {
    e.preventDefault();
    setDragOver(true);
  }, []);

  const handleDragLeave = useCallback(() => setDragOver(false), []);
  const handleClick = useCallback(() => inputRef.current?.click(), []);

  const handleInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) processFile(file);
      if (inputRef.current) inputRef.current.value = "";
    },
    [processFile],
  );

  return (
    <div
      className={`drop-zone${dragOver ? " drag-over" : ""}`}
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onClick={handleClick}
    >
      <input
        ref={inputRef}
        type="file"
        accept={acceptStr}
        onChange={handleInputChange}
        className="hidden"
      />
      <div className="drop-zone-icon">
        {loading ? <span className="text-sm">Loading...</span> : <Upload size={32} />}
      </div>
      <div className="drop-zone-label">
        {loading ? "Reading file..." : "Drop a file here or click to browse"}
      </div>
      <div className="drop-zone-hint">
        Supports PDF, DOCX, XLSX, PPTX, HTML, EPUB
      </div>
    </div>
  );
}
