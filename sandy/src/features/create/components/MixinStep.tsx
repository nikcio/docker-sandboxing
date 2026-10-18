import { useId } from "react";
import type { KitArguments, KitOption } from "../../../lib/kit";
import Icon from "../../../common/components/Icon";
import { KitDetails } from "./KitDetails";
import { KitInputs } from "./KitInputs";

export function MixinStep({
  agent,
  compatible,
  matchingMixins,
  mixinGroups,
  mixins,
  values,
  search,
  onSearch,
  providerFilter,
  onProviderFilter,
  providerLabel,
  onToggleMixin,
  onRemoveMixin,
  onArgumentChange,
  onBack,
  onContinue,
}: {
  agent: KitOption;
  compatible: KitOption[];
  matchingMixins: KitOption[];
  mixinGroups: string[];
  mixins: KitOption[];
  values: KitArguments;
  search: string;
  onSearch: (value: string) => void;
  providerFilter: string;
  onProviderFilter: (value: string) => void;
  providerLabel: (providerId: string) => string;
  onToggleMixin: (kit: KitOption, checked: boolean) => void;
  onRemoveMixin: (kit: KitOption) => void;
  onArgumentChange: (kitId: string, argument: string, value: string) => void;
  onBack: () => void;
  onContinue: () => void;
}) {
  const id = useId();
  const selectedKits = [agent, ...mixins];
  return (
    <form
      className="card catalog-panel"
      onSubmit={(event) => {
        event.preventDefault();
        onContinue();
      }}
    >
      <div className="catalog-section-heading">
        <div>
          <h2>Make it your own</h2>
          <p className="catalog-muted">
            Showing only kit v{agent.schema_version} mixins compatible with {agent.name}, including
            each mixin’s required agent.
          </p>
        </div>
        <span className="catalog-badge">{mixins.length} selected</span>
      </div>
      <div className="catalog-fields">
        <label className="catalog-field" htmlFor={`${id}-mixin-search`}>
          Search mixins
          <input
            id={`${id}-mixin-search`}
            type="search"
            placeholder="Search mixins"
            value={search}
            onChange={(event) => onSearch(event.target.value)}
          />
        </label>
        <label className="catalog-field" htmlFor={`${id}-provider-filter`}>
          Group by provider
          <select
            id={`${id}-provider-filter`}
            value={providerFilter}
            onChange={(event) => onProviderFilter(event.target.value)}
          >
            <option value="">All providers</option>
            {[...new Set(compatible.map((kit) => kit.provider_id))].map((providerId) => (
              <option key={providerId} value={providerId}>
                {providerLabel(providerId)}
              </option>
            ))}
          </select>
        </label>
      </div>
      {mixinGroups.map((providerId) => (
        <fieldset className="catalog-unframed catalog-choice-group" key={providerId}>
          <legend>{providerLabel(providerId)}</legend>
          <div className="catalog-kit-grid">
            {matchingMixins
              .filter((kit) => kit.provider_id === providerId)
              .map((kit) => {
                const selected = mixins.some((item) => item.id === kit.id);
                const duplicateName =
                  !selected && (agent.name === kit.name || mixins.some((item) => item.name === kit.name));
                return (
                  <label className={`catalog-kit ${selected ? "selected" : ""}`} key={kit.id}>
                    <input
                      type="checkbox"
                      checked={selected}
                      disabled={duplicateName}
                      onChange={(event) => onToggleMixin(kit, event.target.checked)}
                    />
                    <span className="catalog-kit-details">
                      <KitDetails kit={kit} />
                      {duplicateName && (
                        <span className="catalog-help">
                          Another kit with this name is already selected.
                        </span>
                      )}
                    </span>
                  </label>
                );
              })}
          </div>
        </fieldset>
      ))}
      {!matchingMixins.length && (
        <div className="catalog-empty-note">
          <p>
            {compatible.length
              ? "No mixins match these filters."
              : "No compatible mixins were discovered. You can continue with the selected agent or workload alone."}
          </p>
          {(search || providerFilter) && (
            <button
              type="button"
              className="ghost"
              onClick={() => {
                onSearch("");
                onProviderFilter("");
              }}
            >
              Clear filters
            </button>
          )}
        </div>
      )}
      {mixins.length > 0 && (
        <div className="catalog-selected">
          <h3>Selected mixins · applied in this order</h3>
          <ol>
            {mixins.map((kit) => (
              <li key={kit.id}>
                <span>{kit.name}</span>
                <button
                  type="button"
                  className="text-button"
                  onClick={() => onRemoveMixin(kit)}
                  aria-label={`Remove ${kit.name}`}
                >
                  Remove
                </button>
              </li>
            ))}
          </ol>
        </div>
      )}
      <KitInputs
        kits={selectedKits}
        values={values}
        disabled={false}
        onChange={onArgumentChange}
      />
      <div className="catalog-actions">
        <button type="button" className="ghost" onClick={onBack}>
          Back
        </button>
        <button type="submit" className="primary">
          Project details <Icon name="arrow" size={16} />
        </button>
      </div>
    </form>
  );
}
