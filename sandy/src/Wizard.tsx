import { useState } from "react";
import { SetupStatus, installSbx, enableImagePaste, setupSsh, login } from "./api";

interface Props {
  status: SetupStatus | null;
  onFinished: () => void;
  onRefresh: () => void;
}

type StepState = "pending" | "active" | "running" | "done" | "error";

interface Step {
  id: string;
  title: string;
  description: string;
}

const STEPS: Step[] = [
  {
    id: "install",
    title: "Install the Docker Sandboxes CLI",
    description:
      "Sandy will install the sbx command-line tool, which creates and manages sandboxes.",
  },
  {
    id: "login",
    title: "Sign in to Docker",
    description:
      "A browser window will open for you to sign in with your Docker account. Sandboxes require sign-in.",
  },
  {
    id: "imagepaste",
    title: "Enable image pasting",
    description:
      "Allows you to paste screenshots from your clipboard into agent chats (Ctrl+V).",
  },
  {
    id: "ssh",
    title: "Set up SSH access",
    description:
      "Configures your SSH client so every sandbox is reachable as <name>.sbx — handy for editors and terminals.",
  },
];

export default function Wizard({ status, onFinished, onRefresh }: Props) {
  const [states, setStates] = useState<Record<string, StepState>>({});
  const [outputs, setOutputs] = useState<Record<string, string>>({});
  const [finished, setFinished] = useState(false);

  const setStep = (id: string, s: StepState, output?: string) => {
    setStates((prev) => ({ ...prev, [id]: s }));
    if (output !== undefined) {
      setOutputs((prev) => ({ ...prev, [id]: output }));
    }
  };

  const stateOf = (id: string): StepState => states[id] ?? "pending";

  const runStep = async (id: string, fn: () => Promise<string | void>) => {
    setStep(id, "running");
    try {
      const out = await fn();
      setStep(id, "done", typeof out === "string" ? out : undefined);
      await onRefresh();
      return true;
    } catch (e) {
      setStep(id, "error", String(e));
      return false;
    }
  };

  const runInstall = () => runStep("install", installSbx);
  const runLogin = () => runStep("login", login);
  const runImagePaste = () => runStep("imagepaste", enableImagePaste);
  const runSsh = () => runStep("ssh", setupSsh);

  const runRemaining = async () => {
    if (stateOf("install") !== "done" && !status?.sbx_installed) {
      const ok = await runInstall();
      if (!ok) return;
    } else if (stateOf("install") === "pending") {
      setStep("install", "done", "Already installed");
    }
    if (stateOf("login") !== "done" && !status?.signed_in) {
      const ok = await runLogin();
      if (!ok) return;
    } else if (stateOf("login") === "pending") {
      setStep("login", "done", "Already signed in");
    }
    if (stateOf("imagepaste") !== "done") {
      const ok = await runImagePaste();
      if (!ok) return;
    }
    if (stateOf("ssh") !== "done") {
      const ok = await runSsh();
      if (!ok) return;
    }
    setFinished(true);
  };

  const allDone =
    (stateOf("install") === "done" || status?.sbx_installed) &&
    (stateOf("login") === "done" || status?.signed_in) &&
    stateOf("imagepaste") === "done" &&
    stateOf("ssh") === "done";

  const busy = Object.values(states).some((s) => s === "running");

  return (
    <main>
      <div className="wizard">
        <h2>Welcome to Sandy</h2>
        <p className="lead">
          Sandy is your command center for Docker Sandboxes — safe, isolated
          environments where AI coding agents do their work. Let's set
          everything up. This only needs to happen once.
        </p>

        {STEPS.map((step, i) => {
          const st = stateOf(step.id);
          const autoDone =
            (step.id === "install" && status?.sbx_installed && st === "pending") ||
            (step.id === "login" && status?.signed_in && st === "pending");
          const cls =
            st === "done" || autoDone
              ? "done"
              : st === "error"
                ? "active error"
                : st === "running"
                  ? "active"
                  : "pending";
          return (
            <div key={step.id} className={`step ${cls}`}>
              <div className="num">
                {st === "done" || autoDone ? "✓" : st === "error" ? "!" : i + 1}
              </div>
              <div className="body">
                <h3>{step.title}</h3>
                <p>{step.description}</p>
                {st === "running" && <p>Working… this may take a few minutes.</p>}
                {st === "error" && outputs[step.id] && (
                  <pre className="error">{outputs[step.id]}</pre>
                )}
                {st === "done" && outputs[step.id] && (
                  <pre>{outputs[step.id]}</pre>
                )}
                {autoDone && <p className="okline">Already done</p>}
              </div>
            </div>
          );
        })}

        <div className="row">
          {(finished || allDone) && (
            <button onClick={onFinished}>Open command center</button>
          )}
          {!finished && !allDone && (
            <button onClick={runRemaining} disabled={busy}>
              {busy ? "Working…" : "Get started"}
            </button>
          )}
          {(finished || allDone) && (
            <button className="secondary" onClick={runRemaining} disabled={busy}>
              Re-run steps
            </button>
          )}
        </div>
      </div>
    </main>
  );
}
