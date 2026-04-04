import { getFormatLabel } from "../../lib/format-utils";

interface FormatSelectorProps {
  targets: string[];
  value: string;
  onChange: (format: string) => void;
}

export default function FormatSelector({ targets, value, onChange }: FormatSelectorProps) {
  return (
    <div className="flex items-center gap-12">
      <label htmlFor="target-format">Convert to:</label>
      <select
        id="target-format"
        className="format-select"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      >
        {targets.map((t) => (
          <option key={t} value={t}>
            {getFormatLabel(t)}
          </option>
        ))}
      </select>
    </div>
  );
}
