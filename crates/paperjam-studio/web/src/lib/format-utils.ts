export const SUPPORTED_EXTENSIONS = [
  ".pdf",
  ".docx",
  ".xlsx",
  ".pptx",
  ".html",
  ".htm",
  ".epub",
];

export function getFormatIcon(format: string): string {
  switch (format.toLowerCase()) {
    case "pdf":
      return "\u{1F4D5}";
    case "docx":
      return "\u{1F4D8}";
    case "xlsx":
      return "\u{1F4CA}";
    case "pptx":
      return "\u{1F4CA}";
    case "html":
    case "htm":
      return "\u{1F310}";
    case "epub":
      return "\u{1F4D6}";
    case "markdown":
    case "md":
      return "\u{1F4DD}";
    case "txt":
    case "text":
      return "\u{1F4C4}";
    default:
      return "\u{1F4C1}";
  }
}

export function getFormatLabel(format: string): string {
  switch (format.toLowerCase()) {
    case "pdf":
      return "PDF";
    case "docx":
      return "Word (DOCX)";
    case "xlsx":
      return "Excel (XLSX)";
    case "pptx":
      return "PowerPoint (PPTX)";
    case "html":
    case "htm":
      return "HTML";
    case "epub":
      return "EPUB";
    case "markdown":
    case "md":
      return "Markdown";
    case "txt":
    case "text":
      return "Plain Text";
    default:
      return format.toUpperCase();
  }
}

export function getConversionTargets(sourceFormat: string): string[] {
  switch (sourceFormat.toLowerCase()) {
    case "pdf":
      return ["markdown", "text", "html"];
    case "docx":
      return ["pdf", "markdown", "text", "html"];
    case "xlsx":
      return ["pdf", "markdown", "text", "html"];
    case "pptx":
      return ["pdf", "markdown", "text"];
    case "html":
    case "htm":
      return ["pdf", "markdown", "text"];
    case "epub":
      return ["pdf", "markdown", "text", "html"];
    default:
      return ["pdf", "markdown", "text"];
  }
}

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const size = bytes / Math.pow(1024, i);
  return `${size.toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

export function extensionToFormat(filename: string): string {
  const ext = filename.split(".").pop()?.toLowerCase() ?? "";
  switch (ext) {
    case "pdf":
      return "pdf";
    case "docx":
      return "docx";
    case "xlsx":
      return "xlsx";
    case "pptx":
      return "pptx";
    case "html":
    case "htm":
      return "html";
    case "epub":
      return "epub";
    default:
      return ext;
  }
}
