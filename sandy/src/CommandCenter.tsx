import { useCallback, useEffect, useRef, useState } from "react";
import {
  SetupStatus,
  SandboxInfo,
  listSandboxes,
  startSandbox,
  stopSandbox,
  restartSandbox,
  openTerminal,
} from "./api";
import ConnectSandbox from "./ConnectSandbox";
import Icon from "./Icon";

export interface CommandCenterProps {
  home: boolean;
  status: SetupStatus | null;
  onGoSettings: () => void;
  onGoTemplates: () => void;
  onCreate: () => void;
  onRefreshStatus: () => Promise<SetupStatus | null>;
  onCount: (count: number) => void;
}

type SandboxAction = "start" | "stop" | "restart" | "terminal";
const actions = {
  start: startSandbox,
  stop: stopSandbox,
  restart: restartSandbox,
  terminal: openTerminal,
};
const actionLabels: Record<SandboxAction, string> = {
  start: "Starting…",
  stop: "Stopping…",
  restart: "Restarting…",
  terminal: "Opening terminal…",
};
const successLabels: Record<SandboxAction, string> = {
  start: "Started",
  stop: "Stopped",
  restart: "Restarted",
  terminal: "Opened a terminal for",
};

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
  onCreate,
  onRefreshStatus,
  onCount,
}: CommandCenterProps) {
  const [sandboxes, setSandboxes] = useState<SandboxInfo[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [selected, setSelected] = useState<string | null>(null);
  const [connecting, setConnecting] = useState<SandboxInfo | null>(null);
  const [pending, setPending] = useState(() => new Map<string, SandboxAction>());
  const [notice, setNotice] = useState<{
    kind: "success" | "error";
    text: string;
  } | null>(null);
  const [search, setSearch] = useState("");
  const [sort, setSort] = useState("default");
  const busySandboxes = useRef(new Set<string>());
  const inFlight = useRef<{
    generation: number;
    promise: Promise<void>;
  } | null>(null);
  const generation = useRef(0);

  const refresh = useCallback(
    async (afterAction = false): Promise<void> => {
      const requestGeneration = generation.current;
      const existing = inFlight.current;
      if (existing?.generation === requestGeneration) {
        await existing.promise;
        // A poll started before an action finished can contain stale state.
        if (!afterAction || requestGeneration !== generation.current) return;
        // Another completed action may already have queued the fresh request.
        if (inFlight.current?.generation === requestGeneration) {
          return inFlight.current.promise;
        }
      }
      if (requestGeneration !== generation.current) return;
      const request = { generation: requestGeneration, promise: Promise.resolve() };
      inFlight.current = request;
      setRefreshing(true);
      request.promise = (async () => {
        try {
          const list = await listSandboxes();
          if (requestGeneration !== generation.current) return;
          setSandboxes(list);
          onCount(list.length);
          setError(null);
          setSelected((current) =>
            list.some((item) => item.name === current) ? current : (list[0]?.name ?? null),
          );
        } catch (failure) {
          if (requestGeneration === generation.current) setError(String(failure));
        } finally {
          if (inFlight.current === request) {
            inFlight.current = null;
            if (requestGeneration === generation.current) setRefreshing(false);
          }
        }
      })();
      await request.promise;
    },
    [onCount],
  );

  useEffect(() => {
    // A generation also protects against stale responses during StrictMode remounts.
    generation.current += 1;
    if (status?.sbx_installed) void refresh();
    return () => {
      generation.current += 1;
    };
  }, [refresh, status?.sbx_installed]);

  useEffect(() => {
    if (!status?.sbx_installed) return;
    const timer = setInterval(() => void refresh(), 5000);
    const onFocus = () => void refresh(true);
    window.addEventListener("focus", onFocus);
    return () => {
      clearInterval(timer);
      window.removeEventListener("focus", onFocus);
    };
  }, [refresh, status?.sbx_installed]);

  const runAction = async (name: string, action: SandboxAction) => {
    if (busySandboxes.current.has(name)) {
      throw new Error(`An action is already in progress for ${name}.`);
    }
    const actionGeneration = generation.current;
    busySandboxes.current.add(name);
    setPending((current) => new Map(current).set(name, action));
    setNotice(null);
    try {
      await actions[action](name);
      if (actionGeneration === generation.current) {
        setNotice({ kind: "success", text: `${successLabels[action]} ${name}.` });
      }
    } catch (failure) {
      if (actionGeneration === generation.current) {
        setNotice({
          kind: "error",
          text: `Couldn’t ${action === "terminal" ? "open a terminal for" : action} ${name}: ${String(failure)}`,
        });
      }
      throw failure;
    } finally {
      if (actionGeneration === generation.current) await refresh(true);
      busySandboxes.current.delete(name);
      if (actionGeneration === generation.current) {
        setPending((current) => {
          const next = new Map(current);
          next.delete(name);
          return next;
        });
      }
    }
  };

  const openConnection = (sandbox: SandboxInfo) => {
    if (busySandboxes.current.has(sandbox.name)) return;
    setSelected(sandbox.name);
    setConnecting(sandbox);
  };

  const all = sandboxes ?? [];
  const visible = all.filter((sandbox) =>
    `${sandbox.name} ${sandbox.agent} ${sandbox.workspace}`
      .toLowerCase()
      .includes(search.trim().toLowerCase()),
  );
  // The API has no creation timestamp. Preserve CLI order rather than sorting UUIDs as dates.
  if (sort === "name") visible.sort((a, b) => a.name.localeCompare(b.name));
  const running = all.filter((sandbox) => stateKind(sandbox.state) === "running").length;
  const stopped = all.filter((sandbox) => stateKind(sandbox.state) === "stopped").length;
  const inactive = all.length - running - stopped;
  const detail = all.find((sandbox) => sandbox.name === selected) ?? null;
  const detailPending = detail ? pending.get(detail.name) : undefined;
  const detailRunning = detail !== null && stateKind(detail.state) === "running";
  const needsSetup =
    status !== null &&
    [status.sbx_installed, status.signed_in, status.image_paste_on, status.ssh_set_up].some(
      (value) => value === false,
    );

  return (
    <div className="dashboard">
      <div className="dashboard-main">
        <div className="page-head dashboard-page-head">
          <div>
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
                ? "Run and manage your Docker sandboxes."
                : "Your local environments. One comfortable place to keep track of them."}
            </p>
          </div>
          {!home && (
            <button className="primary" onClick={onCreate}>
              <Icon name="create" size={19} /> Create sandbox
            </button>
          )}
        </div>
        {home && (
          <div className="action-cards">
            <button className="action-card featured" onClick={onCreate}>
              <span className="action-icon">
                <Icon name="create" size={32} />
              </span>
              <span className="action-title">Create sandbox</span>
              <span className="action-desc">Start with a project and template.</span>
              <span className="action-arrow">
                <Icon name="arrow" size={19} />
              </span>
            </button>
            <button className="action-card" onClick={onGoTemplates}>
              <span className="action-icon blue">
                <Icon name="template" size={27} />
              </span>
              <span className="action-title">From template</span>
              <span className="action-desc">Use a pre-configured environment.</span>
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
              <span className="action-desc">View quick start guide and tips.</span>
              <span className="action-arrow">
                <Icon name="arrow" size={18} />
              </span>
            </a>
          </div>
        )}
        {needsSetup && (
          <div className="setup-banner">
            <Icon name="shield" size={20} />
            <span>A few setup steps still need your attention.</span>
            <button className="text-button" onClick={onGoSettings}>
              Open Settings <Icon name="arrow" size={16} />
            </button>
          </div>
        )}
        <section className="card sandbox-card" aria-labelledby="sandboxes-title">
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
                onClick={() => void refresh()}
              >
                <Icon name="refresh" size={18} />
              </button>
            </div>
          </div>
          {error && (
            <div className="error-banner" role="alert">
              <strong>Couldn’t refresh your sandboxes.</strong>
              <p>{error}</p>
              <button className="ghost" disabled={refreshing} onClick={() => void refresh()}>
                Try again
              </button>
            </div>
          )}
          {sandboxes === null && !error && status?.sbx_installed && (
            <div className="empty" role="status">
              <span className="spinner" /> Loading your local sandboxes…
            </div>
          )}
          {visible.length === 0 && (sandboxes !== null || !status?.sbx_installed) && (
            <div className="empty">
              <span className="empty-icon">
                <Icon name="box" size={32} />
              </span>
              <h3>{search ? "No matching sandboxes" : "A little room for your next big idea"}</h3>
              <p>
                {search
                  ? "Try a different name, agent, or workspace."
                  : "Create a sandbox from a template and a project. Your local environments will appear here."}
              </p>
              {search && (
                <button className="ghost" onClick={() => setSearch("")}>
                  Clear search
                </button>
              )}
              {!search && (
                <button className="primary" onClick={onCreate}>
                  Create sandbox
                </button>
              )}
            </div>
          )}
          <div className="sbx-rows">
            {visible.map((sandbox) => (
              <div
                key={sandbox.name}
                className={`sbx-row ${selected === sandbox.name ? "selected" : ""}`}
                aria-busy={pending.has(sandbox.name)}
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
                      {sandbox.agent && <span className="tag">{sandbox.agent}</span>}
                    </span>
                    <span className="sbx-sub" title={sandbox.workspace}>
                      {sandbox.workspace || "No workspace reported"}
                    </span>
                  </span>
                  <StatePill state={sandbox.state} />
                </button>
                <button
                  className="primary row-action"
                  disabled={pending.has(sandbox.name) || !status?.sbx_installed}
                  aria-label={`Open ${sandbox.name}`}
                  onClick={() => openConnection(sandbox)}
                >
                  {pending.has(sandbox.name) ? (
                    <span className="spinner" aria-hidden="true" />
                  ) : (
                    "Open"
                  )}
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
                </div>
                <p className="local-label">
                  Local sandbox <span>·</span> {detail.agent || "No agent reported"}
                </p>
                <div className="sandbox-actions" aria-busy={!!detailPending}>
                  <button
                    className="primary connect-action"
                    disabled={!!detailPending || !status?.sbx_installed}
                    onClick={() => openConnection(detail)}
                  >
                    <Icon name="monitor" size={20} />
                    Open sandbox
                  </button>
                  <button
                    className="ghost start-action"
                    disabled={!!detailPending || detailRunning || !status?.sbx_installed}
                    onClick={() => void runAction(detail.name, "start").catch(() => {})}
                  >
                    <Icon name="terminal" size={19} />
                    Start
                  </button>
                  <button
                    className="ghost"
                    disabled={!!detailPending || !detailRunning || !status?.sbx_installed}
                    onClick={() => void runAction(detail.name, "stop").catch(() => {})}
                  >
                    <Icon name="stop" size={19} />
                    Stop
                  </button>
                  <button
                    className="ghost"
                    disabled={!!detailPending || !detailRunning || !status?.sbx_installed}
                    onClick={() => void runAction(detail.name, "restart").catch(() => {})}
                  >
                    <Icon name="refresh" size={19} />
                    Restart
                  </button>
                </div>
                {detailPending && (
                  <p className="sandbox-pending" role="status">
                    <span className="spinner" /> {actionLabels[detailPending]}
                  </p>
                )}
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
                <div className="tip">
                  <h4>
                    <Icon name="sparkles" size={20} /> Tip
                  </h4>
                  <p>Templates give your next project a ready-to-use environment.</p>
                  <button className="text-button" onClick={onCreate}>
                    Create another sandbox <Icon name="arrow" size={15} />
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
            </div>
          )}
        </section>
      </aside>
      <div className="dashboard-notifications" aria-live="polite" aria-atomic="true">
        {notice && (
          <div
            className={`dashboard-toast ${notice.kind}`}
            role={notice.kind === "error" ? "alert" : "status"}
          >
            <Icon name={notice.kind === "success" ? "check" : "help"} size={20} />
            <span>{notice.text}</span>
            <button
              className="icon-button"
              aria-label="Dismiss notification"
              onClick={() => setNotice(null)}
            >
              <Icon name="close" size={17} />
            </button>
          </div>
        )}
      </div>
      {connecting && (
        <ConnectSandbox
          key={connecting.name}
          sandbox={all.find((sandbox) => sandbox.name === connecting.name) ?? connecting}
          status={status}
          onRefreshStatus={onRefreshStatus}
          onOpenTerminal={() => runAction(connecting.name, "terminal")}
          onClose={() => setConnecting(null)}
        />
      )}
    </div>
  );
}
