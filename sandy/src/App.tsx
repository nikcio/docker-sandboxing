import { useCallback, useEffect, useState } from "react";
import { SetupStatus, getStatus } from "./api";
import Wizard from "./Wizard";
import CommandCenter from "./CommandCenter";
import logo from "./assets/logo.png";

type View = "loading" | "home" | "sandboxes" | "templates" | "settings";

export default function App() {
  const [view, setView] = useState<View>("loading");
  const [status, setStatus] = useState<SetupStatus | null>(null);
  const [setupNeeded, setSetupNeeded] = useState(false);

  const refreshStatus = useCallback(async () => {
    try {
      setStatus(await getStatus());
    } catch (e) {
      console.error("status check failed", e);
    }
  }, []);

  // First run: check what's already set up. Anything missing -> setup flow.
  useEffect(() => {
    (async () => {
      try {
        const s = await getStatus();
        setStatus(s);
        const complete = s.sbx_installed && s.signed_in && s.image_paste_on && s.ssh_set_up;
        setView(complete ? "home" : "settings");
        setSetupNeeded(!complete);
      } catch (e) {
        console.error("status check failed", e);
        setView("settings");
        setSetupNeeded(true);
      }
    })();
  }, []);

  if (view === "loading") {
    return <div className="loading">Checking your setup…</div>;
  }

  const setupComplete =
    !!status && status.sbx_installed && status.signed_in && status.image_paste_on && status.ssh_set_up;

  return (
    <div className="shell">
      <header className="topbar">
        <div className="brand">
          <img src={logo} alt="Sandy logo" />
          <span>Sandy</span>
        </div>
        <div className="topbar-right">
          <span className="topbar-icon" title="Help">?</span>
          <span className="avatar-chip" title="Docker account">DU</span>
        </div>
      </header>

      <div className="body">
        <aside className="sidebar">
          <nav className="nav">
            <NavItem view={view} id="home" icon="⌂" label="Home" onClick={setView} />
            <NavItem view={view} id="sandboxes" icon="▤" label="Sandboxes" onClick={setView} />
            <NavItem view={view} id="templates" icon="❏" label="Templates" onClick={setView} />
            <NavItem view={view} id="settings" icon="⚙" label="Settings" onClick={setView} />
          </nav>
          <div className="spacer" />
          <div className="help-card">
            <div className="help-title">💡 Need help?</div>
            <p>Check out our guide on how to use sandboxes.</p>
            <a href="https://docs.docker.com/ai/sandboxes/" target="_blank" rel="noreferrer">
              View guide →
            </a>
          </div>
        </aside>

        <main className="content">
          {view === "settings" ? (
            <SettingsView
              status={status}
              setupNeeded={setupNeeded}
              onSetupDone={() => {
                setSetupNeeded(false);
                setView("home");
              }}
              onRefresh={refreshStatus}
            />
          ) : view === "sandboxes" ? (
            <CommandCenter status={status} onRefreshStatus={refreshStatus} />
          ) : view === "templates" ? (
            <TemplatesView />
          ) : (
            <HomeView
              status={status}
              onGoSandboxes={() => setView("sandboxes")}
              onGoSettings={() => setView("settings")}
            />
          )}
        </main>
      </div>
    </div>
  );

  function NavItem({
    view,
    id,
    icon,
    label,
    onClick,
  }: {
    view: View;
    id: View;
    icon: string;
    label: string;
    onClick: (v: View) => void;
  }) {
    return (
      <button
        className={`nav-item ${view === id ? "active" : ""}`}
        onClick={() => onClick(id)}
      >
        <span className="icon">{icon}</span> {label}
      </button>
    );
  }

  function HomeView({
    status,
    onGoSandboxes,
    onGoSettings,
  }: {
    status: SetupStatus | null;
    onGoSandboxes: () => void;
    onGoSettings: () => void;
  }) {
    return (
      <div className="page">
        <div className="page-head">
          <div>
            <h1>
              <span className="wave">👋</span> Welcome to Sandy
            </h1>
            <p className="page-sub">
              Run and manage your Docker sandboxes — safely, easily, and without the command line.
            </p>
          </div>
        </div>

        <div className="action-cards">
          <div className="action-card primary" onClick={() => alert("Creating sandboxes from Sandy is coming in a future version — use sbx run for now.")}>
            <div className="action-icon">▣</div>
            <div className="action-title">Create new sandbox</div>
            <div className="action-desc">Start with a template or choose your own image.</div>
            <div className="action-arrow">→</div>
          </div>
          <div className="action-card" onClick={() => setViewT("templates")}>
            <div className="action-icon blue">❏</div>
            <div className="action-title">From template</div>
            <div className="action-desc">Use a pre-configured environment.</div>
            <div className="action-arrow">→</div>
          </div>
          <div className="action-card" onClick={() => alert("Running custom images is coming in a future version.")}>
            <div className="action-icon green">◇</div>
            <div className="action-title">From image</div>
            <div className="action-desc">Run any Docker image you like.</div>
            <div className="action-arrow">→</div>
          </div>
          <div className="action-card" onClick={() => window.open("https://docs.docker.com/ai/sandboxes/", "_blank")}>
            <div className="action-icon purple">📖</div>
            <div className="action-title">Learn more</div>
            <div className="action-desc">View quick start guide and tips.</div>
            <div className="action-arrow">→</div>
          </div>
        </div>

        <div className="card" style={{ marginTop: 20 }}>
          <div className="home-list-head">
            <div>
              <div className="home-list-title">Your Sandboxes</div>
              <p className="page-sub" style={{ marginBottom: 0 }}>
                {status
                  ? `${status.sbx_installed ? "CLI installed" : "CLI missing"} · ${status.signed_in ? "signed in" : "not signed in"} · image paste ${status.image_paste_on ? "on" : "off"} · SSH ${status.ssh_set_up ? "set up" : "not set up"}`
                  : "Loading…"}
              </p>
            </div>
            <div className="cc-actions">
              <button className="ghost" onClick={onGoSandboxes}>
                View sandboxes
              </button>
              {!setupComplete && (
                <button className="primary" onClick={onGoSettings}>
                  Finish setup
                </button>
              )}
            </div>
          </div>
        </div>
      </div>
    );
  }

  function setViewT(v: View) {
    setView(v);
  }

  function TemplatesView() {
    return (
      <div className="page">
        <div className="page-head">
          <div>
            <h1>Templates</h1>
            <p className="page-sub">
              Pre-configured sandbox environments are coming in a future version.
            </p>
          </div>
        </div>
        <div className="card empty">
          <div className="big">❏</div>
          Templates aren't available yet.
          <br />
          Meanwhile, check out the mixin examples in the{" "}
          <a href="https://github.com/nikcio/docker-sandboxing" target="_blank" rel="noreferrer">
            docker-sandboxing repo
          </a>
          .
        </div>
      </div>
    );
  }

  function SettingsView({
    status,
    setupNeeded,
    onSetupDone,
    onRefresh,
  }: {
    status: SetupStatus | null;
    setupNeeded: boolean;
    onSetupDone: () => void;
    onRefresh: () => void;
  }) {
    return (
      <div className="page">
        <div className="page-head">
          <div>
            <h1>Settings</h1>
            <p className="page-sub">
              {setupNeeded
                ? "Some things still need to be set up on this computer. Work through the steps below."
                : "Re-run any setup step if something changed on this computer."}
            </p>
          </div>
          {!setupNeeded && setupComplete && (
            <button className="ghost" onClick={onSetupDone}>
              Back to home
            </button>
          )}
        </div>
        <Wizard status={status} onRefresh={onRefresh} />
        {setupComplete && (
          <div className="wizard-actions">
            <button className="primary" onClick={onSetupDone}>
              Go to home
            </button>
          </div>
        )}
      </div>
    );
  }
}
