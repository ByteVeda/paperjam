import type { ReactNode } from "react";

interface PanelProps {
  title?: string;
  className?: string;
  children: ReactNode;
}

export default function Panel({ title, className = "", children }: PanelProps) {
  return (
    <div className={`panel${className ? ` ${className}` : ""}`}>
      {title && <div className="panel-header">{title}</div>}
      {children}
    </div>
  );
}
