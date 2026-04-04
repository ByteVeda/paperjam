export interface PipelineStep {
  id: string;
  type: string;
  config: Record<string, string>;
}

export interface StepField {
  key: string;
  label: string;
  placeholder: string;
  options?: string[];
}

export interface StepType {
  type: string;
  label: string;
  fields: StepField[];
}

export const STEP_TYPES: StepType[] = [
  { type: "extract_text", label: "Extract Text", fields: [] },
  {
    type: "extract_tables",
    label: "Extract Tables",
    fields: [{ key: "page", label: "Page (blank = all)", placeholder: "0" }],
  },
  {
    type: "convert",
    label: "Convert",
    fields: [
      {
        key: "format",
        label: "Target Format",
        placeholder: "markdown",
        options: ["pdf", "markdown", "text", "html"],
      },
    ],
  },
  {
    type: "to_markdown",
    label: "To Markdown",
    fields: [
      { key: "layout_aware", label: "Layout Aware", placeholder: "true" },
      { key: "page_numbers", label: "Page Numbers", placeholder: "true" },
      { key: "html_tables", label: "HTML Tables", placeholder: "true" },
    ],
  },
  {
    type: "redact",
    label: "Redact",
    fields: [
      { key: "patterns", label: "Patterns (comma-separated)", placeholder: "SSN,email" },
    ],
  },
  {
    type: "watermark",
    label: "Watermark",
    fields: [
      { key: "text", label: "Watermark Text", placeholder: "CONFIDENTIAL" },
    ],
  },
  { type: "sanitize", label: "Sanitize", fields: [] },
  {
    type: "encrypt",
    label: "Encrypt",
    fields: [{ key: "password", label: "Password", placeholder: "" }],
  },
  {
    type: "save",
    label: "Save",
    fields: [
      { key: "path", label: "Output Path", placeholder: "output/{stem}.pdf" },
    ],
  },
];

export function getStepType(type: string): StepType | undefined {
  return STEP_TYPES.find((s) => s.type === type);
}

/**
 * Serialize pipeline steps to the Rust PipelineDefinition-compatible format.
 * Step config fields are at the SAME level as `type` (not nested in a `config` object).
 */
export function toYamlObject(
  steps: PipelineStep[],
  inputPattern: string,
): Record<string, unknown> {
  return {
    name: "studio-pipeline",
    input: inputPattern,
    parallel: false,
    on_error: "fail_fast",
    steps: steps.map((s) => {
      const entry: Record<string, unknown> = { type: s.type };
      for (const [k, v] of Object.entries(s.config)) {
        if (v) entry[k] = v;
      }
      return entry;
    }),
  };
}

/**
 * Parse a Rust PipelineDefinition-compatible YAML object back into steps.
 */
export function fromYamlObject(parsed: Record<string, unknown>): {
  inputPattern: string;
  steps: PipelineStep[];
} {
  const inputPattern = (parsed.input as string) ?? "*.pdf";
  const rawSteps = parsed.steps as Array<Record<string, string>> | undefined;

  const steps: PipelineStep[] = (rawSteps ?? []).map((s) => {
    const { type, ...rest } = s;
    return {
      id: crypto.randomUUID(),
      type: type || "extract_text",
      config: rest,
    };
  });

  return { inputPattern, steps };
}

/**
 * Execute a single pipeline step against a WasmDocument, returning log lines.
 */
export function runStep(
  doc: { extractAllText: () => string; extractTables: (page: number) => unknown; convertTo: (fmt: string) => Uint8Array; toMarkdown: (l: boolean, p: boolean, h: boolean) => string },
  step: PipelineStep,
): string[] {
  const results: string[] = [];
  switch (step.type) {
    case "extract_text": {
      const text = doc.extractAllText();
      results.push(`[extract_text] ${text.length} chars extracted`);
      results.push(text.slice(0, 500));
      break;
    }
    case "extract_tables": {
      const page = step.config.page ? parseInt(step.config.page, 10) : 0;
      const tables = doc.extractTables(page);
      results.push(`[extract_tables] page ${page}`);
      results.push(JSON.stringify(tables, null, 2).slice(0, 500));
      break;
    }
    case "convert": {
      const fmt = step.config.format || "markdown";
      const outBytes = doc.convertTo(fmt);
      results.push(`[convert -> ${fmt}] ${outBytes.length} bytes`);
      if (["markdown", "text", "html"].includes(fmt)) {
        results.push(new TextDecoder().decode(outBytes).slice(0, 500));
      }
      break;
    }
    case "to_markdown": {
      const layoutAware = step.config.layout_aware !== "false";
      const pageNumbers = step.config.page_numbers !== "false";
      const htmlTables = step.config.html_tables !== "false";
      const md = doc.toMarkdown(layoutAware, pageNumbers, htmlTables);
      results.push(`[to_markdown] ${md.length} chars`);
      results.push(md.slice(0, 500));
      break;
    }
    default:
      results.push(`[${step.type}] executed (no preview available)`);
  }
  return results;
}
