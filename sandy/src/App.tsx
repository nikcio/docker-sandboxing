import { useCallback, useEffect, useRef, useState } from "react";
import { SetupStatus, getStatus } from "./api";
import Wizard, { isSetupComplete } from "./Wizard";
import CommandCenter from "./CommandCenter";
import Icon, { IconName } from "./Icon";
import logo from "./assets/logo.png";

type View = "home" | "sandboxes" | "templates" | "settings";
const navigation: { id: View; icon: IconName; label: string }[] = [
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
  const [notice, setNotice] = useState<string | null>(null);
  const dialog = useRef<HTMLDialogElement>(null);
  const openingSetup = useRef(false);

  const refreshStatus = useCallback(async () => {
    try {
      const next = await getStatus();
      setStatus(next);
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
      if (!isSetupComplete(next)) {
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
    if (notice && !dialog.current?.open) dialog.current?.showModal();
  }, [notice]);

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
            onClick={() => setNotice("Account settings")}
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
                className={`nav-item ${view === item.id ? "active" : ""}`}
                aria-current={view === item.id ? "page" : undefined}
                disabled={busy || checkingSetup}
                onClick={() => setView(item.id)}
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
              onStub={setNotice}
              onCount={setCount}
            />
          )}
          {view === "templates" && (
            <div className="page standalone-page">
              <div className="page-head">
                <h1>Templates</h1>
                <p className="page-sub">A head start for your next idea.</p>
              </div>
              <section className="card template-empty">
                <span className="empty-icon">
                  <Icon name="template" size={32} />
                </span>
                <span className="eyebrow">COMING SOON</span>
                <h2>Your next sandbox, ready to go</h2>
                <p>
                  Browse pre-configured environments and make them your own.
                  The template library is on its way.
                </p>
                <a
                  className="button ghost"
                  href="https://github.com/nikcio/docker-sandboxing/tree/main/examples"
                  target="_blank"
                  rel="noreferrer"
                >
                  Explore environment examples <Icon name="arrow" size={17} />
                </a>
              </section>
            </div>
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
              {setupOpen ? (
                <Wizard
                  key={setupSession}
                  status={status}
                  onRefresh={refreshStatus}
                  onDone={finishSetup}
                  onBusyChange={setBusy}
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
                      {isSetupComplete(status)
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
            </div>
          </div>
        </main>
      </div>
      <dialog
        ref={dialog}
        className="feature-dialog"
        onClose={() => setNotice(null)}
        onClick={(event) => {
          if (event.target === event.currentTarget) dialog.current?.close();
        }}
        aria-labelledby="feature-title"
        aria-describedby="feature-description"
      >
        <button
          className="icon-button dialog-close"
          aria-label="Close"
          onClick={() => dialog.current?.close()}
        >
          <Icon name="close" />
        </button>
        <span className="empty-icon">
          <Icon name="sparkles" size={30} />
        </span>
        <span className="eyebrow">COMING SOON</span>
        <h2 id="feature-title">{notice}</h2>
        <p id="feature-description">
          This feature isn’t available in Sandy yet. For now, you can manage
          your sandboxes with the sbx CLI.
        </p>
        <button className="primary" onClick={() => dialog.current?.close()}>
          Got it
        </button>
      </dialog>
    </div>
  );
}
