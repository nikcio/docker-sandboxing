import { useCallback, useEffect, useRef, useState } from "react";
import { SetupStatus, SandboxInfo, listSandboxes } from "./api";

interface Props {
  status: SetupStatus | null;
  onOpenWizard: () => void;
  onRefreshStatus: () => void;
}

function Badge({ label, on }: { label: string; on: boolean }) {
  return (
    <span className={`badge ${on ? "on" : ""}`}>
      <span className="dot" />
      {label}
    </span>
  );
}

function StatePill({ state }: { state: string }) {
  const running = state.toLowerCase() === "running";
  return <span className={`state-pill ${running ? "running" : ""}`}>{state || "unknown"}</span>;
}

export default function CommandCenter({ status, onOpenWizard, onRefreshStatus }: Props) {
  const [sandboxes, setSandboxes] = useState<SandboxInfo[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const inFlight = useRef(false);
  // Keep the latest callback without re-triggering effects when the parent
  // re-renders.
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

  return (
    <main>
      <div className="cc-head">
        <h2>Your sandboxes</h2>
        <div className="badges">
          <Badge label="CLI installed" on={!!status?.sbx_installed} />
          <Badge label="Signed in" on={!!status?.signed_in} />
          <Badge label="Image paste" on={!!status?.image_paste_on} />
          <Badge label="SSH" on={!!status?.ssh_set_up} />
        </div>
        <div className="row">
          <label className="muted" style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
            />
            Auto
          </label>
          <button className="secondary" onClick={refresh}>
            Refresh
          </button>
          <button className="secondary" onClick={onOpenWizard}>
            Setup
          </button>
        </div>
      </div>

      {error && <p className="muted">Could not list sandboxes: {error}</p>}

      {sandboxes && sandboxes.length === 0 && (
        <div className="empty">
          <div className="big">🏜️</div>
          No sandboxes right now.
          <br />
          Sandboxes appear here when an agent starts one — for example by running{" "}
          <code>sbx env run</code> or <code>sbx run</code> in a project.
        </div>
      )}

      {sandboxes && sandboxes.length > 0 && (
        <table className="sandboxes">
          <thead>
            <tr>
              <th>Name</th>
              <th>State</th>
              <th>Agent</th>
              <th>Workspace</th>
            </tr>
          </thead>
          <tbody>
            {sandboxes.map((sbx) => (
              <tr key={sbx.name}>
                <td>
                  <strong>{sbx.name}</strong>
                </td>
                <td>
                  <StatePill state={sbx.state} />
                </td>
                <td>{sbx.agent}</td>
                <td className="muted">{sbx.workspace}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </main>
  );
}
