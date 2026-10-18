import { useCallback, useEffect, useRef, useState } from "react";
import { SetupStatus, SandboxInfo, listSandboxes } from "./api";
import Icon from "./Icon";
import { isSetupComplete } from "./Wizard";

interface Props {
  home: boolean;
  status: SetupStatus | null;
  onGoSettings: () => void;
  onGoTemplates: () => void;
  onStub: (feature: string) => void;
  onCount: (count: number) => void;
}

function stateKind(state: string) {
  const normalized = state
    .replace(/^[•○]\s*/, "")
    .trim()
    .toLowerCase();
  if (normalized === "running") return "running";
  if (normalized === "stopped") return "stopped";
  return "inactive";
}

function StatePill({ state }: { state: string }) {
  const label = state.replace(/^[•○]\s*/, "").trim() || "Unknown";
  return (
    <span className={`state-pill ${stateKind(state)}`}>
      <span className="dot" />
      {label.charAt(0).toUpperCase() + label.slice(1)}
    </span>
  );
}

function Avatar({ name }: { name: string }) {
  let hash = 0;
  for (const character of name) {
    hash = (hash * 31 + character.charCodeAt(0)) % 997;
  }
  return (
    <span className={`sbx-avatar palette-${hash % 5}`}>
      <Icon name={hash % 3 === 0 ? "code" : "box"} size={27} />
    </span>
  );
}

