import { useId } from "react";
import type { KitOption } from "../../../lib/catalog";
import Icon from "../../../common/components/Icon";
import { KitDetails } from "./KitDetails";

export function AgentStep({
  agents,
  agent,
  search,
  onSearch,
  onSelect,
  onBack,
  onContinue,
  onReviewSources,
}: {
  agents: KitOption[];
  agent: KitOption | null;
  search: string;
  onSearch: (value: string) => void;
  onSelect: (kit: KitOption) => void;
  onBack: () => void;
  onContinue: () => void;
  onReviewSources: () => void;
}) {
  const id = useId();
  return (
    <div className="card catalog-panel">
      <h2>Choose an agent or workload</h2>
      <p className="catalog-muted">
        Built-in agent shortcuts use kit v2. A v3 workload starts a v3-only environment. Remote kit
        sources below come from discovered descriptors.
      </p>
      <label className="catalog-field catalog-search" htmlFor={`${id}-agent-search`}>
        Search agents and workloads
        <input
          id={`${id}-agent-search`}
          type="search"
          placeholder="Search catalog"
          value={search}
          onChange={(event) => onSearch(event.target.value)}
        />
      </label>
      {([2, 3] as const).map((version) => {
        const items = agents.filter((kit) => kit.schema_version === version);
        return (
          items.length > 0 && (
            <fieldset className="catalog-unframed catalog-choice-group" key={version}>
              <legend>{version === 2 ? "Kit v2 agents" : "Kit v3 workloads"}</legend>
              <div className="catalog-kit-grid">
                {items.map((kit) => (
                  <label className={`catalog-kit ${agent?.id === kit.id ? "selected" : ""}`} key={kit.id}>
                    <input
                      type="radio"
                      name={`${id}-agent`}
                      value={kit.id}
                      checked={agent?.id === kit.id}
                      onChange={() => onSelect(kit)}
                    />
                    <KitDetails kit={kit} />
                  </label>
                ))}
              </div>
            </fieldset>
          )
        );
      })}
      {!agents.length && (
        <div className="catalog-empty-note">
          <h3>
            {search ? "No matching agents or workloads" : "No agents or workloads were discovered"}
          </h3>
          <p>
            {search
              ? "Try a different search."
              : "Review the discovery notes, then check your enabled providers and references."}
          </p>
          <button type="button" className="ghost" onClick={() => (search ? onSearch("") : onReviewSources())}>
            {search ? "Clear search" : "Review providers"}
          </button>
        </div>
      )}
      <div className="catalog-actions">
        <button type="button" className="ghost" onClick={onBack}>
          Back
        </button>
        <button type="button" className="primary" disabled={!agent} onClick={onContinue}>
          Choose mixins <Icon name="arrow" size={16} />
        </button>
      </div>
    </div>
  );
}
