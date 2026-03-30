/** Type definitions for the paperjam WASM module. */

export interface WasmModule {
  WasmDocument: new (data: Uint8Array) => WasmDocument;
  mergePdfs(pdfArrays: number[][]): Uint8Array;
}

export interface WasmDocument {
  pageCount(): number;
  pageInfo(page: number): PageInfo;
  extractText(page: number): string;
  extractAllText(): string;
  extractTextLines(page: number): TextLine[];
  extractTables(page: number): TableResult[];
  toMarkdown(
    layoutAware?: boolean,
    includePageNumbers?: boolean,
    htmlTables?: boolean,
  ): string;
  pageToMarkdown(page: number): string;
  metadata(): MetadataResult;
  extractStructure(): StructureBlock[];
  searchText(query: string, caseSensitive?: boolean): SearchMatch[];
  saveBytes(): Uint8Array;
  split(ranges: [number, number][]): Uint8Array[];
  sanitize(
    removeJs?: boolean,
    removeFiles?: boolean,
    removeActions?: boolean,
    removeLinks?: boolean,
  ): SanitizeOutput;
  redactText(
    query: string,
    caseSensitive?: boolean,
    fillColor?: number[],
  ): RedactOutput;
  encrypt(userPassword: string, ownerPassword?: string): Uint8Array;
  analyzeLayout(pageNumber: number): LayoutResult;
}

export interface PageInfo {
  number: number;
  width: number;
  height: number;
  rotation: number;
}

export interface TextSpan {
  text: string;
  x: number;
  y: number;
  width: number;
  font_size: number;
  font_name: string;
}

export interface TextLine {
  text: string;
  bbox: [number, number, number, number];
  spans: TextSpan[];
}

export interface TableResult {
  rows: string[][];
  row_count: number;
  col_count: number;
  bbox: [number, number, number, number];
  strategy: string;
}

export interface MetadataResult {
  title: string | null;
  author: string | null;
  subject: string | null;
  keywords: string | null;
  creator: string | null;
  producer: string | null;
  creation_date: string | null;
  modification_date: string | null;
  pdf_version: string;
  page_count: number;
  is_encrypted: boolean;
}

export interface StructureBlock {
  block_type: string;
  text: string;
  page: number;
  bbox: [number, number, number, number];
  level?: number;
}

export interface SearchMatch {
  page: number;
  line_number: number;
  text: string;
  bbox: [number, number, number, number];
}

export interface SanitizeOutput {
  doc_bytes: Uint8Array;
  result: SanitizeResult;
}

export interface SanitizeResult {
  javascript_removed: number;
  embedded_files_removed: number;
  actions_removed: number;
  links_removed: number;
}

export interface RedactOutput {
  doc_bytes: Uint8Array;
  result: RedactResult;
}

export interface RedactResult {
  pages_modified: number;
  items_redacted: number;
  items: RedactedItem[];
}

export interface RedactedItem {
  page: number;
  text: string;
  rect: [number, number, number, number];
}

export interface LayoutResult {
  num_columns: number;
  regions: LayoutRegion[];
  has_header: boolean;
  has_footer: boolean;
}

export interface LayoutRegion {
  region_type: string;
  bbox: [number, number, number, number];
  text: string;
}
