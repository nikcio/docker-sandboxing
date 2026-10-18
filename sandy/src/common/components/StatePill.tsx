import { stateKind } from "../lib/sandboxState";

export function StatePill({ state }: { state: string }) {
  const label = state.replace(/^[•○]\s*/, "").trim() || "Unknown";
  return (
    <span className={`state-pill ${stateKind(state)}`}>
      <span className="dot" />
      {label.charAt(0).toUpperCase() + label.slice(1)}
    </span>
  );
}
