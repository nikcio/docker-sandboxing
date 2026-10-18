import { useState } from "react";
import {
  SetupStatus,
  installSbx,
  enableImagePaste,
  setupSsh,
  login,
} from "./api";
import Icon, { IconName } from "./Icon";

interface Props {
  status: SetupStatus | null;
  onRefresh: () => Promise<SetupStatus | null>;
  onDone: () => void;
  onBusyChange: (busy: boolean) => void;
  onStepDone?: (step: string) => void;
}

interface StepDef {
  id: string;
  label: string;
  title: string;
  description: string;
  command: string;
  action: string;
  icon: IconName;
  isDone: (status: SetupStatus | null) => boolean;
  run: () => Promise<string | void>;
}

const STEPS: StepDef[] = [
  {
    id: "install",
    label: "Install CLI",
    title: "A small tool. A world of possibilities.",
    description:
      "Sandy uses the Docker Sandboxes CLI (sbx) to work with your local sandboxes. We’ll install it with your system’s package manager. You may be asked for administrator permission.",
    command: "Install sbx using Homebrew, WinGet, or apt for your platform",
    action: "Install CLI",
    icon: "terminal",
    isDone: (status) => !!status?.sbx_installed,
    run: installSbx,
  },
  {
    id: "login",
    label: "Docker account",
    title: "Let’s get you signed in",
    description:
      "Connect your Docker account to use Sandboxes. We’ll open your browser so you can sign in, then check the result here. Already signed in? Sandy will pick that up automatically.",
    command: "sbx login",
    action: "Sign in to Docker",
    icon: "user",
    isDone: (status) => !!status?.signed_in,
    run: login,
  },
  {
    id: "imagepaste",
    label: "Image pasting",
    title: "Give your agent the full picture",
    description:
      "Paste screenshots and images from your clipboard directly into agent chats. This enables clipboard image pasting for your local sandboxes.",
    command: "sbx settings set clipboard.imagePaste true",
    action: "Enable image pasting",
    icon: "clipboard",
    isDone: (status) => !!status?.image_paste_on,
    run: enableImagePaste,
  },
  {
    id: "ssh",
    label: "SSH access",
    title: "Your favorite tools, connected",
    description:
      "Configure your SSH client to reach sandboxes at <name>.sbx. This lets you connect from your editor or terminal using the tools you already love.",
    command: "sbx setup ssh",
    action: "Set up SSH",
    icon: "key",
    isDone: (status) => !!status?.ssh_set_up,
    run: setupSsh,
  },
];

export function isSetupComplete(status: SetupStatus | null) {
  return STEPS.every((step) => step.isDone(status));
}

