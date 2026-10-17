import { useState } from "react";
import { SetupStatus, installSbx, enableImagePaste, setupSsh, login } from "./api";

interface Props {
  status: SetupStatus | null;
  onFinished: () => void;
  onRefresh: () => void;
}

type StepState = "pending" | "running" | "done" | "error";

interface StepDef {
  id: string;
  kicker: string;
  title: string;
  description: string;
}

const STEPS: StepDef[] = [
  {
    id: "install",
    kicker: "Step 1 of 4",
    title: "Install the Docker Sandboxes CLI",
    description:
      "Sandy will install the sbx command-line tool, which creates and manages sandboxes. This uses your platform's package manager and may take a few minutes.",
  },
  {
    id: "login",
    kicker: "Step 2 of 4",
    title: "Sign in to Docker",
    description:
      "A browser window will open for you to sign in with your Docker account. Sandboxes require sign-in, and Sandy checks the result automatically.",
  },
  {
    id: "imagepaste",
    kicker: "Step 3 of 4",
    title: "Enable image pasting",
    description:
      "Allows you to paste screenshots from your clipboard into agent chats (Ctrl+V). You can withdraw this later from the sandbox settings.",
  },
  {
    id: "ssh",
    kicker: "Step 4 of 4",
    title: "Set up SSH access",
    description:
      "Configures your SSH client so every sandbox is reachable as <name>.sbx — handy for editors and terminals.",
  },
];

export default function Wizard({ status, onFinished, onRefresh }: Props) {
  const [current, setCurrent] = useState(0);
  const [states, setStates] = useState<Record<string, StepState>>({});
  const [outputs, setOutputs] = useState<Record<string, string>>({});

  const step = STEPS[current];
  const stateOf = (id: string): StepState => states[id] ?? "pending";

  const setStep = (id: string, s: StepState, output?: string) => {
    setStates((prev) => ({ ...prev, [id]: s }));
    if (output !== undefined) {
      setOutputs((prev) => ({ ...prev, [id]: output }));
    }
  };

  const runStep = async (id: string, fn: () => Promise<string | void>) => {
    setStep(id, "running");
    try {
      const out = await fn();
      setStep(id, "done", typeof out === "string" ? out : undefined);
      onRefresh();
      return true;
    } catch (e) {
      setStep(id, "error", String(e));
      return false;
    }
  };

  const isAutoDone = (id: string): boolean => {
    if (stateOf(id) === "done") return true;
    if (id === "install") return !!status?.sbx_installed;
    if (id === "login") return !!status?.signed_in;
    return false;
  };

  const runners: Record<string, () => Promise<string | void>> = {
    install: installSbx,
    login: login,
    imagepaste: enableImagePaste,
    ssh: setupSsh,
  };

  const runCurrent = async () => {
    const ok = await runStep(step.id, runners[step.id]);
    if (ok && current < STEPS.length - 1) {
      setCurrent(current + 1);
    }
  };

  const skip = () => {
    if (current < STEPS.length - 1) setCurrent(current + 1);
  };

  const back = () => {
    if (current > 0) setCurrent(current - 1);
  };

  const allDone = STEPS.every((s) => isAutoDone(s.id) || stateOf(s.id) === "done");
  const state = stateOf(step.id);
  const autoDone = isAutoDone(step.id);
  const busy = state === "running";

  const doneCount = STEPS.filter((s) => isAutoDone(s.id) || stateOf(s.id) === "done").length;

  return (
    <div className="wizard">
      <div className="page-head">
        <h1>{allDone ? "You're all set" : "Welcome to Sandy"}</h1>
      </div>
      <p className="page-sub">
        {allDone
          ? "Sandy is your command center for Docker Sandboxes. You can re-run any setup step later from here."
          : "Sandy is your command center for Docker Sandboxes — safe, isolated environments where AI coding agents do their work. Let's set everything up."}
      </p>

      <div className="progress-track">
        <div
          className="progress-fill"
          style={{ width: `${(doneCount / STEPS.length) * 100}%` }}
        />
      </div>

      <div className="step-card">
        <div className="step-kicker">{step.kicker}</div>
        <h2>{step.title}</h2>
        <p className="desc">{step.description}</p>

        {busy && (
          <div className="step-note">
            <span className="spinner" /> Working… this may take a few minutes.
          </div>
        )}
        {state === "error" && outputs[step.id] && (
          <pre className="error">{outputs[step.id]}</pre>
        )}
        {state === "done" && outputs[step.id] && <pre>{outputs[step.id]}</pre>}
        {autoDone && state !== "done" && (
          <div className="step-note ok">✓ Already done</div>
        )}

        <div className="wizard-actions">
          {current > 0 && (
            <button className="ghost" onClick={back} disabled={busy}>
              Back
            </button>
          )}
          {!autoDone && state !== "done" && (
            <button className="primary" onClick={runCurrent} disabled={busy}>
              {busy ? "Working…" : current === STEPS.length - 1 ? "Finish setup" : "Continue"}
            </button>
          )}
          {!autoDone && state !== "done" && current < STEPS.length - 1 && (
            <button className="ghost" onClick={skip} disabled={busy}>
              Skip for now
            </button>
          )}
          {(autoDone || state === "done") && current < STEPS.length - 1 && (
            <button className="primary" onClick={() => setCurrent(current + 1)}>
              Next
            </button>
          )}
          {(autoDone || state === "done") && current === STEPS.length - 1 && (
            <button className="primary" onClick={onFinished}>
              Open command center
            </button>
          )}
          {(autoDone || state === "done") && (
            <button
              className="ghost"
              onClick={() => runStep(step.id, runners[step.id])}
              disabled={busy}
            >
              Re-run
            </button>
          )}
          {state === "error" && (
            <button className="ghost" onClick={skip} disabled={busy}>
              Skip for now
            </button>
          )}
        </div>
      </div>

      <div className="step-dots">
        {STEPS.map((s, i) => {
          const st = isAutoDone(s.id) || stateOf(s.id) === "done" ? "done" : i === current ? "current" : "";
          return (
            <span
              key={s.id}
              className={`dot ${st}`}
              style={{ cursor: "pointer" }}
              onClick={() => !busy && setCurrent(i)}
              title={s.title}
            />
          );
        })}
      </div>
    </div>
  );
}
