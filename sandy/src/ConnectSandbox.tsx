import { useCallback, useEffect, useRef, useState } from "react";
import { ConnectionInfo, SandboxInfo, SetupStatus, connectionInfo, setupSsh } from "./api";
import Icon from "./Icon";

export interface ConnectSandboxProps {
  sandbox: SandboxInfo;
  status: SetupStatus | null;
  onRefreshStatus: () => Promise<SetupStatus | null>;
  onOpenTerminal: () => Promise<void>;
  onClose: () => void;
}

type Method = "terminal" | "opencode" | "vscode";
type Feedback = { kind: "success" | "error"; text: string };

async function copyText(text: string) {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }
  // Some native WebViews expose the system clipboard through the editing API.
  const previous = document.activeElement;
  const field = document.createElement("textarea");
  field.value = text;
  field.readOnly = true;
  field.style.position = "fixed";
  field.style.opacity = "0";
  const modal = document.querySelector("dialog[open]");
  (modal ?? document.body).append(field);
  field.select();
  try {
    if (!document.execCommand("copy"))
      throw new Error("Clipboard access is unavailable. Select and copy the text below.");
  } finally {
    field.remove();
    if (previous instanceof HTMLElement) previous.focus();
  }
}

export default function ConnectSandbox({
  sandbox,
  status,
  onRefreshStatus,
  onOpenTerminal,
  onClose,
}: ConnectSandboxProps) {
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
  const running =
    sandbox.state
      .replace(/^[•○]\s*/, "")
      .trim()
      .toLowerCase() === "running";

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
      const next = await onRefreshStatus();
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

  const serverInfo = info
    ? `Host: ${info.host}\nUser: ${info.user}${info.port == null ? "" : `\nPort: ${info.port}`}\nSSH command: ${info.command}`
    : "";

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

      <div className="connection-options" role="group" aria-label="Connection options">
        <button
          className={method === "terminal" ? "active" : ""}
          aria-pressed={method === "terminal"}
          aria-controls="connection-method"
          disabled={!!busy}
          onClick={() => {
            setMethod("terminal");
            setFeedback(null);
          }}
        >
          <Icon name="terminal" size={23} />
          <span>Terminal</span>
          <small>Native terminal</small>
        </button>
        <button
          className={method === "opencode" ? "active" : ""}
          aria-pressed={method === "opencode"}
          aria-controls="connection-method"
          disabled={!!busy}
          onClick={() => {
            setMethod("opencode");
            setFeedback(null);
          }}
        >
          <Icon name="monitor" size={23} />
          <span>OpenCode Desktop</span>
          <small>SSH connection guide</small>
        </button>
        <button
          className={method === "vscode" ? "active" : ""}
          aria-pressed={method === "vscode"}
          aria-controls="connection-method"
          disabled={!!busy}
          onClick={() => {
            setMethod("vscode");
            setFeedback(null);
          }}
        >
          <Icon name="code" size={23} />
          <span>VS Code</span>
          <small>Remote – SSH guide</small>
        </button>
      </div>

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
            <div className={`connection-ssh ${sshReady === true ? "ready" : ""}`}>
              <Icon name={sshReady === true ? "check" : "key"} size={20} />
              <div>
                <strong>
                  {sshReady === true
                    ? "SSH configuration is ready"
                    : sshReady === false
                      ? "Set up SSH before connecting"
                      : "Check your SSH configuration"}
                </strong>
                <p>
                  {sshReady === true
                    ? "Your SSH configuration handles sandbox routing. Use the host and user below."
                    : sshReady === false
                      ? "Run Set up SSH to add sandbox routing to your local SSH configuration (sbx setup ssh). Then follow the editor steps."
                      : "SSH setup hasn’t been verified. Check setup, or run Set up SSH to configure sandbox routing."}
                </p>
                {sshReady !== true && (
                  <div className="connection-setup-actions">
                    <button
                      className="primary"
                      disabled={!!busy || !status?.sbx_installed}
                      onClick={() => void checkSsh(true)}
                    >
                      {busy === "setup" && <span className="spinner" aria-hidden="true" />}
                      {busy === "setup" ? "Setting up SSH…" : "Set up SSH"}
                    </button>
                    <button
                      className="ghost"
                      disabled={!!busy}
                      onClick={() => void checkSsh(false)}
                    >
                      {busy === "check" ? "Checking…" : "Check SSH setup"}
                    </button>
                  </div>
                )}
              </div>
            </div>
            {method === "opencode" ? (
              <ol className="connection-steps">
                <li>
                  Open <strong>OpenCode Desktop</strong> on your computer.
                </li>
                <li>
                  Go to <strong>Settings → Server → Add SSH Server</strong>.
                </li>
                <li>
                  Enter host <code>{info?.host ?? `${sandbox.name}.sbx`}</code> and SSH user{" "}
                  <code>{info?.user ?? "_default_user_"}</code>.
                  {info?.port != null ? (
                    <>
                      {" "}
                      Set the port to <code>{info.port}</code>.
                    </>
                  ) : (
                    <> Leave the port unset to use your SSH configuration.</>
                  )}
                </li>
                <li>Save the server and select it to connect to the sandbox.</li>
              </ol>
            ) : (
              <ol className="connection-steps">
                <li>
                  Install the <strong>Remote – SSH</strong> extension in VS Code.
                </li>
                <li>
                  Click the <strong>&gt;&lt; remote connection button</strong> in the bottom-left
                  corner.
                </li>
                <li>
                  Choose <strong>Connect to Host…</strong> and enter{" "}
                  <code>
                    {info ? `${info.user}@${info.host}` : `_default_user_@${sandbox.name}.sbx`}
                  </code>
                  .
                  {info?.port != null && (
                    <>
                      {" "}
                      Use the SSH command below with <strong>Add New SSH Host…</strong> to include
                      port <code>{info.port}</code>.
                    </>
                  )}
                </li>
                <li>
                  Connect, then use <strong>Open Folder</strong> to choose your project inside the
                  sandbox.
                </li>
              </ol>
            )}
          </>
        )}
      </section>

      <section
        className="connection-server"
        aria-labelledby="connection-server-title"
        aria-busy={loadingInfo}
      >
        <div className="connection-server-head">
          <h3 id="connection-server-title">SSH server information</h3>
          {info && (
            <button
              className="text-button"
              disabled={!!busy}
              onClick={() => void copy(serverInfo, "Server information")}
            >
              <Icon name="clipboard" size={16} /> Copy server info
            </button>
          )}
        </div>
        {loadingInfo && (
          <p className="connection-loading" role="status">
            <span className="spinner" /> Reading connection information…
          </p>
        )}
        {infoError && (
          <div className="error-banner" role="alert">
            <strong>Couldn’t read SSH connection information.</strong>
            <p>{infoError}</p>
            <button
              className="ghost"
              disabled={loadingInfo || !!busy}
              onClick={() => void loadInfo()}
            >
              Try again
            </button>
          </div>
        )}
        {info && (
          <>
            <dl className="connection-info">
              <dt>Host</dt>
              <dd>
                <code>{info.host}</code>
              </dd>
              <dt>SSH user</dt>
              <dd>
                <code>{info.user}</code>
              </dd>
              <dt>Port</dt>
              <dd>{info.port == null ? "From your SSH configuration" : info.port}</dd>
            </dl>
            <div className="connection-command">
              <code>{info.command}</code>
              <button
                className="icon-button"
                aria-label="Copy SSH command"
                title="Copy SSH command"
                disabled={!!busy}
                onClick={() => void copy(info.command, "SSH command")}
              >
                <Icon name="clipboard" size={19} />
              </button>
            </div>
          </>
        )}
      </section>
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
