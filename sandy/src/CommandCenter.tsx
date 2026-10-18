import { useCallback, useEffect, useRef, useState } from "react";
import { SetupStatus, SandboxInfo, listSandboxes } from "./api";

interface Props {
  status: SetupStatus | null;
  onRefreshStatus: () => void;
}

function StatePill({ state }: { state: string }) {
  const running = state.toLowerCase().includes("running");
  const stopped = state.toLowerCase().includes("stopped");
  const label = state.replace(/^[•○]\s*/, "").trim() || "unknown";
  return (
    <span className={`state-pill ${running ? "running" : stopped ? "stopped" : ""}`}>
      <span className="dot" />
      {label.charAt(0).toUpperCase() + label.slice(1)}
    </span>
  );
}

function Avatar({ name, agent }: { name: string; agent: string }) {
  // Deterministic pastel per sandbox name.
  const palettes = [
    ["#e3f7ed", "#2fbf71"],
    ["#e3efff", "#3b82f6"],
    ["#eeebff", "#6c5ce7"],
    ["#fff4dd", "#f59e0b"],
    ["#fdebec", "#e5484d"],
  ];
  let h = 0;
  for (const ch of name) h = (h * 31 + ch.charCodeAt(0)) % 997;
  const [bg, fg] = palettes[h % palettes.length];
  return (
    <div className="sbx-avatar" style={{ background: bg, color: fg }}>
      {agent ? agent.charAt(0).toUpperCase() : name.charAt(0).toUpperCase()}
    </div>
  );
}

export default function CommandCenter({ status, onRefreshStatus }: Props) {
  const [sandboxes, setSandboxes] = useState<SandboxInfo[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [selected, setSelected] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [sort, setSort] = useState<"newest" | "name">("newest");
  const inFlight = useRef(false);
  const refreshStatusRef = useRef(onRefreshStatus);
  useEffect(() => {
    refreshStatusRef.current = onRefreshStatus;
  }, [onRefreshStatus]);

  const refresh = useCallback(async () => {
    if (inFlight.current) return;
    inFlight.current = true;
    try {
      const list = await listSandboxes();
      setSandboxes(list);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      inFlight.current = false;
    }
    refreshStatusRef.current();
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    if (!autoRefresh) return;
    const t = setInterval(refresh, 5000);
    return () => clearInterval(t);
  }, [autoRefresh, refresh]);

  const visible = (sandboxes ?? [])
    .filter((s) => s.name.toLowerCase().includes(search.toLowerCase()))
    .sort((a, b) =>
      sort === "name" ? a.name.localeCompare(b.name) : b.raw.localeCompare(a.raw)
    );

  const running = visible.filter((s) => s.state.toLowerCase().includes("running")).length;
  const stopped = visible.filter((s) => s.state.toLowerCase().includes("stopped")).length;
  const sel = visible.find((s) => s.name === selected) ?? null;

  return (
    <div className="page">
      <div className="page-head">
        <div>
          <h1>Your Sandboxes</h1>
          <p className="page-sub">
            {sandboxes
              ? `${sandboxes.length} sandbox${sandboxes.length === 1 ? "" : "es"} · ${running} running · ${stopped} stopped`
              : "Loading…"}
            {!status?.sbx_installed && " · CLI not installed"}
            {status?.sbx_installed && !status.signed_in && " · not signed in"}
          </p>
        </div>
        <div className="cc-actions">
          <input
            className="search"
            placeholder="Search sandboxes…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
          <select
            className="select"
            value={sort}
            onChange={(e) => setSort(e.target.value as "newest" | "name")}
          >
            <option value="newest">Sort: Newest</option>
            <option value="name">Sort: Name</option>
          </select>
        </div>
      </div>

      {error && <p className="muted">Could not list sandboxes: {error}</p>}

      <div className="card sandbox-card">
        {visible.length === 0 && (
          <div className="empty">
            <div className="big">🏜️</div>
            {sandboxes && sandboxes.length > 0
              ? "No sandboxes match your search."
              : "No sandboxes right now."}
            <br />
            Sandboxes appear here when an agent starts one — for example by running{" "}
            <code>sbx env run</code> or <code>sbx run</code> in a project.
          </div>
        )}

        {visible.length > 0 && (
          <div className="sbx-rows">
            {visible.map((sbx) => (
              <div
                key={sbx.name}
                className={`sbx-row ${selected === sbx.name ? "selected" : ""}`}
                onClick={() => setSelected(sbx.name === selected ? null : sbx.name)}
              >
                <Avatar name={sbx.name} agent={sbx.agent} />
                <div className="sbx-info">
                  <div className="sbx-name">
                    {sbx.name}
                    {sbx.agent && <span className="tag">{sbx.agent}</span>}
                  </div>
                  {sbx.workspace && <div className="sbx-sub">{sbx.workspace}</div>}
                </div>
                <div className="sbx-state">
                  <StatePill state={sbx.state} />
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {sel && (
        <div className="card details-card">
          <div className="details-head">
            <Avatar name={sel.name} agent={sel.agent} />
            <div>
              <div className="sbx-name" style={{ fontSize: 16 }}>{sel.name}</div>
              <StatePill state={sel.state} />
            </div>
          </div>
          <div className="details-grid">
            <div className="details-label">Name</div>
            <div>{sel.name}</div>
            <div className="details-label">Agent</div>
            <div>{sel.agent || "—"}</div>
            <div className="details-label">Workspace</div>
            <div>{sel.workspace || "—"}</div>
            <div className="details-label">Status</div>
            <div>
              <StatePill state={sel.state} />
            </div>
          </div>
          <label className="toggle-row">
            <span className="details-label">Auto-refresh</span>
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
            />
            On
          </label>
          <div className="tip">
            <div className="tip-title">✦ Tip</div>
            Connecting, starting, and creating sandboxes from Sandy is coming in a
            future version — use <code>sbx run</code> or <code>sbx attach {sel.name}</code> for now.
          </div>
        </div>
      )}
    </div>
  );
}
