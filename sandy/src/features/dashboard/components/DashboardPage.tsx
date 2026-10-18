import { useCallback, useEffect, useRef, useState } from "react";
import type { SandboxInfo, SetupStatus } from "../../../lib/sandbox";
import { listSandboxes } from "../../../lib/sandbox";
import { stateKind } from "../../../common/lib/sandboxState";
import Icon from "../../../common/components/Icon";
import { Toast } from "../../../common/components/Toast";
import type { ToastNotice } from "../../../common/components/Toast";
import { SandboxList } from "./SandboxList";
import { HomeActions } from "./HomeActions";
import { SandboxInspector } from "./SandboxInspector";
import { SandboxActions } from "../sandboxActions";
import type { SandboxAction } from "../sandboxActions";
import ConnectDialog from "../../connect/components/ConnectDialog";

export interface DashboardPageProps {
  home: boolean;
  status: SetupStatus | null;
  onGoSettings: () => void;
  onGoTemplates: () => void;
  onCreate: () => void;
  onRefreshStatus: () => Promise<SetupStatus | null>;
  onCount: (count: number) => void;
}

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

export default function DashboardPage({
  home,
  status,
  onGoSettings,
  onGoTemplates,
  onCreate,
  onRefreshStatus,
  onCount,
}: DashboardPageProps) {
  const [sandboxes, setSandboxes] = useState<SandboxInfo[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [selected, setSelected] = useState<string | null>(null);
  const [connecting, setConnecting] = useState<SandboxInfo | null>(null);
  const [pending, setPending] = useState(() => new Map<string, SandboxAction>());
  const [notice, setNotice] = useState<ToastNotice | null>(null);
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
      await SandboxActions[action](name);
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
        {home && <HomeActions onCreate={onCreate} onGoTemplates={onGoTemplates} />}
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
          installed={!!status?.sbx_installed}
          selected={selected}
          pending={pending}
          search={search}
          onSearch={setSearch}
          sort={sort}
          onSort={setSort}
          onRefresh={() => void refresh()}
          onSelect={setSelected}
          onOpen={openConnection}
          onCreate={onCreate}
          summary={`${all.length} sandbox${all.length === 1 ? "" : "es"} · ${running} running · ${stopped} stopped${inactive ? ` · ${inactive} inactive` : ""}`}
        />
      </div>
      <SandboxInspector
        detail={detail}
        pending={detailPending}
        running={detailRunning}
        installed={!!status?.sbx_installed}
        onAction={(action) => detail && void runAction(detail.name, action).catch(() => {})}
        onOpen={() => detail && openConnection(detail)}
        onCreate={onCreate}
        actionLabels={actionLabels}
      />
      <div className="dashboard-notifications" aria-live="polite" aria-atomic="true">
        {notice && <Toast notice={notice} onDismiss={() => setNotice(null)} />}
      </div>
      {connecting && (
        <ConnectDialog
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
