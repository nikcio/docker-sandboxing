import Icon from "../../../common/components/Icon";

export function SshStatus({
  sshReady,
  warning,
  busy,
  sbxInstalled,
  onSetup,
  onCheck,
}: {
  sshReady: boolean | null;
  warning: string | null;
  busy: "setup" | "check" | null;
  sbxInstalled: boolean;
  onSetup: () => void;
  onCheck: () => void;
}) {
  return (
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
        {warning && (
          <p className="connection-ssh-warning">
            <Icon name="bulb" size={15} /> {warning}
          </p>
        )}
        {sshReady !== true && (
          <div className="connection-setup-actions">
            <button className="primary" disabled={!!busy || !sbxInstalled} onClick={onSetup}>
              {busy === "setup" && <span className="spinner" aria-hidden="true" />}
              {busy === "setup" ? "Setting up SSH…" : "Set up SSH"}
            </button>
            <button className="ghost" disabled={!!busy} onClick={onCheck}>
              {busy === "check" ? "Checking…" : "Check SSH setup"}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