export default function Wizard({
  status,
  onRefresh,
  onDone,
  onBusyChange,
  onStepDone,
}: Props) {
  const [current, setCurrent] = useState(() =>
    Math.max(
      0,
      STEPS.findIndex((step) => !step.isDone(status)),
    ),
  );
  const [states, setStates] = useState<
    Record<string, "running" | "done" | "error">
  >({});
  const [outputs, setOutputs] = useState<Record<string, string>>({});
  const [finished, setFinished] = useState(false);
  const step = STEPS[current];
  const busy = Object.values(states).includes("running");
  const isDone = (item: StepDef) =>
    item.isDone(status) || states[item.id] === "done";
  const completed = STEPS.filter(isDone).length;
  const allDone = completed === STEPS.length;
  const done = isDone(step);
  const failed = states[step.id] === "error";
  const needsCli = current > 0 && !isDone(STEPS[0]);

  const run = async () => {
    if (busy) return;
    setStates((previous) => ({ ...previous, [step.id]: "running" }));
    setOutputs((previous) => ({ ...previous, [step.id]: "" }));
    onBusyChange(true);
    try {
      const output = await step.run();
      setOutputs((previous) => ({
        ...previous,
        [step.id]: typeof output === "string" ? output : "",
      }));
      await onRefresh();
      onStepDone?.(step.id);
      setStates((previous) => ({ ...previous, [step.id]: "done" }));
    } catch (error) {
      setStates((previous) => ({ ...previous, [step.id]: "error" }));
      setOutputs((previous) => ({ ...previous, [step.id]: String(error) }));
    } finally {
      onBusyChange(false);
    }
  };

  const next = () => {
    if (current === STEPS.length - 1) setFinished(true);
    else setCurrent((value) => value + 1);
  };

  return (
    <section className="setup-flow" aria-labelledby="setup-title">
      <div className="setup-heading">
        <div>
          <span className="eyebrow">YOUR LOCAL WORKSPACE</span>
          <h2 id="setup-title">
            {finished
              ? allDone
                ? "You’re ready to make something great"
                : "Pick up where you left off"
              : "A warm welcome to Sandy"}
          </h2>
          <p>
            {finished
              ? allDone
                ? "Your tools are configured. Let’s take a look at your sandboxes."
                : "You can use Sandy now and finish the remaining steps in Settings."
              : "A few simple steps, and you’ll be right at home. We’ve checked what’s already configured."}
          </p>
        </div>
        <span className="setup-progress-label">
          {completed} of 4 ready
        </span>
      </div>
      <div
        className="progress-track"
        role="progressbar"
        aria-label="Setup progress"
        aria-valuemin={0}
        aria-valuemax={4}
        aria-valuenow={completed}
      >
        <span style={{ width: `${(completed / 4) * 100}%` }} />
      </div>
      <ol className="setup-steps" aria-label="Setup steps">
        {STEPS.map((item, index) => (
          <li key={item.id}>
            <button
              className={`setup-step ${current === index && !finished ? "current" : ""} ${isDone(item) ? "complete" : ""}`}
              aria-current={
                current === index && !finished ? "step" : undefined
              }
              disabled={busy}
              onClick={() => {
                setCurrent(index);
                setFinished(false);
              }}
            >
              <span className="step-number">
                {isDone(item) ? <Icon name="check" size={16} /> : index + 1}
              </span>
              <span>{item.label}</span>
            </button>
          </li>
        ))}
      </ol>
      {finished ? (
        <div className="card setup-finish">
          <span className={`empty-icon ${allDone ? "green" : ""}`}>
            <Icon name={allDone ? "check" : "shield"} size={34} />
          </span>
          <h3>
            {allDone
              ? "All set. All yours."
              : `${4 - completed} step${4 - completed === 1 ? "" : "s"} left for later`}
          </h3>
          <p>
            {allDone
              ? "Your sandbox workspace is ready to explore."
              : "Missing steps will still be here whenever you’re ready."}
          </p>
          <div className="wizard-actions">
            <button className="primary" onClick={onDone}>
              Go to home <Icon name="arrow" size={18} />
            </button>
            {!allDone && (
              <button
                className="ghost"
                onClick={() => {
                  setCurrent(STEPS.findIndex((item) => !isDone(item)));
                  setFinished(false);
                }}
              >
                Continue setup
              </button>
            )}
          </div>
        </div>
      ) : (
        <div className="card step-card" aria-busy={busy}>
          <span className="empty-icon">
            <Icon name={step.icon} size={30} />
          </span>
          <span className="eyebrow">STEP {current + 1} OF 4</span>
          <h3>{step.title}</h3>
          <p className="step-description">{step.description}</p>
          <div className="command-preview">
            <Icon name="terminal" size={17} />
            <code>{step.command}</code>
          </div>
          <div className="step-feedback" aria-live="polite">
            {busy ? (
              <p className="step-note">
                <span className="spinner" />
                {current === 1
                  ? "Finish signing in in your browser. We’ll wait right here."
                  : "Working on it… this may take a few minutes."}
              </p>
            ) : failed ? (
              <p className="step-note failure">
                Something didn’t go through. You can try again below.
              </p>
            ) : done ? (
              <p className="step-note success">
                <Icon name="check" size={18} />
                {states[step.id] === "done"
                  ? "Done — you’re good to go."
                  : "Already set up on this computer. One less thing to do!"}
              </p>
            ) : needsCli ? (
              <p className="step-note">
                Install the CLI in step 1 before running this step.
              </p>
            ) : null}
          </div>
          {outputs[step.id] && (
            <details
              className={`step-output ${failed ? "failure" : ""}`}
              open={failed}
            >
              <summary>
                {failed ? "View error details" : "View command output"}
              </summary>
              <pre>{outputs[step.id]}</pre>
            </details>
          )}
          <div className="wizard-actions">
            {current > 0 && (
              <button
                className="ghost"
                disabled={busy}
                onClick={() => setCurrent((value) => value - 1)}
              >
                Back
              </button>
            )}
            {done && !failed ? (
              <>
                <button className="primary" disabled={busy} onClick={next}>
                  {current === STEPS.length - 1 ? "Finish setup" : "Continue"}
                  <Icon name="arrow" size={17} />
                </button>
                <button
                  className="text-button"
                  disabled={busy || needsCli}
                  onClick={run}
                >
                  Run again
                </button>
              </>
            ) : (
              <>
                <button
                  className="primary"
                  disabled={busy || needsCli}
                  onClick={run}
                >
                  {busy ? "Working…" : failed ? "Try again" : step.action}
                  {!busy && <Icon name="arrow" size={17} />}
                </button>
                <button
                  className="text-button"
                  disabled={busy}
                  onClick={next}
                >
                  Skip for now
                </button>
              </>
            )}
          </div>
        </div>
      )}
      {!finished && (
        <div className="setup-footer">
          <span>
            <Icon name="shield" size={16} /> Your tools. Your computer. You’re
            in control.
          </span>
          <button className="text-button" disabled={busy} onClick={onDone}>
            Set up later
          </button>
        </div>
      )}
    </section>
  );
}
