import { useEffect, useId, useRef, useState } from "react";
import type { FormEvent } from "react";
import {
  getProviders,
  isHubSearchProvider,
  providerVersions,
  saveProviders,
} from "./catalog-api";
import type { Provider } from "./catalog-api";
import Icon from "./Icon";

export interface ProvidersProps {
  /** Called with the persisted configuration after loading or saving. */
  onChange?: (providers: Provider[]) => void;
  onDirtyChange?: (dirty: boolean) => void;
  disabled?: boolean;
}

type VersionState = {
  values: string[];
  loading: boolean;
  error: string | null;
  fetchedAt?: string;
};

function normalizeRepository(value: string, kind: Provider["kind"]): string {
  let repository = value.trim().replace(/\/+$/, "");
  if (kind === "github") {
    repository = repository
      .replace(/^https?:\/\/github\.com\//i, "")
      .replace(/\.git$/, "");
  } else {
    repository = repository
      .replace(/^https?:\/\/hub\.docker\.com\/r\//i, "")
      .replace(/^docker\.io\//i, "");
  }
  return repository;
}

function repositoryIsValid(value: string, kind: Provider["kind"]) {
  const repository = normalizeRepository(value, kind);
  if (isHubSearchProvider({ repository, kind } as Provider)) return true;
  return kind === "github"
    ? /^[\w.-]+\/[\w.-]+$/.test(repository)
    : /^[a-z0-9][a-z0-9_.-]*\/[a-z0-9][a-z0-9_.-]*$/.test(repository);
}

function ProviderEditor({
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
  }, [repositoryKey]);

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
        ...new Set(
          result.filter((value) => typeof value === "string" && value.trim()),
        ),
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
            onChange={(event) =>
              onUpdate({ ...provider, enabled: event.target.checked })
            }
          />
          Enabled for discovery
        </label>
        <span className="catalog-badge">
          {provider.kind === "github" ? "GitHub" : "Docker Hub"}
        </span>
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
            onChange={(event) =>
              onUpdate({ ...provider, label: event.target.value })
            }
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
              provider.kind === "github"
                ? "owner/repository"
                : "Search URL or namespace/repository"
            }
            aria-describedby={`${id}-repository-help`}
            onChange={(event) =>
              onUpdate({ ...provider, repository: event.target.value })
            }
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
                onChange={(event) =>
                  onUpdate({ ...provider, reference: event.target.value })
                }
              >
                <option value="" disabled>
                  {provider.reference
                    ? `Current: ${provider.reference}`
                    : "Choose a version"}
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
                disabled={
                  versions.loading ||
                  !repositoryIsValid(provider.repository, provider.kind)
                }
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
                <input
                  type="checkbox"
                  checked={manual}
                  onChange={(event) => setManual(event.target.checked)}
                />
                Enter reference manually
              </label>
            </div>
          </div>
          {(manual || !versions.values.length) && (
            <label
              className="catalog-field catalog-reference"
              htmlFor={`${id}-reference`}
            >
              {provider.kind === "github"
                ? "Tag, branch, or commit"
                : "Tag or digest"}
              <input
                id={`${id}-reference`}
                value={provider.reference}
                required={provider.enabled}
                placeholder="Reference"
                onChange={(event) =>
                  onUpdate({ ...provider, reference: event.target.value })
                }
              />
            </label>
          )}
          <p className="catalog-help">
            Selected reference: <code>{provider.reference || "None"}</code>
            {versions.fetchedAt
              ? ` · Version list cached in this view at ${versions.fetchedAt}`
              : " · Remote versions have not been loaded for this repository."}
          </p>
          {versions.error && (
            <div className="catalog-inline-error" role="alert">
              <strong>Couldn’t load remote versions.</strong> {versions.error}
              <span>
                {" "}Your selected reference is kept. You can enter one manually
                or fetch again.
              </span>
            </div>
          )}
        </>
      )}
      {searchProvider && (
        <>
          <label
            className="catalog-field catalog-reference"
            htmlFor={`${id}-tag-filter`}
          >
            Repository tag filter (optional)
            <input
              id={`${id}-tag-filter`}
              value={provider.reference}
              placeholder="Newest kit tag per repository"
              onChange={(event) =>
                onUpdate({ ...provider, reference: event.target.value })
              }
            />
          </label>
          <p className="catalog-help">
            Docker Hub search has no global version. Leave the filter blank to
            discover each repository’s newest sandbox-kit tag. Add a specific
            repository provider to fetch and choose its versions before discovery.
          </p>
        </>
      )}
    </fieldset>
  );
}

