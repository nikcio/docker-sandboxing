import { useCallback, useEffect, useRef, useState } from "react";
import type { SetupStatus } from "./lib/sandbox";
import { getStatus } from "./lib/sandbox";
import Icon from "./common/components/Icon";
import type { IconName } from "./common/components/Icon";
import logo from "./assets/logo.png";
import DashboardPage from "./features/dashboard/components/DashboardPage";
import TemplatesPage from "./features/templates/components/TemplatesPage";
import CreatePage from "./features/create/components/CreatePage";
import SettingsPage from "./features/settings/components/SettingsPage";

type View = "home" | "sandboxes" | "templates" | "settings" | "create";
const navigation: {
  id: Exclude<View, "create">;
  icon: IconName;
  label: string;
}[] = [
  { id: "home", icon: "home", label: "Home" },
  { id: "sandboxes", icon: "box", label: "Sandboxes" },
  { id: "templates", icon: "template", label: "Templates" },
  { id: "settings", icon: "settings", label: "Settings" },
];

export default function App() {
  const [view, setView] = useState<View>("home");
  const [loading, setLoading] = useState(true);
  const [status, setStatus] = useState<SetupStatus | null>(null);
  const [statusError, setStatusError] = useState<string | null>(null);
  const [setupOpen, setSetupOpen] = useState(false);
  const [setupSession, setSetupSession] = useState(0);
  const [checkingSetup, setCheckingSetup] = useState(false);
  const [busy, setBusy] = useState(false);
  const [count, setCount] = useState<number | null>(null);
  const openingSetup = useRef(false);
  const completedSteps = useRef(
    new Map<"sbx_installed" | "signed_in" | "image_paste_on" | "ssh_set_up", number>(),
  );

  const refreshStatus = useCallback(async () => {
    try {
      const next = await getStatus();
      // Successful commands are authoritative while the daemon's checks settle.
      // Drop each confirmation once observed, or after a short grace period, so
      // later logout/configuration changes still appear accurately.
      for (const [key, expires] of completedSteps.current) {
        if (next[key] === true || Date.now() >= expires) {
          completedSteps.current.delete(key);
        } else {
          next[key] = true;
        }
      }
      // A temporarily unavailable check must not erase a previously verified result.
      setStatus((previous) =>
        previous && next.sbx_installed
          ? {
              ...next,
              signed_in: next.signed_in ?? previous.signed_in,
              image_paste_on: next.image_paste_on ?? previous.image_paste_on,
              ssh_set_up: next.ssh_set_up ?? previous.ssh_set_up,
            }
          : next,
      );
      setStatusError(null);
      return next;
    } catch (error) {
      setStatusError(String(error));
      return null;
    }
  }, []);

  useEffect(() => {
    let active = true;
    void refreshStatus().then((next) => {
      if (!active) return;
      if (!next) {
        setView("settings");
      } else if (
        [next.sbx_installed, next.signed_in, next.image_paste_on, next.ssh_set_up].some(
          (value) => value === false,
        )
      ) {
        setView("settings");
        setSetupOpen(true);
      }
      setLoading(false);
    });
    return () => {
      active = false;
    };
  }, [refreshStatus]);

  useEffect(() => {
    if (busy) return;
    const onFocus = () => {
      void refreshStatus();
    };
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [busy, refreshStatus]);

  const openSetup = async () => {
    if (openingSetup.current || busy) return;
    openingSetup.current = true;
    setCheckingSetup(true);
    try {
      const next = await refreshStatus();
      if (!next) return;
      setSetupSession((value) => value + 1);
      setSetupOpen(true);
    } finally {
      openingSetup.current = false;
      setCheckingSetup(false);
    }
  };

  const finishSetup = () => {
    setSetupOpen(false);
    setView("home");
  };

  const onStepDone = (step: string) => {
    const key = (
      {
        install: "sbx_installed",
        login: "signed_in",
        imagepaste: "image_paste_on",
        ssh: "ssh_set_up",
      } as const
    )[step as "install" | "login" | "imagepaste" | "ssh"];
    if (key) {
      completedSteps.current.set(key, Date.now() + 30_000);
      setStatus((previous) => (previous ? { ...previous, [key]: true } : previous));
    }
  };

  if (loading) {
    return (
      <div className="loading" role="status">
        <img src={logo} alt="" />
        <span className="spinner" /> Getting Sandy ready…
      </div>
    );
  }

  return (
    <div className="shell">
      <header className="topbar">
        <div className="brand">
          <img src={logo} alt="" />
          <span>Sandy</span>
        </div>
        <div className="topbar-right">
          <a
            className="icon-button"
            href="https://docs.docker.com/ai/sandboxes/"
            target="_blank"
            rel="noreferrer"
            aria-label="Docker Sandboxes help"
          >
            <Icon name="help" />
          </a>
          <button
            className="account-button"
            disabled={busy || checkingSetup}
            onClick={() => {
              setView("settings");
              void refreshStatus();
            }}
            aria-label="Account settings"
          >
            <span className="avatar-chip">
              <Icon name="user" size={18} />
            </span>
            <Icon name="chevron" size={16} />
          </button>
        </div>
      </header>
      <div className="body">
        <aside className="sidebar">
          <nav className="nav" aria-label="Main navigation">
            {navigation.map((item) => (
              <button
                key={item.id}
                className={`nav-item ${
                  view === item.id || (view === "create" && item.id === "home") ? "active" : ""
                }`}
                aria-current={
                  view === item.id || (view === "create" && item.id === "home") ? "page" : undefined
                }
                disabled={busy || checkingSetup}
                onClick={() => {
                  setView(item.id);
                  if (item.id === "settings") void refreshStatus();
                }}
              >
                <Icon name={item.icon} />
                <span>{item.label}</span>
                {item.id === "sandboxes" && count !== null && (
                  <span className="nav-count">{count}</span>
                )}
              </button>
            ))}
          </nav>
          <div className="help-card">
            <div className="help-title">
              <Icon name="bulb" size={19} /> Need help?
            </div>
            <p>Check out our guide on how to use sandboxes.</p>
            <a href="https://docs.docker.com/ai/sandboxes/" target="_blank" rel="noreferrer">
              View guide <Icon name="arrow" size={15} />
            </a>
          </div>
        </aside>
        <main className="content" id="main-content">
          {(view === "home" || view === "sandboxes") && (
            <DashboardPage
              home={view === "home"}
              status={status}
              onGoSettings={() => setView("settings")}
              onGoTemplates={() => setView("templates")}
              onCreate={() => setView("create")}
              onRefreshStatus={refreshStatus}
              onCount={setCount}
            />
          )}
          {view === "templates" && (
            <TemplatesPage
              onCreate={() => setView("create")}
              onLaunched={() => {
                void refreshStatus();
              }}
              onBusyChange={setBusy}
            />
          )}
          {view === "create" && (
            <CreatePage
              onClose={() => setView("home")}
              onCreated={() => {
                void refreshStatus();
              }}
              onTemplates={() => setView("templates")}
              onBusyChange={setBusy}
            />
          )}
          <div hidden={view !== "settings"}>
            <SettingsPage
              status={status}
              statusError={statusError}
              busy={busy}
              checkingSetup={checkingSetup}
              setupOpen={setupOpen}
              setupSession={setupSession}
              active={view === "settings"}
              onRefreshStatus={refreshStatus}
              onOpenSetup={openSetup}
              onFinishSetup={finishSetup}
              onBusyChange={setBusy}
              onStepDone={onStepDone}
            />
          </div>
        </main>
      </div>
    </div>
  );
}
