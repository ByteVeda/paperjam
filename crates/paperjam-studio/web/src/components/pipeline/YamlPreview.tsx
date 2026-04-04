import Panel from "../ui/Panel";
import CodeBlock from "../ui/CodeBlock";

interface YamlPreviewProps {
  yaml: string;
}

export default function YamlPreview({ yaml }: YamlPreviewProps) {
  return (
    <Panel title="YAML Preview" className="yaml-preview-panel">
      <div className="panel-body">
        <CodeBlock>{yaml}</CodeBlock>
      </div>
    </Panel>
  );
}
