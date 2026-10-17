import { useCallback, useEffect, useState } from "react";
import { SetupStatus, getStatus } from "./api";
import Wizard from "./Wizard";
import CommandCenter from "./CommandCenter";

type Phase = "loading" | "wizard" | "ready";

export default function App() {
  const [phase, setPhase] = useState<Phase>("loading");
  const [status, setStatus] = useState<SetupStatus | null>(null);

  // Only refreshes the status object — never the phase. Phase changes are
  // explicit (wizard finished / Setup button), so refreshing mid-wizard
  // can't unmount it or flip the user back to the command center.
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
        setPhase(s.sbx_installed && s.signed_in ? "ready" : "wizard");
      } catch (e) {
        console.error("status check failed", e);
        setPhase("wizard");
      }
    })();
  }, []);

  if (phase === "loading") {
    return <div className="loading">Checking your setup…</div>;
  }

  return phase === "wizard" ? (
    <Wizard status={status} onFinished={() => setPhase("ready")} onRefresh={refreshStatus} />
  ) : (
    <CommandCenter
      status={status}
      onOpenWizard={() => setPhase("wizard")}
      onRefreshStatus={refreshStatus}
    />
  );
}
