// WASM loading singleton
// The WASM module is loaded by dynamically importing paperjam_wasm.js and calling init()
// WASM files (.js + .wasm) live in public/wasm/ and are served at /wasm/ at runtime

export interface WasmModule {
  WasmDocument: typeof WasmDocument;
  convertDocument: (data: Uint8Array, from: string, to: string) => Uint8Array;
  mergePdfs: (arrays: Uint8Array[]) => Uint8Array;
}

export declare class WasmDocument {
  constructor(data: Uint8Array);
  static openWithFormat(data: Uint8Array, format: string): WasmDocument;
  documentFormat(): string;
  pageCount(): number;
  extractAllText(): string;
  extractText(page: number): string;
  extractTables(page: number): unknown;
  extractStructure(): unknown;
  toMarkdown(
    layoutAware?: boolean,
    pageNumbers?: boolean,
    htmlTables?: boolean,
  ): string;
  metadata(): Record<string, unknown>;
  searchText(
    query: string,
    caseSensitive?: boolean,
  ): Array<{ page: number; text: string }>;
  convertTo(format: string): Uint8Array;
  saveBytes(): Uint8Array;
  free(): void;
}

let wasmModule: WasmModule | null = null;
let initPromise: Promise<void> | null = null;

export async function initWasm(): Promise<void> {
  if (wasmModule) return;
  if (initPromise) return initPromise;

  initPromise = (async () => {
    try {
      const mod = await import(
        /* @vite-ignore */ `${import.meta.env.BASE_URL}wasm/paperjam_wasm.js`
      );
      // The default export is the init function that loads the .wasm file
      await mod.default();
      wasmModule = mod as unknown as WasmModule;
    } catch (err) {
      initPromise = null;
      throw err;
    }
  })();

  return initPromise;
}

export function getWasm(): WasmModule | null {
  return wasmModule;
}

export function isWasmReady(): boolean {
  return wasmModule !== null;
}
