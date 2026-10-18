import type { KitOption } from "../../../lib/kit";

export function KitDetails({ kit }: { kit: KitOption }) {
  return (
    <span className="catalog-kit-details">
      <span className="catalog-kit-title">
        {kit.name}
        <span className="catalog-badge">
          Kit v{kit.schema_version} ·{" "}
          {kit.kind === "workload" ? "Workload" : kit.kind === "agent" ? "Agent" : "Mixin"}
        </span>
      </span>
      <span className="catalog-muted">{kit.description || "No description provided."}</span>
      <code className="catalog-source">{kit.source}</code>
      {kit.required_agent && <span className="catalog-help">Requires agent: {kit.required_agent}</span>}
    </span>
  );
}
