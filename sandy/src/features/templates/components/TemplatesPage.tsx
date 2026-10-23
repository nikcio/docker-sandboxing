import { useCallback, useEffect, useId, useRef, useState } from "react";
import type { SavedTemplate } from "../../../lib/templates";
import { listTemplates, templateWarnings } from "../../../lib/templates";
import Icon from "../../../common/components/Icon";
import { ErrorBanner } from "../../../common/components/ErrorBanner";
import { useMountedRef } from "../../../common/hooks/useMountedRef";
import { useBusyReporting } from "../../../common/hooks/useBusyReporting";
import { Toast, type ToastNotice } from "../../../common/components/Toast";
import { LaunchDialog } from "./LaunchDialog";
import { DeleteTemplateDialog } from "./DeleteTemplateDialog";

export interface TemplatesPageProps {
  onCreate: () => void;
  onBusyChange?: (busy: boolean) => void;
}

export default function TemplatesPage({ onCreate, onBusyChange }: TemplatesPageProps) {
  const id = useId();
  const mounted = useMountedRef();
  const [templates, setTemplates] = useState<SavedTemplate[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [warnings, setWarnings] = useState<string[]>([]);
  const [warningsError, setWarningsError] = useState<string | null>(null);
  const [notice, setNotice] = useState<ToastNotice | null>(null);
  const [search, setSearch] = useState("");
  const [selected, setSelected] = useState<SavedTemplate | null>(null);
  const [deleting, setDeleting] = useState<SavedTemplate | null>(null);
  const returnFocus = useRef<HTMLButtonElement | null>(null);
  const loadingLock = useRef(false);
  const generation = useRef(0);

  useBusyReporting(selected !== null, onBusyChange);

  const refresh = useCallback(async () => {
    if (loadingLock.current) return;
    loadingLock.current = true;
    const requestGeneration = generation.current;
    setLoading(true);
    setError(null);
    setWarningsError(null);
    try {
      const [templatesResult, warningsResult] = await Promise.allSettled([
        listTemplates(),
        templateWarnings(),
      ]);
      if (!mounted.current || requestGeneration !== generation.current) return;
      if (templatesResult.status === "fulfilled") {
        setTemplates(templatesResult.value);
      } else {
        setError(String(templatesResult.reason));
      }
      if (warningsResult.status === "fulfilled") {
        setWarnings(warningsResult.value);
      } else {
        setWarningsError(String(warningsResult.reason));
      }
    } finally {
      if (requestGeneration === generation.current) {
        loadingLock.current = false;
        if (mounted.current) setLoading(false);
      }
    }
  }, [mounted]);

  useEffect(() => {
    generation.current += 1;
    loadingLock.current = false;
    void refresh();
    return () => {
      generation.current += 1;
    };
  }, [refresh]);

  const visible = (templates ?? []).filter((template) =>
    `${template.name} ${template.agent} ${template.workspace} ${template.path}`
      .toLowerCase()
      .includes(search.trim().toLowerCase()),
  );

  return (
    <section className="standalone-page catalog-page" aria-labelledby={`${id}-title`}>
      <header className="page-head catalog-page-heading">
        <div>
          <h1 id={`${id}-title`}>Templates</h1>
          <p className="page-sub">Your saved environments, ready for the next project.</p>
        </div>
        <button type="button" className="primary" onClick={onCreate}>
          <Icon name="create" size={18} /> Create template
        </button>
      </header>
      <div className="card catalog-panel" aria-busy={loading}>
        <div className="catalog-section-heading">
          <div>
            <h2>Saved on this computer</h2>
            <p className="catalog-muted">
              Reuse the same tools with a different sandbox name or workspace. Your saved template
              stays the same.
            </p>
          </div>
          <button type="button" className="ghost" disabled={loading} onClick={() => void refresh()}>
            <Icon name="refresh" size={16} />
            {loading ? "Refreshing…" : "Refresh"}
          </button>
        </div>
        {error && (
          <ErrorBanner
            title="Couldn’t load saved templates."
            actionLabel="Try again"
            disabled={loading}
            onAction={() => void refresh()}
          >
            <p>{error}</p>
          </ErrorBanner>
        )}
        {warningsError && (
          <ErrorBanner
            title="Couldn’t check template file warnings."
            actionLabel="Retry file check"
            disabled={loading}
            onAction={() => void refresh()}
          >
            <p>{warningsError}</p>
          </ErrorBanner>
        )}
        {warnings.length > 0 && (
          <details className="catalog-warnings" open>
            <summary>Template file warnings ({warnings.length})</summary>
            <p>Some saved files could not be read. Readable templates are still available below.</p>
            <ul>
              {warnings.map((warning, index) => (
                <li key={index}>{warning}</li>
              ))}
            </ul>
          </details>
        )}
        {loading && templates === null && (
          <p className="catalog-status" role="status">
            <span className="spinner" /> Loading saved templates…
          </p>
        )}
        {templates !== null && templates.length === 0 && (
          <div className="template-empty">
            <span className="empty-icon">
              <Icon name="template" size={32} />
            </span>
            <h2>A starting point worth keeping</h2>
            <p>Create an environment with your agent and mixins, then save it here to use again.</p>
            <button type="button" className="primary" onClick={onCreate}>
              Create your first template <Icon name="arrow" size={16} />
            </button>
          </div>
        )}
        {!!templates?.length && (
          <>
            <label className="catalog-field catalog-search" htmlFor={`${id}-search`}>
              Search saved templates
              <input
                id={`${id}-search`}
                type="search"
                value={search}
                placeholder="Search templates"
                onChange={(event) => setSearch(event.target.value)}
              />
            </label>
            <p className="catalog-help" role="status">
              {visible.length} of {templates.length} template
              {templates.length === 1 ? "" : "s"}
            </p>
            <div className="catalog-template-list">
              {visible.map((template) => (
                <article className="catalog-template" key={template.id}>
                  <div className="catalog-section-heading">
                    <div>
                      <h3>{template.name}</h3>
                      <p className="catalog-template-meta">
                        {template.agent} <span className="catalog-badge">Kit v{template.kit_version}</span>
                      </p>
                    </div>
                    <div className="catalog-template-actions">
                      <button
                        type="button"
                        className="ghost"
                        aria-label={`Delete ${template.name}`}
                        onClick={(event) => {
                          returnFocus.current = event.currentTarget;
                          setDeleting(template);
                        }}
                      >
                        <Icon name="trash" size={16} /> Delete
                      </button>
                      <button
                        type="button"
                        className="primary"
                        aria-label={`Launch ${template.name}`}
                        onClick={() => setSelected(template)}
                      >
                        Launch <Icon name="arrow" size={16} />
                      </button>
                    </div>
                  </div>
                  <dl className="catalog-review-grid">
                    <dt>Workspace</dt>
                    <dd>{template.workspace || "Not set"}</dd>
                    <dt>Template file</dt>
                    <dd>
                      <code>{template.path}</code>
                    </dd>
                  </dl>
                  <details className="catalog-output">
                    <summary>Preview saved YAML</summary>
                    <pre className="catalog-yaml" aria-label={`Saved YAML for ${template.name}`}>
                      <code>{template.content}</code>
                    </pre>
                  </details>
                </article>
              ))}
            </div>
            {!visible.length && (
              <div className="catalog-empty-note">
                <p>No templates match this search.</p>
                <button type="button" className="ghost" onClick={() => setSearch("")}>
                  Clear search
                </button>
              </div>
            )}
          </>
        )}
      </div>
      {selected && (
        <LaunchDialog
          key={selected.id}
          template={selected}
          onLaunched={(text) => setNotice({ kind: "success", text })}
          onClose={() => setSelected(null)}
        />
      )}
      {deleting && (
        <DeleteTemplateDialog
          key={deleting.id}
          template={deleting}
          onDeleted={() => void refresh()}
          onClose={() => {
            setDeleting(null);
            returnFocus.current?.focus();
          }}
        />
      )}
      <div className="dashboard-notifications" aria-live="polite" aria-atomic="true">
        {notice && <Toast notice={notice} onDismiss={() => setNotice(null)} />}
      </div>
    </section>
  );
}