export default function Providers({
  onChange,
  onDirtyChange,
  disabled = false,
}: ProvidersProps) {
  const [providers, setProviders] = useState<Provider[] | null>(null);
  const [saved, setSaved] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState("");
  const [newKind, setNewKind] = useState<Provider["kind"]>("github");
  const [reload, setReload] = useState(0);
  const callbacks = useRef({ onChange, onDirtyChange });
  callbacks.current = { onChange, onDirtyChange };
  const lock = useRef(false);
  const id = useId();
  const dirty = providers !== null && JSON.stringify(providers) !== saved;
  const incomplete =
    providers?.some(
      (provider) =>
        provider.enabled &&
        ((!isHubSearchProvider(provider) && !provider.reference.trim()) ||
          !repositoryIsValid(provider.repository, provider.kind)),
    ) ?? false;

  useEffect(() => {
    let active = true;
    setLoading(true);
    setError(null);
    getProviders()
      .then((result) => {
        if (!active) return;
        setProviders(result);
        setSaved(JSON.stringify(result));
        callbacks.current.onChange?.(result);
      })
      .catch((failure) => {
        if (active) setError(String(failure));
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [reload]);

  useEffect(() => {
    callbacks.current.onDirtyChange?.(
      loading || providers === null || dirty || incomplete,
    );
  }, [dirty, loading, providers, incomplete]);

  const update = (next: Provider[]) => {
    setProviders(next);
    setNotice("");
    setError(null);
  };

  const save = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (lock.current || !providers) return;
    if (
      providers.some(
        (provider) =>
          !provider.label.trim() ||
          (!isHubSearchProvider(provider) &&
            provider.enabled &&
            !provider.reference.trim()) ||
          !repositoryIsValid(provider.repository, provider.kind),
      )
    ) {
      setError(
        "Give each provider a name, a valid repository, and a reference before saving.",
      );
      return;
    }
    lock.current = true;
    setSaving(true);
    setError(null);
    setNotice("");
    try {
      const result = await saveProviders(
        providers.map((provider) => ({
          ...provider,
          label: provider.label.trim(),
          repository: normalizeRepository(provider.repository, provider.kind),
          reference: provider.reference.trim(),
        })),
      );
      setProviders(result);
      setSaved(JSON.stringify(result));
      setNotice("Provider configuration saved to disk.");
      callbacks.current.onChange?.(result);
    } catch (failure) {
      setError(String(failure));
    } finally {
      setSaving(false);
      lock.current = false;
    }
  };

  return (
    <section
      className="catalog-providers"
      aria-labelledby={`${id}-title`}
      aria-busy={loading || saving}
    >
      <div className="catalog-section-heading">
        <div>
          <h2 id={`${id}-title`}>Kit providers</h2>
          <p className="catalog-muted">
            Choose where Sandy finds agents, workloads, and mixins. Save your
            references before discovering kits.
          </p>
        </div>
        {providers !== null && (
          <span className="catalog-badge">
            {providers.filter((provider) => provider.enabled).length} enabled
          </span>
        )}
      </div>
      <p className="catalog-help">
        Enabling a GitHub repository authorizes Sandy to add a repository-scoped
        entry to <code>sbx kit.allowedSources</code> when you launch its kits.
        Existing policy entries are preserved. Managed policy restrictions are
        reported rather than overridden.
      </p>
      {loading && (
        <p className="catalog-status" role="status">
          <span className="spinner" /> Loading saved providers…
        </p>
      )}
      {error && (
        <div className="error-banner" role="alert">
          <strong>Provider configuration needs attention.</strong>
          <p>{error}</p>
          {providers === null && (
            <button
              type="button"
              className="ghost"
              disabled={loading}
              onClick={() => setReload((value) => value + 1)}
            >
              Try again
            </button>
          )}
        </div>
      )}
      {providers !== null && !loading && (
        <form onSubmit={save}>
          <fieldset className="catalog-unframed" disabled={disabled || saving}>
            <legend className="catalog-sr-only">Provider configuration</legend>
            {providers.map((provider) => (
              <ProviderEditor
                key={provider.id}
                provider={provider}
                disabled={disabled || saving}
                onUpdate={(next) =>
                  update(
                    providers.map((item) =>
                      item.id === provider.id ? next : item,
                    ),
                  )
                }
                onRemove={() =>
                  update(providers.filter((item) => item.id !== provider.id))
                }
              />
            ))}
            {!providers.length && (
              <p className="catalog-empty-note">
                No providers configured. Add a GitHub repository or a Docker Hub
                provider to get started.
              </p>
            )}
            <div className="catalog-provider-add">
              <label className="catalog-field" htmlFor={`${id}-kind`}>
                Provider type
                <select
                  id={`${id}-kind`}
                  value={newKind}
                  onChange={(event) =>
                    setNewKind(event.target.value as Provider["kind"])
                  }
                >
                  <option value="github">GitHub</option>
                  <option value="dockerhub">Docker Hub</option>
                </select>
              </label>
              <button
                type="button"
                className="ghost"
                onClick={() =>
                  update([
                    ...providers,
                    {
                      id: crypto.randomUUID(),
                      label: "",
                      kind: newKind,
                      repository: "",
                      reference: "",
                      enabled: true,
                    },
                  ])
                }
              >
                Add provider
              </button>
              {!providers.some(
                (provider) =>
                  normalizeRepository(provider.repository, provider.kind) ===
                    "nikcio/docker-sandboxing" && provider.kind === "github",
              ) && (
                <button
                  type="button"
                  className="text-button"
                  onClick={() =>
                    update([
                      ...providers,
                      {
                        id: crypto.randomUUID(),
                        label: "Docker sandboxing",
                        kind: "github",
                        repository: "nikcio/docker-sandboxing",
                        reference: "",
                        enabled: true,
                      },
                    ])
                  }
                >
                  Add docker-sandboxing
                </button>
              )}
            </div>
            <div className="catalog-actions">
              <button
                type="submit"
                className="primary"
                disabled={!dirty || saving}
              >
                {saving ? "Saving providers…" : "Save providers"}
              </button>
              <span className="catalog-help" role="status">
                {notice ||
                  (incomplete
                    ? "Choose a reference for each enabled repository."
                    : dirty
                      ? "Unsaved provider changes"
                      : "Using the saved provider configuration")}
              </span>
            </div>
          </fieldset>
        </form>
      )}
    </section>
  );
}
