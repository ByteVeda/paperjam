import CodeBlock from "../../ui/CodeBlock";

interface TableViewerProps {
  content: string;
}

export default function TableViewer({ content }: TableViewerProps) {
  return <CodeBlock>{content}</CodeBlock>;
}
