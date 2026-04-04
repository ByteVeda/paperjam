import { Play, Download, Upload, RotateCcw } from "lucide-react";
import Button from "../ui/Button";

interface PipelineToolbarProps {
  canRun: boolean;
  running: boolean;
  onRun: () => void;
  onExport: () => void;
  onImport: () => void;
  onClear: () => void;
}

export default function PipelineToolbar({
  canRun,
  running,
  onRun,
  onExport,
  onImport,
  onClear,
}: PipelineToolbarProps) {
  return (
    <div className="flex items-center gap-8">
      <Button
        variant="primary"
        icon={<Play size={14} />}
        disabled={!canRun || running}
        onClick={onRun}
      >
        {running ? "Running..." : "Run"}
      </Button>
      <Button icon={<Download size={14} />} onClick={onExport}>
        Export YAML
      </Button>
      <Button icon={<Upload size={14} />} onClick={onImport}>
        Import YAML
      </Button>
      <Button icon={<RotateCcw size={14} />} onClick={onClear}>
        Clear
      </Button>
    </div>
  );
}
