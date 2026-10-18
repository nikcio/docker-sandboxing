import type { ConnectionInfo } from "../../../lib/sandbox";
import Icon from "../../../common/components/Icon";
import { ErrorBanner } from "../../../common/components/ErrorBanner";

export type Feedback = { kind: "success" | "error"; text: string };

export function ServerInfo({
  info,
  loading,
  error,
  busy,
  onReload,
  onCopy,
}: {
  info: ConnectionInfo | null;
  loading: boolean;
  error: string | null;
  busy: boolean;
  onReload: () => void;
  onCopy: (text: string, label: string) => void;
}) {
  const serverInfo = info
    ? `Host: ${info.host}\nUser: ${info.user}${info.port == null ? "" : `\nPort: ${info.port}`}\nSSH command: ${info.command}`
    : "";
  return (
    <section className="connection-server" aria-labelledby="connection-server-title" aria-busy={loading}>
      <div className="connection-server-head">
        <h3 id="connection-server-title">SSH server information</h3>
        {info && (
          <button
            className="text-button"
            disabled={busy}
            onClick={() => onCopy(serverInfo, "Server information")}
          >
            <Icon name="clipboard" size={16} /> Copy server info
          </button>
        )}
      </div>
      {loading && (
        <p className="connection-loading" role="status">
          <span className="spinner" /> Reading connection information…
        </p>
      )}
      {error && (
        <ErrorBanner title="Couldn’t read SSH connection information." onAction={onReload} disabled={loading || busy}>
          <p>{error}</p>
        </ErrorBanner>
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
              disabled={busy}
              onClick={() => onCopy(info.command, "SSH command")}
            >
              <Icon name="clipboard" size={19} />
            </button>
          </div>
        </>
      )}
    </section>
  );
}
