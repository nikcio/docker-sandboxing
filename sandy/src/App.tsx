import { useCallback, useEffect, useRef, useState } from "react";
import { SetupStatus, getStatus } from "./api";
import Wizard, { isSetupComplete } from "./Wizard";
import CommandCenter from "./CommandCenter";
import Icon, { IconName } from "./Icon";
import logo from "./assets/logo.png";
import CreateSandbox from "./CreateSandbox";
import Templates from "./Templates";
import Providers from "./Providers";

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
    new Map<
      "sbx_installed" | "signed_in" | "image_paste_on" | "ssh_set_up",
      number
    >(),
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
    refreshStatus().then((next) => {
      if (!active) return;
      if (!next) {
        setView("settings");
      } else if (
        [
          next.sbx_installed,
          next.signed_in,
          next.image_paste_on,
          next.ssh_set_up,
        ].some((value) => value === false)
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
    const key = ({
      install: "sbx_installed",
      login: "signed_in",
      imagepaste: "image_paste_on",
      ssh: "ssh_set_up",
    } as const)[step as "install" | "login" | "imagepaste" | "ssh"];
    if (key) {
      completedSteps.current.set(key, Date.now() + 30_000);
      setStatus((previous) =>
        previous ? { ...previous, [key]: true } : previous,
      );
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
                  view === item.id || (view === "create" && item.id === "home")
                    ? "active"
                    : ""
                }`}
                aria-current={
                  view === item.id || (view === "create" && item.id === "home")
                    ? "page"
                    : undefined
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
            <a
              href="https://docs.docker.com/ai/sandboxes/"
              target="_blank"
              rel="noreferrer"
            >
              View guide <Icon name="arrow" size={15} />
            </a>
          </div>
        </aside>
        <main className="content" id="main-content">
          {(view === "home" || view === "sandboxes") && (
            <CommandCenter
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
            <Templates
              onCreate={() => setView("create")}
              onLaunched={() => {
                void refreshStatus();
              }}
              onBusyChange={setBusy}
            />
          )}
          {view === "create" && (
            <CreateSandbox
              onClose={() => setView("home")}
              onCreated={() => {
                void refreshStatus();
              }}
              onTemplates={() => setView("templates")}
              onBusyChange={setBusy}
            />
          )}
          <div hidden={view !== "settings"}>
            <div className="page standalone-page settings-page">
              <div className="page-head">
                <h1>Settings</h1>
                <p className="page-sub">
                  Make Sandy feel at home on your computer.
                </p>
              </div>
              {statusError && (
                <div className="error-banner" role="alert">
                  <strong>We couldn’t check your setup.</strong>
                  <p>{statusError}</p>
                  <button
                    className="ghost"
                    disabled={busy || checkingSetup}
                    onClick={refreshStatus}
                  >
                    Try again
                  </button>
                </div>
              )}
              {!!status?.issues?.length && (
                <div className="error-banner" role="status">
                  <strong>Some setup checks are temporarily unavailable.</strong>
                  <p>{status.issues.join("\n")}</p>
                  <button
                    className="ghost"
                    disabled={busy || checkingSetup}
                    onClick={refreshStatus}
                  >
                    Check again
                  </button>
                </div>
              )}
              {setupOpen ? (
                <Wizard
                  key={setupSession}
                  status={status}
                  onRefresh={refreshStatus}
                  onDone={finishSetup}
                  onBusyChange={setBusy}
                  onStepDone={onStepDone}
                />
              ) : (
                <section
                  className="card setup-summary"
                  aria-busy={checkingSetup}
                >
                  <span className="empty-icon green">
                    <Icon name="shield" size={28} />
                  </span>
                  <div>
                    <h2>
                      {status &&
                      [
                        status.signed_in,
                        status.image_paste_on,
                        status.ssh_set_up,
                      ].some((value) => value === null)
                        ? "Setup status is temporarily unavailable"
                        : isSetupComplete(status)
                          ? "You’re all set"
                          : "Let’s get you set up"}
                    </h2>
                    <p>
                      {isSetupComplete(status)
                        ? "Your local sandbox tools are configured and ready to use."
                        : "A few simple steps to get your sandbox tools ready."}
                    </p>
                  </div>
                  <button
                    className="primary"
                    disabled={busy || checkingSetup}
                    onClick={openSetup}
                  >
                    {checkingSetup ? (
                      <>
                        <span className="spinner" /> Checking setup…
                      </>
                    ) : (
                      <>
                        {isSetupComplete(status)
                          ? "Review setup"
                          : "Continue setup"}
                        <Icon name="arrow" size={17} />
                      </>
                    )}
                  </button>
                </section>
              )}
              {!setupOpen && (
                <section className="card settings-info">
                  <h2>About your workspace</h2>
                  <p>
                    Sandy connects to Docker Sandboxes on this computer. Your
                    sandbox data comes directly from the local <code>sbx</code>{" "}
                    CLI.
                  </p>
                  <a
                    href="https://docs.docker.com/ai/sandboxes/"
                    target="_blank"
                    rel="noreferrer"
                  >
                    Docker Sandboxes documentation{" "}
                    <Icon name="arrow" size={15} />
                  </a>
                </section>
              )}
              {view === "settings" && (
                <div className="card settings-info">
                  <Providers disabled={busy || checkingSetup} />
                </div>
              )}
            </div>
          </div>
        </main>
      </div>
    </div>
  );
}
