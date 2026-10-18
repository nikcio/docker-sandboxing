import type { SandboxInfo } from "../../../lib/sandbox";
import Icon from "../../../common/components/Icon";
import { Avatar } from "../../../common/components/Avatar";
import { StatePill } from "../../../common/components/StatePill";
import type { SandboxAction } from "../sandboxActions";

export function SandboxInspector({
  detail,
  pending,
  running,
  installed,
  onAction,
  onOpen,
  onClose,
  actionLabels,
}: {
  detail: SandboxInfo | null;
  pending: SandboxAction | undefined;
  running: boolean;
  installed: boolean;
  onAction: (action: SandboxAction) => void;
  onOpen: () => void;
  onClose: () => void;
  actionLabels: Record<SandboxAction, string>;
}) {
  return (
    <aside className="inspector" aria-label="Sandbox details">
      <section className="card details-card">
        {detail ? (
          <>
            <div className="details-overview">
              <div className="details-head">
                <Avatar name={detail.name} />
                <div>
                  <h2>{detail.name}</h2>
                  <StatePill state={detail.state} />
                </div>
                <button
                  className="icon-button inspector-close"
                  aria-label="Close sandbox details"
                  title="Close sandbox details"
                  onClick={onClose}
                >
                  <Icon name="close" size={18} />
                </button>
              </div>
              <p className="local-label">
                Local sandbox <span>·</span> {detail.agent || "No agent reported"}
              </p>
              <div className="sandbox-actions" aria-busy={!!pending}>
                <button className="primary connect-action" disabled={!!pending || !installed} onClick={onOpen}>
                  {running ? (
                    <>
                      <Icon name="monitor" size={20} />
                      Open sandbox
                    </>
                  ) : (
                    <>
                      <Icon name="terminal" size={19} />
                      Start
                    </>
                  )}
                </button>
                <button
                  className="ghost"
                  disabled={!!pending || !running || !installed}
                  onClick={() => onAction("stop")}
                >
                  <Icon name="stop" size={19} />
                  Stop
                </button>
                <button
                  className="ghost"
                  disabled={!!pending || !running || !installed}
                  onClick={() => onAction("restart")}
                >
                  <Icon name="refresh" size={19} />
                  Restart
                </button>
              </div>
              {pending && (
                <p className="sandbox-pending" role="status">
                  <span className="spinner" /> {actionLabels[pending]}
                </p>
              )}
            </div>
            <div className="details-body">
              <h3>Details</h3>
              <dl className="details-grid">
                <dt>Name</dt>
                <dd>{detail.name}</dd>
                <dt>Agent</dt>
                <dd>{detail.agent || "Not reported"}</dd>
                <dt>Workspace</dt>
                <dd>{detail.workspace || "No workspace"}</dd>
                <dt>Status</dt>
                <dd>
                  <StatePill state={detail.state} />
                </dd>
              </dl>
            </div>
          </>
        ) : (
          <div className="inspector-empty">
            <span className="empty-icon">
              <Icon name="monitor" size={30} />
            </span>
            <h2>A closer look</h2>
            <p>Select a sandbox to see its details and available actions.</p>
          </div>
        )}
      </section>
    </aside>
  );
}
