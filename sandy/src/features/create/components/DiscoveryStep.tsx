import { useId } from "react";
import type { Provider } from "../../../lib/catalog";
import Icon from "../../../common/components/Icon";

/**
 * The discovery trigger below the provider editor: optional Docker Hub query
 * and the "Discover kits" action. The provider editor itself lives in
 * `features/settings`. Unsaved provider changes are persisted by discovery
 * itself, so they don't block the action.
 */
export function DiscoveryStep({
  providers,
  hubQuery,
  onHubQuery,
  busy,
  onDiscover,
  onRefresh,
}: {
  providers: Provider[];
  hubQuery: string;
  onHubQuery: (value: string) => void;
  busy: boolean;
  onDiscover: () => void;
  /** Re-discover while bypassing the immutable-source caches. */
  onRefresh: () => void;
}) {
  const id = useId();
  return (
    <>
      {providers.some((provider) => provider.kind === "dockerhub" && provider.enabled) && (
        <label className="catalog-field catalog-hub-query" htmlFor={`${id}-hub-query`}>
          Docker Hub search (optional)
          <input
            id={`${id}-hub-query`}
            value={hubQuery}
            placeholder="Search kits"
            disabled={busy}
            onChange={(event) => onHubQuery(event.target.value)}
          />
          <span className="catalog-help">
            Narrows discovery results, including Docker Hub. Search runs only when you choose
            Discover kits. Specific repositories use their selected reference.
          </span>
        </label>
      )}
      <div className="catalog-actions">
        <button
          type="button"
          className="primary"
          disabled={busy || !providers.some((provider) => provider.enabled)}
          onClick={onDiscover}
        >
          {busy ? (
            <>
              <span className="spinner" /> Discovering kits…
            </>
          ) : (
            <>
              Discover kits <Icon name="arrow" size={16} />
            </>
          )}
        </button>
        <button type="button" className="ghost" disabled={busy} onClick={onRefresh}>
          <Icon name="refresh" size={16} /> Refresh sources
        </button>
        <span className="catalog-help">
          Only kits with supported metadata will be offered. Unsaved provider changes are saved
          when you discover. GitHub tags and commits are immutable, so discovery reuses cached
          data; Refresh sources fetches everything again.
        </span>
      </div>
    </>
  );
}
