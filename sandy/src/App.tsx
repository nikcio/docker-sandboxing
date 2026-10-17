import { useCallback, useEffect, useState } from "react";
import { SetupStatus, getStatus } from "./api";
import Wizard from "./Wizard";
import CommandCenter from "./CommandCenter";

type View = "loading" | "setup" | "sandboxes";

export default function App() {
  const [view, setView] = useState<View>("loading");
  const [status, setStatus] = useState<SetupStatus | null>(null);

  // Only refreshes the status object — never the view. View changes are
  // explicit (wizard finished / sidebar navigation), so refreshing
  // mid-wizard can't unmount it or flip the user back to the list.
  const refreshStatus = useCallback(async () => {
    try {
      setStatus(await getStatus());
    } catch (e) {
      console.error("status check failed", e);
    }
  }, []);

  useEffect(() => {
    (async () => {
      try {
        const s = await getStatus();
        setStatus(s);
        setView(s.sbx_installed && s.signed_in ? "sandboxes" : "setup");
      } catch (e) {
        console.error("status check failed", e);
        setView("setup");
      }
    })();
  }, []);

  if (view === "loading") {
    return <div className="loading">Checking your setup…</div>;
  }

  return (
    <div className="app">
      <Sidebar view={view} onNavigate={setView} status={status} />
      <div className="content">
        {view === "setup" ? (
          <Wizard status={status} onFinished={() => setView("sandboxes")} onRefresh={refreshStatus} />
        ) : (
          <CommandCenter
            status={status}
            onOpenSetup={() => setView("setup")}
            onRefreshStatus={refreshStatus}
          />
        )}
      </div>
    </div>
  );
}

import logo from "./assets/logo.png";

function Sidebar({
  view,
  onNavigate,
  status,
}: {
  view: View;
  onNavigate: (v: View) => void;
  status: SetupStatus | null;
}) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <img src={logo} alt="Sandy logo" />
        <span>Sandy</span>
      </div>
      <button
        className="new-task"
        onClick={() => onNavigate("setup")}
        disabled={view === "setup"}
      >
        Run setup
      </button>
      <nav className="nav">
        <button
          className={`nav-item ${view === "setup" ? "active" : ""}`}
          onClick={() => onNavigate("setup")}
        >
          <span className="icon">⚙</span> Setup
        </button>
        <button
          className={`nav-item ${view === "sandboxes" ? "active" : ""}`}
          onClick={() => onNavigate("sandboxes")}
        >
          <span className="icon">▤</span> Sandboxes
          {status && (
            <span className="muted" style={{ marginLeft: "auto", fontSize: 12 }}>
              {status.sbx_installed && status.signed_in ? "" : "!"}
            </span>
          )}
        </button>
      </nav>
      <div className="spacer" />
      <div className="foot">
        <div className="nav-item" style={{ cursor: "default" }}>
          <span className="icon">♡</span> Sandy
        </div>
      </div>
    </aside>
  );
}
