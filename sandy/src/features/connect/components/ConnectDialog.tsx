import { useCallback, useEffect, useRef, useState } from "react";
import type { ConnectionInfo, SandboxInfo, SetupStatus } from "../../../lib/sandbox";
import { connectionInfo, setupSsh } from "../../../lib/sandbox";
import { stateKind } from "../../../common/lib/sandboxState";
import Icon from "../../../common/components/Icon";
import { copyText } from "../../../common/lib/clipboard";
import { ConnectionMethods } from "./ConnectionMethods";
import { SshStatus } from "./SshStatus";
import { EditorSteps } from "./EditorSteps";
import { ServerInfo } from "./ServerInfo";
import type { Method } from "./ConnectionMethods";
import type { Feedback } from "./ServerInfo";

export interface ConnectDialogProps {
  sandbox: SandboxInfo;
  status: SetupStatus | null;
  onCheckSetup: () => Promise<SetupStatus | null>;
  onOpenTerminal: () => Promise<void>;
  onClose: () => void;
}

export default function ConnectDialog({
  sandbox,
  status,
  onCheckSetup,
  onOpenTerminal,
  onClose,
}: ConnectDialogProps) {
  const dialog = useRef<HTMLDialogElement>(null);
  const terminalButton = useRef<HTMLButtonElement>(null);
  const operation = useRef(false);
  const infoRequest = useRef(0);
  const [method, setMethod] = useState<Method>("terminal");
  const [info, setInfo] = useState<ConnectionInfo | null>(null);
  const [infoError, setInfoError] = useState<string | null>(null);
  const [loadingInfo, setLoadingInfo] = useState(true);
  const [busy, setBusy] = useState<"terminal" | "setup" | "check" | null>(null);
  const [feedback, setFeedback] = useState<Feedback | null>(null);
  const [sshReady, setSshReady] = useState<boolean | null>(status?.ssh_set_up ?? null);
  const sshWarning = status?.warnings?.[0] ?? null;
  const running = stateKind(sandbox.state) === "running";

  useEffect(() => {
    const node = dialog.current;
    node?.showModal();
    // The primary action, rather than the close button, is the default target.
    if (!terminalButton.current?.disabled) terminalButton.current?.focus();
    return () => node?.close();
  }, []);

  useEffect(() => setSshReady(status?.ssh_set_up ?? null), [status?.ssh_set_up]);

  const loadInfo = useCallback(async () => {
    const request = ++infoRequest.current;
    setLoadingInfo(true);
    setInfoError(null);
    try {
      const next = await connectionInfo(sandbox.name);
      if (request === infoRequest.current) setInfo(next);
    } catch (failure) {
      if (request === infoRequest.current) setInfoError(String(failure));
    } finally {
      if (request === infoRequest.current) setLoadingInfo(false);
    }
  }, [sandbox.name]);

  useEffect(() => {
    void loadInfo();
    return () => {
      infoRequest.current += 1;
    };
  }, [loadInfo]);

  const close = () => {
    if (!operation.current) dialog.current?.close();
  };

  const launchTerminal = async () => {
    if (operation.current || !running) return;
    operation.current = true;
    setBusy("terminal");
    setFeedback(null);
    try {
      await onOpenTerminal();
      setFeedback({
        kind: "success",
        text: `Opened your computer’s terminal for ${sandbox.name}.`,
      });
    } catch (failure) {
      setFeedback({ kind: "error", text: `Couldn’t open a terminal: ${String(failure)}` });
    } finally {
      operation.current = false;
      setBusy(null);
      terminalButton.current?.focus();
    }
  };

  const checkSsh = async (configure: boolean) => {
    if (operation.current) return;
    operation.current = true;
    setBusy(configure ? "setup" : "check");
    setFeedback(null);
    try {
      if (configure) await setupSsh();
      const next = await onCheckSetup();
      const ready = next?.ssh_set_up ?? null;
      setSshReady(ready);
      if (ready === true) {
        setFeedback({
          kind: "success",
          text: "SSH is ready. Follow the editor’s connection steps below.",
        });
        await loadInfo();
      } else {
        setFeedback({
          kind: "error",
          text:
            ready === false
              ? "SSH setup is still reported as incomplete. Set up SSH and check again."
              : "Sandy couldn’t verify SSH setup. Check setup again to confirm it is ready.",
        });
      }
    } catch (failure) {
      setFeedback({
        kind: "error",
        text: `Couldn’t ${configure ? "set up" : "check"} SSH: ${String(failure)}`,
      });
    } finally {
      operation.current = false;
      setBusy(null);
    }
  };

  const copy = async (text: string, label: string) => {
    try {
      await copyText(text);
      setFeedback({ kind: "success", text: `${label} copied.` });
    } catch (failure) {
      setFeedback({ kind: "error", text: `Couldn’t copy: ${String(failure)}` });
    }
  };

  return (
    <dialog
      ref={dialog}
      className="connect-dialog"
      aria-labelledby="connect-title"
      aria-describedby="connect-description"
      onClose={(event) => {
        // StrictMode can reopen the dialog before an earlier close event arrives.
        if (!event.currentTarget.open) onClose();
      }}
      onCancel={(event) => {
        event.preventDefault();
        close();
      }}
      onClick={(event) => {
        if (event.target !== event.currentTarget) return;
        const bounds = event.currentTarget.getBoundingClientRect();
        if (
          event.clientX < bounds.left ||
          event.clientX > bounds.right ||
          event.clientY < bounds.top ||
          event.clientY > bounds.bottom
        )
          close();
      }}
    >
      <button
        className="icon-button dialog-close"
        aria-label="Close connection dialog"
        disabled={!!busy}
        onClick={close}
      >
        <Icon name="close" />
      </button>
      <span className="eyebrow">CONNECT TO SANDBOX</span>
      <h2 id="connect-title">Open {sandbox.name}</h2>
      <p id="connect-description">Use your native terminal or connect your editor over SSH.</p>

      <ConnectionMethods
        method={method}
        busy={busy !== null}
        onSelect={(next) => {
          setMethod(next);
          setFeedback(null);
        }}
      />

      {!running && (
        <p className="connection-note">
          This sandbox is not running. Close this dialog and use <strong>Start</strong> in its
          details before connecting.
        </p>
      )}

      <section
        id="connection-method"
        className="connection-method"
        aria-labelledby="connection-method-title"
      >
        {method === "terminal" ? (
          <>
            <h3 id="connection-method-title">Your sandbox, in a terminal</h3>
            <p>
              Open a Bash session in your computer’s terminal. This uses the sandbox CLI and does
              not require SSH setup.
            </p>
            <button
              ref={terminalButton}
              className="primary"
              disabled={!!busy || !running}
              onClick={() => void launchTerminal()}
            >
              {busy === "terminal" ? (
                <span className="spinner" aria-hidden="true" />
              ) : (
                <Icon name="terminal" size={19} />
              )}
              {busy === "terminal" ? "Opening terminal…" : "Open Terminal"}
            </button>
          </>
        ) : (
          <>
            <h3 id="connection-method-title">
              Connect with {method === "opencode" ? "OpenCode Desktop" : "VS Code"}
            </h3>
            <SshStatus
              sshReady={sshReady}
              warning={sshWarning}
              busy={busy === "setup" ? "setup" : busy === "check" ? "check" : null}
              sbxInstalled={!!status?.sbx_installed}
              onSetup={() => void checkSsh(true)}
              onCheck={() => void checkSsh(false)}
            />
            <EditorSteps method={method} info={info} sandboxName={sandbox.name} />
          </>
        )}
      </section>

      <ServerInfo
        info={info}
        loading={loadingInfo}
        error={infoError}
        busy={busy !== null}
        onReload={() => void loadInfo()}
        onCopy={copy}
      />

      <div className="connection-feedback" aria-live="polite" aria-atomic="true">
        {feedback && (
          <p className={feedback.kind} role={feedback.kind === "error" ? "alert" : "status"}>
            <Icon name={feedback.kind === "success" ? "check" : "help"} size={19} />
            {feedback.text}
          </p>
        )}
      </div>
    </dialog>
  );
}
