import CodeBlock from "../../ui/CodeBlock";

interface StructureViewerProps {
  content: string;
}

export default function StructureViewer({ content }: StructureViewerProps) {
  return <CodeBlock>{content}</CodeBlock>;
}
