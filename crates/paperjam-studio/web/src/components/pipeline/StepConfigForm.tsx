import type { StepField } from "./pipeline-types";

interface StepConfigFormProps {
  fields: StepField[];
  config: Record<string, string>;
  onUpdate: (key: string, value: string) => void;
}

export default function StepConfigForm({ fields, config, onUpdate }: StepConfigFormProps) {
  if (fields.length === 0) return null;

  return (
    <div className="step-config-fields">
      {fields.map((field) => (
        <div key={field.key} className="step-config-field">
          <label>{field.label}</label>
          {field.options ? (
            <select
              value={config[field.key] ?? ""}
              onChange={(e) => onUpdate(field.key, e.target.value)}
            >
              <option value="">--</option>
              {field.options.map((o) => (
                <option key={o} value={o}>{o}</option>
              ))}
            </select>
          ) : (
            <input
              type="text"
              placeholder={field.placeholder}
              value={config[field.key] ?? ""}
              onChange={(e) => onUpdate(field.key, e.target.value)}
            />
          )}
        </div>
      ))}
    </div>
  );
}
