import { Trash2 } from "lucide-react";
import Button from "../ui/Button";
import StepConfigForm from "./StepConfigForm";
import { getStepType, type PipelineStep } from "./pipeline-types";

interface StepCardProps {
  step: PipelineStep;
  index: number;
  total: number;
  onRemove: () => void;
  onMove: (direction: -1 | 1) => void;
  onUpdateConfig: (key: string, value: string) => void;
}

export default function StepCard({
  step,
  index,
  total,
  onRemove,
  onMove,
  onUpdateConfig,
}: StepCardProps) {
  const info = getStepType(step.type);

  return (
    <div className="pipeline-step">
      <span className="step-number">{index + 1}</span>
      <div className="step-config">
        <div className="step-card-header">
          <strong className="step-card-title">{info?.label ?? step.type}</strong>
          <span className="flex-spacer" />
          <Button size="sm" onClick={() => onMove(-1)} disabled={index === 0} title="Move up">
            {"\u2191"}
          </Button>
          <Button size="sm" onClick={() => onMove(1)} disabled={index === total - 1} title="Move down">
            {"\u2193"}
          </Button>
          <Button size="sm" icon={<Trash2 size={12} />} onClick={onRemove} title="Remove" />
        </div>
        {info && (
          <StepConfigForm
            fields={info.fields}
            config={step.config}
            onUpdate={onUpdateConfig}
          />
        )}
      </div>
    </div>
  );
}
