import CodeBlock from "../../ui/CodeBlock";

interface InfoViewerProps {
  content: string;
}

export default function InfoViewer({ content }: InfoViewerProps) {
  return <CodeBlock>{content}</CodeBlock>;
}
