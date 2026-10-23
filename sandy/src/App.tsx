import { useEffect, useState } from "react";
import { getCurrentWindow, Window } from "@tauri-apps/api/window";
import { Sidebar, Topbar } from "./common/components/AppShell";
import type { View } from "./common/components/AppShell";
import BootSplash from "./common/components/BootSplash";
import { ComingSoonDialog } from "./common/components/ComingSoonDialog";
import { useSetupStatus } from "./common/hooks/useSetupStatus";
import DashboardPage from "./features/dashboard/components/DashboardPage";
import TemplatesPage from "./features/templates/components/TemplatesPage";
import CreatePage from "./features/create/components/CreatePage";
import SettingsPage from "./features/settings/components/SettingsPage";

const revealMainWindow = async () => {
  const splash = (await Window.getAll()).find((window) => window.label === "splash");
  await getCurrentWindow().show();
  // The splash may already be gone (e.g. the user closed it); ignore failures.
  await splash?.close().catch(() => {});
};

export default function App() {
  const [view, setView] = useState<View>("home");
  const [guideDialogOpen, setGuideDialogOpen] = useState(false);
  const [setupOpen, setSetupOpen] = useState(false);
  const [setupSession, setSetupSession] = useState(0);
  const [count, setCount] = useState<number | null>(null);
  const setup = useSetupStatus();
  const {
    status,
    statusError,
    loading,
    busy,
    setBusy,
    checkSetup,
    checkingSetup,
    openSetup,
    onStepDone,
  } = setup;

  // Route to setup when the first setup check finds incomplete configuration.
  useEffect(() => {
    if (loading) return;
    // The main window is ready: reveal it and drop the boot splash.
    void revealMainWindow();
    if (statusError && !status) {
      setView("settings");
      return;
    }
    if (
      status &&
      [status.sbx_installed, status.signed_in, status.image_paste_on, status.ssh_set_up].some(
        (value) => value === false,
      )
    ) {
      setView("settings");
      setSetupOpen(true);
    }
    // Runs once: the first completed setup check decides the landing view.
  }, [loading]);

  const finishSetup = () => {
    setSetupOpen(false);
    setView("home");
  };

  const navigate = (next: Exclude<View, "create">) => {
    setView(next);
  };

  if (loading) {
    return <BootSplash />;
  }

  return (
    <div className="shell">
      <Topbar />
      <div className="body">
        <Sidebar
          view={view}
          busy={busy}
          checkingSetup={checkingSetup}
          count={count}
          onNavigate={navigate}
          onViewGuide={() => setGuideDialogOpen(true)}
        />
        <main className="content" id="main-content">
          {(view === "home" || view === "sandboxes") && (
            <DashboardPage
              home={view === "home"}
              status={status}
              onGoSettings={() => setView("settings")}
              onGoTemplates={() => setView("templates")}
              onCreate={() => setView("create")}
              onCheckSetup={checkSetup}
              onCount={setCount}
            />
          )}
          {view === "templates" && (
            <TemplatesPage
              onCreate={() => setView("create")}
              onBusyChange={setBusy}
            />
          )}
          {view === "create" && (
            <CreatePage
              onClose={() => setView("home")}
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
              onCheckSetup={checkSetup}
              onOpenSetup={() => {
                void openSetup().then((next) => {
                  if (!next) return;
                  setSetupSession((value) => value + 1);
                  setSetupOpen(true);
                });
              }}
              onFinishSetup={finishSetup}
              onBusyChange={setBusy}
              onStepDone={onStepDone}
            />
          </div>
        </main>
      </div>
      {guideDialogOpen && (
        <ComingSoonDialog feature="View guide" onClose={() => setGuideDialogOpen(false)} />
      )}
    </div>
  );
}
