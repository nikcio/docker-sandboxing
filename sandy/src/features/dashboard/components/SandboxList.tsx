import type { SandboxInfo } from "../../../lib/sandbox";
import Icon from "../../../common/components/Icon";
import { Avatar } from "../../../common/components/Avatar";
import { StatePill } from "../../../common/components/StatePill";
import { stateKind } from "../../../common/lib/sandboxState";
import type { SandboxAction } from "../sandboxActions";

export function SandboxList({
  sandboxes,
  visible,
  error,
  refreshing,
  installed,
  selected,
  pending,
  search,
  onSearch,
  onRefresh,
  onSelect,
  onOpen,
  onCreate,
  summary,
}: {
  sandboxes: SandboxInfo[] | null;
  visible: SandboxInfo[];
  error: string | null;
  refreshing: boolean;
  installed: boolean;
  selected: string | null;
  pending: Map<string, SandboxAction>;
  search: string;
  onSearch: (value: string) => void;
  onRefresh: () => void;
  onSelect: (name: string) => void;
  onOpen: (sandbox: SandboxInfo) => void;
  onCreate: () => void;
  summary: string;
}) {
  return (
    <section className="card sandbox-card" aria-labelledby="sandboxes-title">
      <div className="list-head">
        <div>
          <h2 id="sandboxes-title">Your Sandboxes</h2>
          <p className="list-summary" role="status">
            {sandboxes
              ? summary
              : !installed
                ? "Install the CLI to see your sandboxes"
                : error
                  ? "Unable to load sandboxes"
                  : "Finding your sandboxes…"}
          </p>
        </div>
        <div className="list-controls">
          <label className="search-field">
            <Icon name="search" size={19} />
            <input
              aria-label="Search sandboxes"
              placeholder="Search sandboxes…"
              value={search}
              onChange={(event) => onSearch(event.target.value)}
            />
          </label>
          <button
            className="icon-button refresh-button"
            title="Refresh sandboxes"
            aria-label="Refresh sandboxes"
            disabled={refreshing || !installed}
            onClick={onRefresh}
          >
            <Icon name="refresh" size={18} />
          </button>
        </div>
      </div>
      {error && (
        <div className="error-banner" role="alert">
          <strong>Couldn’t refresh your sandboxes.</strong>
          <p>{error}</p>
          <button className="ghost" disabled={refreshing} onClick={onRefresh}>
            Try again
          </button>
        </div>
      )}
      {sandboxes === null && !error && installed && (
        <div className="empty" role="status">
          <span className="spinner" /> Loading your local sandboxes…
        </div>
      )}
      {visible.length === 0 && (sandboxes !== null || !installed) && (
        <div className="empty">
          <span className="empty-icon">
            <Icon name="box" size={32} />
          </span>
          <h3>{search ? "No matching sandboxes" : "A little room for your next big idea"}</h3>
          <p>
            {search
              ? "Try a different name, agent, or workspace."
              : "Create a sandbox from a template and a project. Your local environments will appear here."}
          </p>
          {search && (
            <button className="ghost" onClick={() => onSearch("")}>
              Clear search
            </button>
          )}
          {!search && (
            <button className="primary" onClick={onCreate}>
              Create sandbox
            </button>
          )}
        </div>
      )}
      <div className="sbx-rows">
        {visible.map((sandbox) => (
          <div
            key={sandbox.name}
            className={`sbx-row ${selected === sandbox.name ? "selected" : ""}`}
            aria-busy={pending.has(sandbox.name)}
          >
            <button
              className="sandbox-select"
              aria-label={`Show details for ${sandbox.name}`}
              aria-pressed={selected === sandbox.name}
              onClick={() => onSelect(sandbox.name)}
            >
              <Avatar name={sandbox.name} />
              <span className="sbx-info">
                <span className="sbx-name">
                  {sandbox.name}
                  {sandbox.agent && <span className="tag">{sandbox.agent}</span>}
                </span>
                <span className="sbx-sub" title={sandbox.workspace}>
                  {sandbox.workspace || "Mountless"}
                </span>
              </span>
              <StatePill state={sandbox.state} />
            </button>
            <button
              className="primary row-action"
              disabled={pending.has(sandbox.name) || !installed}
              aria-label={
                pending.has(sandbox.name)
                  ? undefined
                  : stateKind(sandbox.state) === "running"
                    ? `Open ${sandbox.name}`
                    : `Start ${sandbox.name}`
              }
              onClick={() => onOpen(sandbox)}
            >
              {pending.has(sandbox.name) ? (
                <span className="spinner" aria-hidden="true" />
              ) : stateKind(sandbox.state) === "running" ? (
                "Open"
              ) : (
                "Start"
              )}
            </button>
          </div>
        ))}
      </div>
      {search && sandboxes !== null && (
        <p className="search-results">
          Showing {visible.length} of {sandboxes.length} sandboxes
        </p>
      )}
    </section>
  );
}
