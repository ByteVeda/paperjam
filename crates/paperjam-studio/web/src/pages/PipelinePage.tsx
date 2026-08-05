import DropZone from "../components/ui/DropZone";
import Panel from "../components/ui/Panel";
import CodeBlock from "../components/ui/CodeBlock";
import EmptyState from "../components/ui/EmptyState";
import StepPalette from "../components/pipeline/StepPalette";
import StepCard from "../components/pipeline/StepCard";
import PipelineToolbar from "../components/pipeline/PipelineToolbar";
import YamlPreview from "../components/pipeline/YamlPreview";
import { usePipeline } from "../lib/hooks/usePipeline";

interface Props {
  wasmReady: boolean;
}

export default function PipelinePage({ wasmReady }: Props) {
  const pl = usePipeline(wasmReady);

  return (
    <div className="flex flex-col gap-16 page-full-height">
      <DropZone onFileLoaded={(name, bytes) => pl.setInputFile({ name, bytes })} />
      {pl.inputFile && (
        <p className="text-sm text-muted">
          Input: <strong>{pl.inputFile.name}</strong>
        </p>
      )}

      <div className="pipeline-layout">
        <StepPalette onAddStep={pl.addStep} />

        <div className="flex flex-col gap-12 pipeline-center">
          <div className="flex items-center gap-8">
            <label htmlFor="input-pattern">Input pattern:</label>
            <input
              id="input-pattern"
              type="text"
              className="flex-1"
              value={pl.inputPattern}
              onChange={(e) => pl.setInputPattern(e.target.value)}
            />
          </div>

          {pl.steps.length === 0 && (
            <EmptyState icon={"\uD83D\uDD27"} message="Click a step from the palette to add it" />
          )}

          {pl.steps.map((step, idx) => (
            <StepCard
              key={step.id}
              step={step}
              index={idx}
              total={pl.steps.length}
              onRemove={() => pl.removeStep(step.id)}
              onMove={(dir) => pl.moveStep(step.id, dir)}
              onUpdateConfig={(key, value) => pl.updateStepConfig(step.id, key, value)}
            />
          ))}

          <PipelineToolbar
            canRun={wasmReady && pl.steps.length > 0 && pl.inputFile !== null}
            running={pl.running}
            onRun={pl.handleRun}
            onExport={pl.handleExportYaml}
            onImport={pl.handleImportYaml}
            onClear={pl.handleClear}
          />

          {pl.output && (
            <Panel title="Output">
              <div className="panel-body">
                <CodeBlock>{pl.output}</CodeBlock>
              </div>
            </Panel>
          )}
        </div>

        <YamlPreview yaml={pl.pipelineYaml} />
      </div>
    </div>
  );
}
