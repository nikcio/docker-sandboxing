import { useCallback, useEffect, useRef, useState } from "react";
import type { SandboxInfo } from "../../../lib/sandbox";
import { listSandboxes } from "../../../lib/sandbox";
import { stateKind } from "../../../common/lib/sandboxState";
import { SandboxActions } from "../sandboxActions";
import type { SandboxAction } from "../sandboxActions";

const successLabels: Record<SandboxAction, string> = {
  start: "Started",
  stop: "Stopped",
  restart: "Restarted",
  terminal: "Opened a terminal for",
};

/**
 * The sandbox list data engine: periodic refresh with in-flight dedup, and
 * running per-sandbox actions with toasts. The page component only renders.
 */
export function useSandboxDashboard({
  installed,
  onCount,
}: {
  installed: boolean;
  onCount: (count: number) => void;
}) {
  const [sandboxes, setSandboxes] = useState<SandboxInfo[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [pending, setPending] = useState(() => new Map<string, SandboxAction>());
  const [notice, setNotice] = useState<{ kind: "success" | "error"; text: string } | null>(null);
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
    if (installed) void refresh();
    return () => {
      generation.current += 1;
    };
  }, [refresh, installed]);

  useEffect(() => {
    if (!installed) return;
    const timer = setInterval(() => void refresh(), 5000);
    const onFocus = () => void refresh(true);
    window.addEventListener("focus", onFocus);
    return () => {
      clearInterval(timer);
      window.removeEventListener("focus", onFocus);
    };
  }, [refresh, installed]);

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

  return {
    sandboxes,
    error,
    refreshing,
    pending,
    notice,
    dismissNotice: () => setNotice(null),
    refresh,
    runAction,
    isActionInFlight: (name: string) => busySandboxes.current.has(name),
  };
}

/** Derived summary counts shared by the list and the inspector. */
export function summarize(sandboxes: SandboxInfo[]) {
  const running = sandboxes.filter((sandbox) => stateKind(sandbox.state) === "running").length;
  const stopped = sandboxes.filter((sandbox) => stateKind(sandbox.state) === "stopped").length;
  const inactive = sandboxes.length - running - stopped;
  return {
    running,
    stopped,
    inactive,
    summary: `${sandboxes.length} sandbox${sandboxes.length === 1 ? "" : "es"} · ${running} running · ${stopped} stopped${inactive ? ` · ${inactive} inactive` : ""}`,
  };
}
