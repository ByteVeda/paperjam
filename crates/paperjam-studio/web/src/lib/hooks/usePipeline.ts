import { useState, useCallback, useMemo } from "react";
import yaml from "js-yaml";
import {
  toYamlObject,
  fromYamlObject,
  runStep,
  type PipelineStep,
} from "../../components/pipeline/pipeline-types";
import { getWasm } from "../wasm-bridge";
import { extensionToFormat } from "../format-utils";

interface InputFile {
  name: string;
  bytes: Uint8Array;
}

export function usePipeline(wasmReady: boolean) {
  const [steps, setSteps] = useState<PipelineStep[]>([]);
  const [inputPattern, setInputPattern] = useState("*.pdf");
  const [inputFile, setInputFile] = useState<InputFile | null>(null);
  const [output, setOutput] = useState("");
  const [running, setRunning] = useState(false);

  const addStep = useCallback((type: string) => {
    setSteps((prev) => [...prev, { id: crypto.randomUUID(), type, config: {} }]);
  }, []);

  const removeStep = useCallback((id: string) => {
    setSteps((prev) => prev.filter((s) => s.id !== id));
  }, []);

  const updateStepConfig = useCallback((id: string, key: string, value: string) => {
    setSteps((prev) =>
      prev.map((s) => (s.id === id ? { ...s, config: { ...s.config, [key]: value } } : s)),
    );
  }, []);

  const moveStep = useCallback((id: string, direction: -1 | 1) => {
    setSteps((prev) => {
      const idx = prev.findIndex((s) => s.id === id);
      if (idx < 0) return prev;
      const newIdx = idx + direction;
      if (newIdx < 0 || newIdx >= prev.length) return prev;
      const next = [...prev];
      [next[idx], next[newIdx]] = [next[newIdx], next[idx]];
      return next;
    });
  }, []);

  const pipelineYaml = useMemo(
    () => yaml.dump(toYamlObject(steps, inputPattern), { flowLevel: -1, lineWidth: 80 }),
    [steps, inputPattern],
  );

  const handleRun = useCallback(() => {
    if (!wasmReady || !inputFile) return;
    setRunning(true);
    setOutput("");

    try {
      const wasm = getWasm();
      if (!wasm) throw new Error("WASM not loaded");

      const format = extensionToFormat(inputFile.name);
      const doc =
        format === "pdf"
          ? new wasm.WasmDocument(inputFile.bytes)
          : wasm.WasmDocument.openWithFormat(inputFile.bytes, format);

      const results: string[] = [];
      for (const step of steps) {
        try {
          results.push(...runStep(doc, step));
        } catch (err) {
          results.push(`[${step.type}] ERROR: ${err}`);
        }
      }

      try { doc.free(); } catch { /* already freed */ }
      setOutput(results.join("\n\n"));
    } catch (err) {
      setOutput(`Pipeline error: ${err}`);
    } finally {
      setRunning(false);
    }
  }, [wasmReady, inputFile, steps]);

  const handleExportYaml = useCallback(() => {
    const blob = new Blob([pipelineYaml], { type: "text/yaml" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "pipeline.yaml";
    a.click();
    URL.revokeObjectURL(url);
  }, [pipelineYaml]);

  const handleImportYaml = useCallback(() => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".yaml,.yml";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      try {
        const text = await file.text();
        const parsed = yaml.load(text) as Record<string, unknown>;
        const imported = fromYamlObject(parsed);
        setInputPattern(imported.inputPattern);
        setSteps(imported.steps);
      } catch (err) {
        console.error("Failed to parse YAML:", err);
      }
    };
    input.click();
  }, []);

  const handleClear = useCallback(() => {
    setSteps([]);
    setInputPattern("*.pdf");
    setOutput("");
    setInputFile(null);
  }, []);

  return {
    steps,
    inputPattern,
    setInputPattern,
    inputFile,
    setInputFile,
    output,
    running,
    pipelineYaml,
    addStep,
    removeStep,
    updateStepConfig,
    moveStep,
    handleRun,
    handleExportYaml,
    handleImportYaml,
    handleClear,
  };
}
