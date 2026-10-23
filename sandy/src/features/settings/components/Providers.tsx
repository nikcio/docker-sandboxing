import { useEffect, useId, useRef, useState } from "react";
import type { FormEvent } from "react";
import type { Provider } from "../../../lib/catalog";
import { getProviders, saveProviders, isHubSearchProvider } from "../../../lib/catalog";
import { ErrorBanner } from "../../../common/components/ErrorBanner";
import { normalizeRepository, repositoryIsValid } from "../providerRepository";
import { ProviderEditor } from "./ProviderEditor";

export interface ProvidersProps {
  onChange?: (providers: Provider[]) => void;
  onDirtyChange?: (dirty: boolean) => void;
  disabled?: boolean;
}

export default function Providers({ onChange, onDirtyChange, disabled = false }: ProvidersProps) {
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
  // An empty GitHub reference is valid: discovery resolves it to the latest tag.
  const incomplete =
    providers?.some(
      (provider) =>
        provider.enabled &&
        (isHubSearchProvider(provider)
          ? !repositoryIsValid(provider.repository, provider.kind)
          : !repositoryIsValid(provider.repository, provider.kind) ||
            (provider.kind === "dockerhub" && !provider.reference.trim())),
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
    callbacks.current.onDirtyChange?.(loading || providers === null || dirty || incomplete);
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
          !repositoryIsValid(provider.repository, provider.kind) ||
          // An empty GitHub reference is valid: discovery resolves it to the latest tag.
          (provider.kind === "dockerhub" &&
            provider.enabled &&
            !isHubSearchProvider(provider) &&
            !provider.reference.trim()),
      )
    ) {
      setError("Give each provider a name and a valid repository before saving.");
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
    <section className="catalog-providers" aria-labelledby={`${id}-title`} aria-busy={loading || saving}>
      <div className="catalog-section-heading">
        <div>
          <h2 id={`${id}-title`}>Kit providers</h2>
          <p className="catalog-muted">
            Choose where Sandy finds agents, workloads, and mixins. Save your references before
            discovering kits.
          </p>
        </div>
        {providers !== null && (
          <span className="catalog-badge">
            {providers.filter((provider) => provider.enabled).length} enabled
          </span>
        )}
      </div>
      <p className="catalog-help">
        Enabling a GitHub repository authorizes Sandy to add a repository-scoped entry to{" "}
        <code>sbx kit.allowedSources</code> when you launch its kits. Existing policy entries are
        preserved. Managed policy restrictions are reported rather than overridden.
      </p>
      {loading && (
        <p className="catalog-status" role="status">
          <span className="spinner" /> Loading saved providers…
        </p>
      )}
      {error && (
        <ErrorBanner
          title="Provider configuration needs attention."
          actionLabel={providers === null ? "Try again" : undefined}
          onAction={providers === null ? () => setReload((value) => value + 1) : undefined}
          disabled={loading}
        >
          <p>{error}</p>
        </ErrorBanner>
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
                  update(providers.map((item) => (item.id === provider.id ? next : item)))
                }
                onRemove={() => update(providers.filter((item) => item.id !== provider.id))}
              />
            ))}
            {!providers.length && (
              <p className="catalog-empty-note">
                No providers configured. Add a GitHub repository or a Docker Hub provider to get
                started.
              </p>
            )}
            <div className="catalog-provider-add">
              <label className="catalog-field" htmlFor={`${id}-kind`}>
                Provider type
                <select
                  id={`${id}-kind`}
                  value={newKind}
                  onChange={(event) => setNewKind(event.target.value as Provider["kind"])}
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
              <button type="submit" className="primary" disabled={!dirty || saving}>
                {saving ? "Saving providers…" : "Save providers"}
              </button>
              <span className="catalog-help" role="status">
                {notice ||
                  (incomplete
                    ? "Give each enabled repository a valid repository URL."
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
