import { useCallback, useEffect, useId, useRef, useState } from "react";
import type { FormEvent } from "react";
import { launchTemplate, listTemplates, pickWorkspace, templateWarnings } from "./catalog-api";
import type { SavedTemplate } from "./catalog-api";
import Icon from "./Icon";

export interface TemplatesProps {
  onCreate: () => void;
  onLaunched: () => void;
  onBusyChange?: (busy: boolean) => void;
}

export default function Templates({ onCreate, onLaunched, onBusyChange }: TemplatesProps) {
  const id = useId();
  const [templates, setTemplates] = useState<SavedTemplate[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [warnings, setWarnings] = useState<string[]>([]);
  const [warningsError, setWarningsError] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [selected, setSelected] = useState<SavedTemplate | null>(null);
  const [name, setName] = useState("");
  const [workspace, setWorkspace] = useState("");
  const [busy, setBusy] = useState<"launch" | "workspace" | null>(null);
  const [launchError, setLaunchError] = useState<string | null>(null);
  const [notice, setNotice] = useState("");
  const [output, setOutput] = useState("");
  const dialog = useRef<HTMLDialogElement>(null);
  const loadingLock = useRef(false);
  const actionLock = useRef(false);
  const mounted = useRef(false);
  const generation = useRef(0);
  const returnFocus = useRef<HTMLButtonElement | null>(null);
  const busyCallback = useRef(onBusyChange);
  busyCallback.current = onBusyChange;

  useEffect(() => {
    busyCallback.current?.(busy !== null);
  }, [busy]);

  useEffect(() => {
    return () => {
      busyCallback.current?.(false);
    };
  }, []);

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
  }, []);

  useEffect(() => {
    mounted.current = true;
    generation.current += 1;
    loadingLock.current = false;
    void refresh();
    return () => {
      mounted.current = false;
      generation.current += 1;
    };
  }, [refresh]);

  useEffect(() => {
    if (selected && !dialog.current?.open) {
      dialog.current?.showModal();
      dialog.current?.querySelector<HTMLInputElement>("input")?.focus();
    }
  }, [selected]);

  const close = () => {
    if (actionLock.current) return;
    dialog.current?.close();
  };

  const browse = async () => {
    if (actionLock.current) return;
    actionLock.current = true;
    setBusy("workspace");
    setLaunchError(null);
    try {
      const path = await pickWorkspace();
      if (path && mounted.current) setWorkspace(path);
    } catch (failure) {
      if (mounted.current) setLaunchError(String(failure));
    } finally {
      actionLock.current = false;
      if (mounted.current) setBusy(null);
    }
  };

  const launch = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (actionLock.current || !selected) return;
    if (!name.trim() || !workspace.trim()) {
      setLaunchError("Enter a sandbox name and a workspace folder.");
      return;
    }
    actionLock.current = true;
    setBusy("launch");
    setLaunchError(null);
    setNotice("");
    setOutput("");
    try {
      const result = await launchTemplate({
        id: selected.id,
        name: name.trim(),
        workspace: workspace.trim(),
      });
      if (!mounted.current) return;
      setOutput(result);
      setNotice(`${name.trim()} launched from ${selected.name}.`);
      dialog.current?.close();
      onLaunched();
    } catch (failure) {
      if (mounted.current) setLaunchError(String(failure));
    } finally {
      actionLock.current = false;
      if (mounted.current) setBusy(null);
    }
  };

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
        <button type="button" className="primary" disabled={busy !== null} onClick={onCreate}>
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
          <button
            type="button"
            className="ghost"
            disabled={loading || busy !== null}
            onClick={refresh}
          >
            <Icon name="refresh" size={16} />
            {loading ? "Refreshing…" : "Refresh"}
          </button>
        </div>
        {error && (
          <div className="error-banner" role="alert">
            <strong>Couldn’t load saved templates.</strong>
            <p>{error}</p>
            <button type="button" className="ghost" disabled={loading} onClick={refresh}>
              Try again
            </button>
          </div>
        )}
        {warningsError && (
          <div className="error-banner" role="alert">
            <strong>Couldn’t check template file warnings.</strong>
            <p>{warningsError}</p>
            <button type="button" className="ghost" disabled={loading} onClick={refresh}>
              Retry file check
            </button>
          </div>
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
        {notice && (
          <div className="catalog-success" role="status">
            <Icon name="check" size={18} />
            {notice}
          </div>
        )}
        {output && (
          <details className="catalog-output">
            <summary>Launch output</summary>
            <pre>{output}</pre>
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
                        {template.agent}{" "}
                        <span className="catalog-badge">Kit v{template.kit_version}</span>
                      </p>
                    </div>
                    <button
                      type="button"
                      className="primary"
                      disabled={busy !== null}
                      aria-label={`Launch ${template.name}`}
                      onClick={(event) => {
                        returnFocus.current = event.currentTarget;
                        setName(template.name);
                        setWorkspace(template.workspace);
                        setLaunchError(null);
                        setSelected(template);
                      }}
                    >
                      Launch <Icon name="arrow" size={16} />
                    </button>
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
      <dialog
        ref={dialog}
        className="catalog-launch-dialog"
        aria-labelledby={`${id}-launch-title`}
        aria-describedby={`${id}-launch-description`}
        onCancel={(event) => {
          if (actionLock.current) event.preventDefault();
        }}
        onClose={() => {
          setSelected(null);
          setLaunchError(null);
          returnFocus.current?.focus();
        }}
      >
        <form onSubmit={launch} aria-busy={busy !== null}>
          <div className="catalog-section-heading">
            <h2 id={`${id}-launch-title`}>Launch {selected?.name}</h2>
            <button
              type="button"
              className="icon-button"
              aria-label="Close launch dialog"
              disabled={busy !== null}
              onClick={close}
            >
              <Icon name="close" size={20} />
            </button>
          </div>
          <p className="catalog-muted" id={`${id}-launch-description`}>
            Choose a new name or workspace to reuse this environment. These choices apply to this
            launch.
          </p>
          {launchError && (
            <div className="error-banner" role="alert">
              <strong>Couldn’t launch this sandbox.</strong>
              <p>{launchError}</p>
              <span>Edit the details or try launching again.</span>
            </div>
          )}
          <fieldset className="catalog-unframed" disabled={busy !== null}>
            <legend className="catalog-sr-only">Sandbox launch details</legend>
            <label className="catalog-field" htmlFor={`${id}-launch-name`}>
              Sandbox name
              <input
                id={`${id}-launch-name`}
                autoFocus
                required
                maxLength={128}
                pattern="[a-zA-Z0-9][a-zA-Z0-9_.\\-]*"
                value={name}
                placeholder="Sandbox name"
                onChange={(event) => setName(event.target.value)}
              />
              <span className="catalog-help">
                Use a new name to create a separate sandbox. Start with a letter or number; use
                letters, numbers, dots, underscores, or hyphens.
              </span>
            </label>
            <label className="catalog-field" htmlFor={`${id}-launch-workspace`}>
              Workspace folder
            </label>
            <div className="catalog-path-row">
              <input
                id={`${id}-launch-workspace`}
                required
                value={workspace}
                placeholder="Absolute project path"
                onChange={(event) => setWorkspace(event.target.value)}
              />
              <button type="button" className="ghost" onClick={browse}>
                Browse…
              </button>
            </div>
            <div className="catalog-actions">
              <button type="button" className="ghost" onClick={close}>
                Cancel
              </button>
              <button
                type="submit"
                className="primary"
                disabled={!name.trim() || !workspace.trim()}
              >
                {busy === "launch" ? (
                  <>
                    <span className="spinner" /> Launching…
                  </>
                ) : launchError ? (
                  "Retry launch"
                ) : (
                  "Launch sandbox"
                )}
              </button>
            </div>
          </fieldset>
          {busy === "workspace" && (
            <p className="catalog-status" role="status">
              Choose a workspace in the folder picker…
            </p>
          )}
          {busy === "launch" && (
            <p className="catalog-status" role="status">
              Creating and starting your sandbox. This may take a little while.
            </p>
          )}
        </form>
      </dialog>
    </section>
  );
}