export default function CommandCenter({
  home,
  status,
  onGoSettings,
  onGoTemplates,
  onStub,
  onCount,
}: Props) {
  const [sandboxes, setSandboxes] = useState<SandboxInfo[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [selected, setSelected] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [sort, setSort] = useState("default");
  const inFlight = useRef(false);
  const generation = useRef(0);

  const refresh = useCallback(async () => {
    if (inFlight.current) return;
    inFlight.current = true;
    const requestGeneration = generation.current;
    setRefreshing(true);
    try {
      const list = await listSandboxes();
      if (requestGeneration !== generation.current) return;
      setSandboxes(list);
      onCount(list.length);
      setError(null);
      setSelected((current) =>
        list.some((item) => item.name === current)
          ? current
          : (list[0]?.name ?? null),
      );
    } catch (failure) {
      if (requestGeneration === generation.current) setError(String(failure));
    } finally {
      inFlight.current = false;
      if (requestGeneration === generation.current) setRefreshing(false);
    }
  }, [onCount]);

  useEffect(() => {
    // A generation also protects against stale responses during StrictMode remounts.
    generation.current += 1;
    inFlight.current = false;
    if (status?.sbx_installed) refresh();
    return () => {
      generation.current += 1;
    };
  }, [refresh, status?.sbx_installed]);

  useEffect(() => {
    if (!autoRefresh || !status?.sbx_installed) return;
    const timer = setInterval(refresh, 5000);
    return () => clearInterval(timer);
  }, [autoRefresh, refresh, status?.sbx_installed]);

  const all = sandboxes ?? [];
  const visible = all.filter((sandbox) =>
    `${sandbox.name} ${sandbox.agent} ${sandbox.workspace}`
      .toLowerCase()
      .includes(search.trim().toLowerCase()),
  );
  // The API has no creation timestamp. Preserve CLI order rather than sorting UUIDs as dates.
  if (sort === "name") visible.sort((a, b) => a.name.localeCompare(b.name));
  const running = all.filter(
    (sandbox) => stateKind(sandbox.state) === "running",
  ).length;
  const stopped = all.filter(
    (sandbox) => stateKind(sandbox.state) === "stopped",
  ).length;
  const inactive = all.length - running - stopped;
  const detail = all.find((sandbox) => sandbox.name === selected) ?? null;

  return (
    <div className="dashboard">
      <div className="dashboard-main">
        <div className="page-head">
          <h1>
            {home ? (
              <>
                <span className="wave" aria-hidden="true">
                  <Icon name="wave" size={36} />
                </span>{" "}
                Welcome to Sandy
              </>
            ) : (
              "Your Sandboxes"
            )}
          </h1>
          <p className="page-sub">
            {home
              ? "Run and manage your Docker sandboxes — safely, easily, and without the command line."
              : "Your local environments. One comfortable place to keep track of them."}
          </p>
        </div>
        {home && (
          <div className="action-cards">
            <button
              className="action-card featured"
              onClick={() => onStub("Create new sandbox")}
            >
              <span className="action-icon">
                <Icon name="create" size={32} />
              </span>
              <span className="action-title">Create new sandbox</span>
              <span className="action-desc">
                Start with a template or choose your own image.
              </span>
              <span className="action-arrow">
                <Icon name="arrow" size={19} />
              </span>
            </button>
            <button className="action-card" onClick={onGoTemplates}>
              <span className="action-icon blue">
                <Icon name="template" size={27} />
              </span>
              <span className="action-title">From template</span>
              <span className="action-desc">
                Use a pre-configured environment.
              </span>
              <span className="action-arrow">
                <Icon name="arrow" size={18} />
              </span>
            </button>
            <button
              className="action-card"
              onClick={() => onStub("Create from image")}
            >
              <span className="action-icon green">
                <Icon name="box" size={27} />
              </span>
              <span className="action-title">From image</span>
              <span className="action-desc">
                Run any Docker image you like.
              </span>
              <span className="action-arrow">
                <Icon name="arrow" size={18} />
              </span>
            </button>
            <a
              className="action-card"
              href="https://docs.docker.com/ai/sandboxes/"
              target="_blank"
              rel="noreferrer"
            >
              <span className="action-icon purple">
                <Icon name="book" size={27} />
              </span>
              <span className="action-title">Learn more</span>
              <span className="action-desc">
                View quick start guide and tips.
              </span>
              <span className="action-arrow">
                <Icon name="arrow" size={18} />
              </span>
            </a>
          </div>
        )}
        {!isSetupComplete(status) && (
          <div className="setup-banner">
            <Icon name="shield" size={20} />
            <span>A few setup steps still need your attention.</span>
            <button className="text-button" onClick={onGoSettings}>
              Open Settings <Icon name="arrow" size={16} />
            </button>
          </div>
        )}
        <section
          className="card sandbox-card"
          aria-labelledby="sandboxes-title"
        >
          <div className="list-head">
            <div>
              <h2 id="sandboxes-title">Your Sandboxes</h2>
              <p className="list-summary" role="status">
                {sandboxes
                  ? `${all.length} sandbox${all.length === 1 ? "" : "es"} · ${running} running · ${stopped} stopped${inactive ? ` · ${inactive} inactive` : ""}`
                  : !status?.sbx_installed
                    ? "Install the CLI to see your sandboxes"
                    : error
                      ? "Unable to load sandboxes"
                      : "Finding your sandboxes…"}
              </p>
            </div>
            <div className="list-controls">
              <label className="search-field">
                <Icon name="search" size={19} />
                <input
                  aria-label="Search sandboxes"
                  placeholder="Search sandboxes…"
                  value={search}
                  onChange={(event) => setSearch(event.target.value)}
                />
              </label>
              <select
                aria-label="Sort sandboxes"
                value={sort}
                onChange={(event) => setSort(event.target.value)}
              >
                <option value="default">Sort: CLI order</option>
                <option value="name">Sort: Name</option>
              </select>
              <button
                className="icon-button refresh-button"
                title="Refresh sandboxes"
                aria-label="Refresh sandboxes"
                disabled={refreshing || !status?.sbx_installed}
                onClick={refresh}
              >
                <Icon name="refresh" size={18} />
              </button>
            </div>
          </div>
          {error && (
            <div className="error-banner" role="alert">
              <strong>Couldn’t refresh your sandboxes.</strong>
              <p>{error}</p>
              <button className="ghost" disabled={refreshing} onClick={refresh}>
                Try again
              </button>
            </div>
          )}
          {sandboxes === null && !error && status?.sbx_installed && (
            <div className="empty" role="status">
              <span className="spinner" /> Loading your local sandboxes…
            </div>
          )}
          {visible.length === 0 &&
            (sandboxes !== null || !status?.sbx_installed) && (
              <div className="empty">
                <span className="empty-icon">
                  <Icon name="box" size={32} />
                </span>
                <h3>
                  {search
                    ? "No matching sandboxes"
                    : "A little room for your next big idea"}
                </h3>
                <p>
                  {search
                    ? "Try a different name, agent, or workspace."
                    : "Your local sandboxes will appear here. Start one from a project with sbx env run."}
                </p>
                {search && (
                  <button className="ghost" onClick={() => setSearch("")}>
                    Clear search
                  </button>
                )}
              </div>
            )}
          <div className="sbx-rows">
            {visible.map((sandbox) => (
              <div
                key={sandbox.name}
                className={`sbx-row ${selected === sandbox.name ? "selected" : ""}`}
              >
                <button
                  className="sandbox-select"
                  aria-label={`Show details for ${sandbox.name}`}
                  aria-pressed={selected === sandbox.name}
                  onClick={() => setSelected(sandbox.name)}
                >
                  <Avatar name={sandbox.name} />
                  <span className="sbx-info">
                    <span className="sbx-name">
                      {sandbox.name}
                      {sandbox.agent && (
                        <span className="tag">{sandbox.agent}</span>
                      )}
                    </span>
                    <span className="sbx-sub" title={sandbox.workspace}>
                      {sandbox.workspace || "No workspace reported"}
                    </span>
                  </span>
                  <StatePill state={sandbox.state} />
                </button>
                <button
                  className={
                    stateKind(sandbox.state) === "running"
                      ? "primary row-action"
                      : "ghost row-action"
                  }
                  onClick={() => {
                    setSelected(sandbox.name);
                    onStub(
                      stateKind(sandbox.state) === "running"
                        ? "Open sandbox"
                        : "Start sandbox",
                    );
                  }}
                >
                  {stateKind(sandbox.state) === "running" ? "Open" : "Start"}
                </button>
                <button
                  className="icon-button row-more"
                  aria-label={`More actions for ${sandbox.name}`}
                  onClick={() => {
                    setSelected(sandbox.name);
                    onStub("More sandbox actions");
                  }}
                >
                  <Icon name="more" size={19} />
                </button>
              </div>
            ))}
          </div>
          {search && sandboxes !== null && (
            <p className="search-results">
              Showing {visible.length} of {all.length} sandboxes
            </p>
          )}
        </section>
      </div>
      <aside className="inspector" aria-label="Sandbox details">
        <section className="card details-card">
          {detail ? (
            <>
              <div className="details-overview">
                <div className="details-head">
                  <Avatar name={detail.name} />
                  <div>
                    <h2>{detail.name}</h2>
                    <StatePill state={detail.state} />
                  </div>
                  <button
                    className="icon-button"
                    aria-label="More actions for selected sandbox"
                    onClick={() => onStub("More sandbox actions")}
                  >
                    <Icon name="more" size={20} />
                  </button>
                </div>
                <p className="local-label">
                  Local sandbox <span>·</span>{" "}
                  {detail.agent || "No agent reported"}
                </p>
                <div className="sandbox-actions">
                  <button
                    className="primary"
                    onClick={() =>
                      onStub(
                        stateKind(detail.state) === "running"
                          ? "Open sandbox"
                          : "Start sandbox",
                      )
                    }
                  >
                    <Icon name="monitor" size={20} />
                    {stateKind(detail.state) === "running"
                      ? "Open sandbox"
                      : "Start sandbox"}
                  </button>
                  <button
                    className="ghost"
                    onClick={() => onStub("Stop sandbox")}
                  >
                    <Icon name="stop" size={19} />
                    Stop
                  </button>
                  <button
                    className="ghost"
                    onClick={() => onStub("Restart sandbox")}
                  >
                    <Icon name="refresh" size={19} />
                    Restart
                  </button>
                  <button
                    className="ghost"
                    onClick={() => onStub("More sandbox actions")}
                  >
                    More actions <Icon name="chevron" size={16} />
                  </button>
                </div>
              </div>
              <div className="details-body">
                <h3>Details</h3>
                <dl className="details-grid">
                  <dt>Name</dt>
                  <dd>{detail.name}</dd>
                  <dt>Agent</dt>
                  <dd>{detail.agent || "Not reported"}</dd>
                  <dt>Workspace</dt>
                  <dd>{detail.workspace || "Not reported"}</dd>
                  <dt>Status</dt>
                  <dd>
                    <StatePill state={detail.state} />
                  </dd>
                </dl>
                <label className="toggle-row">
                  <span>Auto-refresh</span>
                  <span className="toggle-control">
                    <input
                      type="checkbox"
                      role="switch"
                      checked={autoRefresh}
                      onChange={(event) => setAutoRefresh(event.target.checked)}
                    />
                    <span className="switch-track" />
                    <span>{autoRefresh ? "On" : "Off"}</span>
                  </span>
                </label>
                <div className="tip">
                  <h4>
                    <Icon name="sparkles" size={20} /> Tip
                  </h4>
                  <p>
                    Have a favorite environment? Turn it into a starting point
                    for your next idea.
                  </p>
                  <button
                    className="text-button"
                    onClick={() => onStub("Create from this sandbox")}
                  >
                    Create from this sandbox <Icon name="arrow" size={15} />
                  </button>
                </div>
              </div>
            </>
          ) : (
            <div className="inspector-empty">
              <span className="empty-icon">
                <Icon name="monitor" size={30} />
              </span>
              <h2>A closer look</h2>
              <p>Select a sandbox to see its details and available actions.</p>
              <label className="toggle-row">
                <span>Auto-refresh</span>
                <span className="toggle-control">
                  <input
                    type="checkbox"
                    role="switch"
                    checked={autoRefresh}
                    onChange={(event) => setAutoRefresh(event.target.checked)}
                  />
                  <span className="switch-track" />
                  <span>{autoRefresh ? "On" : "Off"}</span>
                </span>
              </label>
            </div>
          )}
        </section>
      </aside>
    </div>
  );
}
