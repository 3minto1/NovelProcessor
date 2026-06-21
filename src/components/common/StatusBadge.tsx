type StatusBadgeProps = {
  status: string;
  label?: string;
};

const statusColors: Record<string, string> = {
  pending: "#f39c12",
  running: "#3498db",
  completed: "#27ae60",
  failed: "#e74c3c",
  imported: "#9b59b6",
  valid: "#27ae60",
  invalid: "#e74c3c",
};

export function StatusBadge({ status, label }: StatusBadgeProps) {
  const color = statusColors[status] || "#95a5a6";
  const displayLabel = label || status;

  return (
    <span
      className="status-badge"
      style={{
        backgroundColor: color,
        color: "white",
        padding: "2px 8px",
        borderRadius: "4px",
        fontSize: "12px",
        fontWeight: 500,
      }}
    >
      {displayLabel}
    </span>
  );
}

export function getStatusTone(status: string): "ok" | "warning" | "error" {
  switch (status) {
    case "completed":
    case "valid":
      return "ok";
    case "running":
    case "pending":
      return "warning";
    case "failed":
    case "invalid":
      return "error";
    default:
      return "warning";
  }
}
