import { useEffect, useState } from "react";
import type { SandboxInfo, SetupStatus } from "../../../lib/sandbox";
import Icon from "../../../common/components/Icon";
import { Toast } from "../../../common/components/Toast";
import { SandboxList } from "./SandboxList";
import { HomeActions } from "./HomeActions";
import { SandboxInspector } from "./SandboxInspector";
import type { SandboxAction } from "../sandboxActions";
import { stateKind } from "../../../common/lib/sandboxState";
import { summarize, useSandboxDashboard } from "../hooks/useSandboxDashboard";
import ConnectDialog from "../../connect/components/ConnectDialog";
import { ComingSoonDialog } from "../../../common/components/ComingSoonDialog";

export interface DashboardPageProps {
  home: boolean;
  status: SetupStatus | null;
  onGoSettings: () => void;
  onGoTemplates: () => void;
  onCreate: () => void;
  onCheckSetup: () => Promise<SetupStatus | null>;
  onCount: (count: number) => void;
}

const actionLabels: Record<SandboxAction, string> = {
  start: "Starting…",
  stop: "Stopping…",
  restart: "Restarting…",
  terminal: "Opening terminal…",
};

export default function DashboardPage({
  home,
  status,
  onGoSettings,
  onGoTemplates,
  onCreate,
  onCheckSetup,
  onCount,
}: DashboardPageProps) {
  const [selected, setSelected] = useState<string | null>(null);
  const [inspectorOpen, setInspectorOpen] = useState(false);
  const [connecting, setConnecting] = useState<SandboxInfo | null>(null);
  const [search, setSearch] = useState("");
  const [comingSoonOpen, setComingSoonOpen] = useState(false);
  const installed = !!status?.sbx_installed;
  const {
    sandboxes,
    error,
    refreshing,
    pending,
    notice,
    dismissNotice,
    refresh,
    runAction,
    isActionInFlight,
  } = useSandboxDashboard({ installed, onCount });

  useEffect(() => {
    // Keep the selection on a sandbox that still exists after a refresh.
    setSelected((current) =>
      (sandboxes ?? []).some((item) => item.name === current)
        ? current
        : (sandboxes?.[0]?.name ?? null),
    );
  }, [sandboxes]);

  const selectSandbox = (name: string) => {
    setSelected(name);
    setInspectorOpen(true);
  };

  const openConnection = (sandbox: SandboxInfo) => {
    if (isActionInFlight(sandbox.name)) return;
    selectSandbox(sandbox.name);
    setConnecting(sandbox);
  };

  const runPrimaryAction = (sandbox: SandboxInfo) => {
    if (stateKind(sandbox.state) === "running") {
      openConnection(sandbox);
    } else {
      void runAction(sandbox.name, "start").catch(() => {});
    }
  };

  const all = sandboxes ?? [];
  const visible = all.filter((sandbox) =>
    `${sandbox.name} ${sandbox.agent} ${sandbox.workspace}`
      .toLowerCase()
      .includes(search.trim().toLowerCase()),
  );
  const { summary } = summarize(all);
  const detail = all.find((sandbox) => sandbox.name === selected) ?? null;
  const detailPending = detail ? pending.get(detail.name) : undefined;
  const detailRunning = detail !== null && stateKind(detail.state) === "running";
  const needsSetup =
    status !== null &&
    [status.sbx_installed, status.signed_in, status.image_paste_on, status.ssh_set_up].some(
      (value) => value === false,
    );

  return (
    <div className={`dashboard ${inspectorOpen ? "" : "inspector-closed"}`}>
      <div className="dashboard-main">
        <div className="page-head dashboard-page-head">
          <div>
            <h1>{home ? "Welcome to Sandy" : "Your Sandboxes"}</h1>
          </div>
          {!home && (
            <button className="primary" onClick={onCreate}>
              <Icon name="create" size={19} /> Create sandbox
            </button>
          )}
        </div>
        {home && (
          <HomeActions
            onCreate={onCreate}
            onGoTemplates={onGoTemplates}
            onLearnMore={() => setComingSoonOpen(true)}
          />
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
        <SandboxList
          sandboxes={sandboxes}
          visible={visible}
          error={error}
          refreshing={refreshing}
          installed={installed}
          selected={selected}
          pending={pending}
          search={search}
          onSearch={setSearch}
          onRefresh={() => void refresh()}
          onSelect={selectSandbox}
          onOpen={runPrimaryAction}
          onCreate={onCreate}
          summary={summary}
        />
      </div>
      {inspectorOpen && (
        <SandboxInspector
          detail={detail}
          pending={detailPending}
          running={detailRunning}
          installed={installed}
          onAction={(action) => detail && void runAction(detail.name, action).catch(() => {})}
          onOpen={() => detail && runPrimaryAction(detail)}
          onClose={() => setInspectorOpen(false)}
          actionLabels={actionLabels}
        />
      )}
      <div className="dashboard-notifications" aria-live="polite" aria-atomic="true">
        {notice && <Toast notice={notice} onDismiss={dismissNotice} />}
      </div>
      {connecting && (
        <ConnectDialog
          key={connecting.name}
          sandbox={all.find((sandbox) => sandbox.name === connecting.name) ?? connecting}
          status={status}
          onCheckSetup={onCheckSetup}
          onOpenTerminal={() => runAction(connecting.name, "terminal")}
          onClose={() => setConnecting(null)}
        />
      )}
      {comingSoonOpen && (
        <ComingSoonDialog feature="Learn more" onClose={() => setComingSoonOpen(false)} />
      )}
    </div>
  );
}
