interface TextViewerProps {
  content: string;
}

export default function TextViewer({ content }: TextViewerProps) {
  return (
    <div className="viewer-content">{content || "No content extracted"}</div>
  );
}
