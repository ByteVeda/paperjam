import { Plus } from "lucide-react";
import Panel from "../ui/Panel";
import { STEP_TYPES } from "./pipeline-types";

interface StepPaletteProps {
  onAddStep: (type: string) => void;
}

export default function StepPalette({ onAddStep }: StepPaletteProps) {
  return (
    <Panel title="Steps" className="step-palette-panel">
      <div className="step-palette-list">
        {STEP_TYPES.map((st) => (
          <div
            key={st.type}
            className="step-palette-item"
            onClick={() => onAddStep(st.type)}
          >
            <Plus size={12} />
            {st.label}
          </div>
        ))}
      </div>
    </Panel>
  );
}
