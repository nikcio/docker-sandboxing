import { useEffect, useId, useRef, useState } from "react";
import type { Provider } from "../../../lib/catalog";
import { providerVersions, isHubSearchProvider } from "../../../lib/catalog";
import Icon from "../../../common/components/Icon";
import { normalizeRepository, repositoryIsValid } from "../providerRepository";

type VersionState = {
  values: string[];
  loading: boolean;
  error: string | null;
  fetchedAt?: string;
};

export function ProviderEditor({
  provider,
  disabled,
  onUpdate,
  onRemove,
}: {
  provider: Provider;
  disabled: boolean;
  onUpdate: (provider: Provider) => void;
  onRemove: () => void;
}) {
  const id = useId();
  const cache = useRef(new Map<string, VersionState>());
  const [versions, setVersions] = useState<VersionState>({
    values: [],
    loading: false,
    error: null,
  });
  const [manual, setManual] = useState(false);
  const repositoryKey = `${provider.kind}:${normalizeRepository(provider.repository, provider.kind)}`;
  const currentKey = useRef(repositoryKey);
  currentKey.current = repositoryKey;
  const inFlight = useRef(false);
  // Latest values for the async version fetch, without making it a render effect.
  const providerRef = useRef(provider);
  providerRef.current = provider;
  const onUpdateRef = useRef(onUpdate);
  onUpdateRef.current = onUpdate;
  const manualRef = useRef(manual);
  manualRef.current = manual;
  const searchProvider = isHubSearchProvider(provider);

  useEffect(() => {
    setVersions(
      cache.current.get(repositoryKey) ?? {
        values: [],
        loading: false,
        error: null,
      },
    );
    setManual(false);
    // Auto-fetch the version list for enabled, valid providers with no chosen
    // reference so the latest release can be selected without extra clicks.
    if (
      !disabled &&
      provider.enabled &&
      !isHubSearchProvider(provider) &&
      repositoryIsValid(provider.repository, provider.kind) &&
      !provider.reference.trim() &&
      !inFlight.current
    ) {
      void loadVersions();
    }
  }, [repositoryKey, provider.id, disabled]);

  const loadVersions = async () => {
    if (inFlight.current) return;
    inFlight.current = true;
    const key = repositoryKey;
    setVersions((previous) => ({ ...previous, loading: true, error: null }));
    try {
      const result = await providerVersions({
        ...provider,
        repository: normalizeRepository(provider.repository, provider.kind),
      });
      const values = [
        ...new Set(result.filter((value) => typeof value === "string" && value.trim())),
      ];
      const next: VersionState = {
        values,
        loading: false,
        error: values.length
          ? null
          : "No versions were returned. Enter a reference manually or try again.",
        fetchedAt: new Date().toLocaleTimeString(),
      };
      cache.current.set(key, next);
      if (currentKey.current === key) {
        setVersions(next);
        if (!values.length) setManual(true);
        // With nothing chosen yet, default to the newest release. Discovery
        // treats an empty reference as "latest", so only fill a concrete
        // selection while the user has not started typing a manual reference.
        if (values.length && !manualRef.current && !providerRef.current.reference.trim()) {
          onUpdateRef.current({ ...providerRef.current, reference: values[0] });
        }
      }
    } catch (failure) {
      if (currentKey.current === key) {
        setVersions((previous) => ({
          ...previous,
          loading: false,
          error: String(failure),
        }));
        setManual(true);
      }
    } finally {
      inFlight.current = false;
    }
  };

  const hasCurrentReference = versions.values.includes(provider.reference);

  return (
    <fieldset className="catalog-provider" disabled={disabled}>
      <legend>{provider.label || provider.repository || "Provider"}</legend>
      <div className="catalog-provider-heading">
        <label className="catalog-check">
          <input
            type="checkbox"
            checked={provider.enabled}
            onChange={(event) => onUpdate({ ...provider, enabled: event.target.checked })}
          />
          Enabled for discovery
        </label>
        <span className="catalog-badge">{provider.kind === "github" ? "GitHub" : "Docker Hub"}</span>
        <button
          type="button"
          className="text-button"
          onClick={onRemove}
          aria-label={`Remove ${provider.label || provider.repository}`}
        >
          Remove
        </button>
      </div>
      <div className="catalog-fields">
        <label className="catalog-field" htmlFor={`${id}-label`}>
          Display name
          <input
            id={`${id}-label`}
            required
            value={provider.label}
            placeholder="Provider name"
            onChange={(event) => onUpdate({ ...provider, label: event.target.value })}
          />
        </label>
        <label className="catalog-field" htmlFor={`${id}-repository`}>
          {provider.kind === "github"
            ? "GitHub URL or owner/repository"
            : "Docker Hub search URL or repository"}
          <input
            id={`${id}-repository`}
            required={provider.kind === "github"}
            value={provider.repository}
            placeholder={
              provider.kind === "github" ? "owner/repository" : "Search URL or namespace/repository"
            }
            aria-describedby={`${id}-repository-help`}
            onChange={(event) => onUpdate({ ...provider, repository: event.target.value })}
          />
          <span className="catalog-help" id={`${id}-repository-help`}>
            {provider.kind === "github"
              ? "Use the repository root; choose its revision below."
              : "Leave blank to search Docker Hub kits, or enter namespace/repository to choose its tags."}
          </span>
        </label>
      </div>
      {!searchProvider && (
        <>
          <div className="catalog-fields catalog-version-fields">
            <label className="catalog-field" htmlFor={`${id}-version`}>
              Version / reference
              <select
                id={`${id}-version`}
                value={hasCurrentReference ? provider.reference : ""}
                disabled={!versions.values.length || versions.loading || manual}
                onChange={(event) => onUpdate({ ...provider, reference: event.target.value })}
              >
                <option value="" disabled>
                  {provider.reference
                    ? `Current: ${provider.reference}`
                    : versions.loading
                      ? "Fetching versions…"
                      : "Latest release"}
                </option>
                {versions.values.map((reference) => (
                  <option key={reference} value={reference}>
                    {reference}
                  </option>
                ))}
              </select>
            </label>
            <div className="catalog-version-actions">
              <button
                type="button"
                className="ghost"
                disabled={versions.loading || !repositoryIsValid(provider.repository, provider.kind)}
                onClick={loadVersions}
              >
                {versions.loading ? (
                  <span className="spinner" />
                ) : (
                  <Icon name="refresh" size={16} />
                )}
                {versions.loading
                  ? "Fetching versions…"
                  : versions.fetchedAt
                    ? "Refresh versions"
                    : "Fetch versions"}
              </button>
              <label className="catalog-check">
                <input type="checkbox" checked={manual} onChange={(event) => setManual(event.target.checked)} />
                Enter reference manually
              </label>
            </div>
          </div>
          {(manual || (!versions.values.length && !versions.loading)) && (
            <label className="catalog-field catalog-reference" htmlFor={`${id}-reference`}>
              {provider.kind === "github" ? "Tag, branch, or commit" : "Tag or digest"}
              <input
                id={`${id}-reference`}
                value={provider.reference}
                required={provider.enabled}
                placeholder="Reference"
                onChange={(event) => onUpdate({ ...provider, reference: event.target.value })}
              />
            </label>
          )}
          <p className="catalog-help">
            Selected reference:{" "}
            <code>
              {provider.reference || (versions.loading ? "Fetching latest…" : "Latest release")}
            </code>
            {versions.fetchedAt
              ? ` · Version list cached in this view at ${versions.fetchedAt}`
              : versions.loading
                ? " · Loading remote versions…"
                : " · Remote versions have not been loaded for this repository."}
          </p>
          {versions.error && (
            <div className="catalog-inline-error" role="alert">
              <strong>Couldn’t load remote versions.</strong> {versions.error}
              <span>
                {" "}
                Your selected reference is kept. You can enter one manually or fetch again.
              </span>
            </div>
          )}
        </>
      )}
      {searchProvider && (
        <>
          <label className="catalog-field catalog-reference" htmlFor={`${id}-tag-filter`}>
            Repository tag filter (optional)
            <input
              id={`${id}-tag-filter`}
              value={provider.reference}
              placeholder="Newest kit tag per repository"
              onChange={(event) => onUpdate({ ...provider, reference: event.target.value })}
            />
          </label>
          <p className="catalog-help">
            Docker Hub search has no global version. Leave the filter blank to discover each
            repository’s newest sandbox-kit tag. Add a specific repository provider to fetch and
            choose its versions before discovery.
          </p>
        </>
      )}
    </fieldset>
  );
}
