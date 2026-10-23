import { useState } from "react";
import type { SetupStatus } from "../../../lib/sandbox";
import Icon from "../../../common/components/Icon";
import { SETUP_STEPS } from "../../setup/setupSteps";
import type { SetupStep } from "../../setup/setupSteps";

interface Props {
  status: SetupStatus | null;
  onCheckSetup: () => Promise<SetupStatus | null>;
  onDone: () => void;
  onBusyChange: (busy: boolean) => void;
  onStepDone?: (step: string) => void;
}

export default function SetupWizard({ status, onCheckSetup, onDone, onBusyChange, onStepDone }: Props) {
  const [current, setCurrent] = useState(() =>
    Math.max(
      0,
      SETUP_STEPS.findIndex((step) => !step.isDone(status)),
    ),
  );
  const [states, setStates] = useState<Record<string, "running" | "done" | "error">>({});
  const [outputs, setOutputs] = useState<Record<string, string>>({});
  const [finished, setFinished] = useState(false);
  const step = SETUP_STEPS[current];
  const busy = Object.values(states).includes("running");
  const isDone = (item: SetupStep) => item.isDone(status) || states[item.id] === "done";
  const completed = SETUP_STEPS.filter(isDone).length;
  const allDone = completed === SETUP_STEPS.length;
  const done = isDone(step);
  const failed = states[step.id] === "error";
  const needsCli = current > 0 && !isDone(SETUP_STEPS[0]);

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
      await onCheckSetup();
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
    if (current === SETUP_STEPS.length - 1) setFinished(true);
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
        <span className="setup-progress-label">{completed} of 4 ready</span>
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
        {SETUP_STEPS.map((item, index) => (
          <li key={item.id}>
            <button
              className={`setup-step ${current === index && !finished ? "current" : ""} ${isDone(item) ? "complete" : ""}`}
              aria-current={current === index && !finished ? "step" : undefined}
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
                  setCurrent(SETUP_STEPS.findIndex((item) => !isDone(item)));
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
              <p className="step-note">Install the CLI in step 1 before running this step.</p>
            ) : null}
          </div>
          {outputs[step.id] && (
            <details className={`step-output ${failed ? "failure" : ""}`} open={failed}>
              <summary>{failed ? "View error details" : "View command output"}</summary>
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
                  {current === SETUP_STEPS.length - 1 ? "Finish setup" : "Continue"}
                  <Icon name="arrow" size={17} />
                </button>
                <button className="text-button" disabled={busy || needsCli} onClick={run}>
                  Run again
                </button>
              </>
            ) : (
              <>
                <button className="primary" disabled={busy || needsCli} onClick={run}>
                  {busy ? "Working…" : failed ? "Try again" : step.action}
                  {!busy && <Icon name="arrow" size={17} />}
                </button>
                <button className="text-button" disabled={busy} onClick={next}>
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
            <Icon name="shield" size={16} /> Your tools. Your computer. You’re in control.
          </span>
          <button className="text-button" disabled={busy} onClick={onDone}>
            Set up later
          </button>
        </div>
      )}
    </section>
  );
}
