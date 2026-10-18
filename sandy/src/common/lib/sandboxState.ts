/** Normalizes a sandbox state string from the CLI into a UI state kind. */
export function stateKind(state: string): "running" | "stopped" | "inactive" {
  const normalized = state
    .replace(/^[•○]\s*/, "")
    .trim()
    .toLowerCase();
  if (normalized === "running") return "running";
  if (normalized === "stopped") return "stopped";
  return "inactive";
}

/** The state label without CLI decoration prefixes. */
export function stateLabel(state: string): string {
  return state.replace(/^[•○]\s*/, "").trim();
}
