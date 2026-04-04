import { useCallback } from "react";
import { Download } from "lucide-react";
import Button from "../ui/Button";

interface ConversionResultProps {
  result: Uint8Array;
  sourceName: string;
  targetFormat: string;
  preview: string;
}

const EXT_MAP: Record<string, string> = {
  pdf: "pdf",
  markdown: "md",
  text: "txt",
  html: "html",
  docx: "docx",
  xlsx: "xlsx",
  pptx: "pptx",
};

export default function ConversionResult({
  result,
  sourceName,
  targetFormat,
  preview,
}: ConversionResultProps) {
  const handleDownload = useCallback(() => {
    const ext = EXT_MAP[targetFormat] ?? targetFormat;
    const baseName = sourceName.replace(/\.[^.]+$/, "");
    const outName = `${baseName}.${ext}`;

    const blob = new Blob([result as unknown as ArrayBuffer]);
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = outName;
    a.click();
    URL.revokeObjectURL(url);
  }, [result, sourceName, targetFormat]);

  return (
    <div className="flex flex-col gap-12">
      <div>
        <Button icon={<Download size={14} />} onClick={handleDownload}>
          Download
        </Button>
      </div>
      {preview && <div className="viewer-content">{preview}</div>}
    </div>
  );
}
