import { useState, useCallback } from "react";
import DropZone from "../components/ui/DropZone";
import Panel from "../components/ui/Panel";
import Button from "../components/ui/Button";
import FormatSelector from "../components/convert/FormatSelector";
import ConversionResult from "../components/convert/ConversionResult";
import { getWasm } from "../lib/wasm-bridge";
import type { WasmDocument } from "../lib/wasm-bridge";
import {
  extensionToFormat,
  getConversionTargets,
  getFormatLabel,
  getFormatIcon,
  formatFileSize,
} from "../lib/format-utils";

interface Props {
  wasmReady: boolean;
}

interface SourceFile {
  name: string;
  bytes: Uint8Array;
  format: string;
  doc: WasmDocument | null;
}

export default function ConvertPage({ wasmReady }: Props) {
  const [source, setSource] = useState<SourceFile | null>(null);
  const [targetFormat, setTargetFormat] = useState("");
  const [result, setResult] = useState<Uint8Array | null>(null);
  const [preview, setPreview] = useState("");
  const [error, setError] = useState("");
  const [converting, setConverting] = useState(false);

  const handleFileLoaded = useCallback((name: string, bytes: Uint8Array) => {
    setResult(null);
    setPreview("");
    setError("");

    const format = extensionToFormat(name);
    const wasm = getWasm();
    let doc: WasmDocument | null = null;

    if (wasm) {
      try {
        doc =
          format === "pdf"
            ? new wasm.WasmDocument(bytes)
            : wasm.WasmDocument.openWithFormat(bytes, format);
      } catch (err) {
        console.error("Failed to open:", err);
      }
    }

    setSource({ name, bytes, format, doc });
    const targets = getConversionTargets(format);
    setTargetFormat(targets[0] ?? "");
  }, []);

  const handleConvert = useCallback(() => {
    if (!source || !targetFormat) return;
    setConverting(true);
    setError("");
    setResult(null);
    setPreview("");

    try {
      const wasm = getWasm();
      if (!wasm) throw new Error("WASM not loaded");

      const outputBytes = source.doc
        ? source.doc.convertTo(targetFormat)
        : wasm.convertDocument(source.bytes, source.format, targetFormat);

      setResult(outputBytes);

      if (["markdown", "text", "html", "md", "txt"].includes(targetFormat)) {
        setPreview(new TextDecoder().decode(outputBytes).slice(0, 5000));
      } else {
        setPreview(
          `Converted to ${getFormatLabel(targetFormat)} (${formatFileSize(outputBytes.length)})`,
        );
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setConverting(false);
    }
  }, [source, targetFormat]);

  const targets = source ? getConversionTargets(source.format) : [];

  return (
    <div className="flex flex-col gap-20">
      <DropZone onFileLoaded={handleFileLoaded} />

      {source && (
        <Panel
          title={`${getFormatIcon(source.format)} ${source.name} \u2014 ${getFormatLabel(source.format)}`}
        >
          <div className="panel-body flex flex-col gap-16">
            <div className="flex items-center gap-12">
              <FormatSelector
                targets={targets}
                value={targetFormat}
                onChange={(fmt) => {
                  setTargetFormat(fmt);
                  setResult(null);
                  setPreview("");
                }}
              />
              <Button
                variant="primary"
                onClick={handleConvert}
                disabled={!wasmReady || converting || !targetFormat}
              >
                {converting ? "Converting..." : "Convert"}
              </Button>
            </div>

            {error && <p className="text-error">{error}</p>}

            {result && (
              <ConversionResult
                result={result}
                sourceName={source.name}
                targetFormat={targetFormat}
                preview={preview}
              />
            )}
          </div>
        </Panel>
      )}
    </div>
  );
}
