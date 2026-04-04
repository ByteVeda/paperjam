import type { ReactNode } from "react";

interface EmptyStateProps {
  icon: ReactNode;
  message: string;
}

export default function EmptyState({ icon, message }: EmptyStateProps) {
  return (
    <div className="empty-state">
      <div className="empty-state-icon">{icon}</div>
      <div className="empty-state-label">{message}</div>
    </div>
  );
}
