interface MarkdownViewerProps {
  content: string;
}

export default function MarkdownViewer({ content }: MarkdownViewerProps) {
  return (
    <div className="viewer-content">{content || "No content extracted"}</div>
  );
}
