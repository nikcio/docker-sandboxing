import { useEffect, useState } from "react";
import { SetupStatus, getStatus } from "./api";
import Wizard from "./Wizard";
import CommandCenter from "./CommandCenter";

type Phase = "loading" | "wizard" | "ready";

export default function App() {
  const [phase, setPhase] = useState<Phase>("loading");
  const [status, setStatus] = useState<SetupStatus | null>(null);

  const refresh = async () => {
    try {
      const s = await getStatus();
      setStatus(s);
      setPhase(s.sbx_installed && s.signed_in ? "ready" : "wizard");
    } catch (e) {
      console.error("status check failed", e);
      setPhase("wizard");
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  if (phase === "loading") {
    return <div className="loading">Checking your setup…</div>;
  }

  return phase === "wizard" ? (
    <Wizard
      status={status}
      onFinished={() => {
        setPhase("ready");
      }}
      onRefresh={refresh}
    />
  ) : (
    <CommandCenter
      status={status}
      onOpenWizard={() => setPhase("wizard")}
      onRefreshStatus={refresh}
    />
  );
}
