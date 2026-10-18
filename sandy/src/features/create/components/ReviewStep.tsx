import { useId } from "react";
import type { FormEvent } from "react";
import type { KitOption } from "../../../lib/kit";
import { previewTemplate } from "../../../lib/kit";
import type { SaveTemplateRequest } from "../../../lib/templates";
import Icon from "../../../common/components/Icon";

export function ReviewStep({
  agent,
  mixins,
  selectedKits,
  name,
  onName,
  workspace,
  onWorkspace,
  savePath,
  onSavePath,
  busy,
  request,
  onSubmit,
  onBrowse,
  onBack,
}: {
  agent: KitOption;
  mixins: KitOption[];
  selectedKits: KitOption[];
  name: string;
  onName: (value: string) => void;
  workspace: string;
  onWorkspace: (value: string) => void;
  savePath: string;
  onSavePath: (value: string) => void;
  busy: boolean;
  request: SaveTemplateRequest;
  onSubmit: (launch: boolean) => void;
  onBrowse: (target: "workspace" | "path") => void;
  onBack: () => void;
}) {
  const id = useId();
  return (
    <form
      className="card catalog-panel"
      onSubmit={(event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        const submitter = (event.nativeEvent as SubmitEvent).submitter;
        onSubmit(submitter?.getAttribute("value") === "launch");
      }}
    >
      <h2>Bring your project</h2>
      <p className="catalog-muted">
        Choose the project folder to mount, then review the exact kit references. Your configuration
        is saved as a reusable template.
      </p>
      <fieldset className="catalog-unframed" disabled={busy}>
        <legend className="catalog-sr-only">Project details</legend>
        <label className="catalog-field" htmlFor={`${id}-name`}>
          Sandbox & template name <span className="catalog-sr-only">(required)</span>
          <input
            id={`${id}-name`}
            value={name}
            required
            maxLength={128}
            pattern="[a-zA-Z0-9][a-zA-Z0-9_.\-]*"
            placeholder="Project name"
            onChange={(event) => onName(event.target.value)}
          />
          <span className="catalog-help">
            Start with a letter or number; use letters, numbers, dots, underscores, or hyphens. This
            is the default launch name.
          </span>
        </label>
        <label className="catalog-field" htmlFor={`${id}-workspace`}>
          Workspace folder <span className="catalog-sr-only">(required)</span>
        </label>
        <div className="catalog-path-row">
          <input
            id={`${id}-workspace`}
            value={workspace}
            required
            placeholder="Absolute project path"
            aria-describedby={`${id}-workspace-help`}
            onChange={(event) => onWorkspace(event.target.value)}
          />
          <button type="button" className="ghost" onClick={() => onBrowse("workspace")}>
            Browse…
          </button>
        </div>
        <p className="catalog-help" id={`${id}-workspace-help`}>
          Choose an existing folder on this computer. The template stores this workspace for reuse.
        </p>
        <label className="catalog-field" htmlFor={`${id}-path`}>
          Save file location (optional)
        </label>
        <div className="catalog-path-row">
          <input
            id={`${id}-path`}
            value={savePath}
            placeholder="Use managed template storage"
            aria-describedby={`${id}-path-help`}
            onChange={(event) => onSavePath(event.target.value)}
          />
          <button type="button" className="ghost" onClick={() => onBrowse("path")}>
            Choose file…
          </button>
        </div>
        <p className="catalog-help" id={`${id}-path-help`}>
          Leave blank to use Sandy’s managed templates outside your workspace.
        </p>
        {savePath && (
          <button type="button" className="text-button" onClick={() => onSavePath("")}>
            Use managed storage
          </button>
        )}
        <div className="catalog-review">
          <h3>Review your environment</h3>
          <dl className="catalog-review-grid">
            <dt>Agent</dt>
            <dd>{agent.name}</dd>
            <dt>Kit format</dt>
            <dd>
              v{agent.schema_version} · environment schema v1
            </dd>
            <dt>Mixins</dt>
            <dd>{mixins.length} selected</dd>
            <dt>File location</dt>
            <dd>{savePath.trim() || "Sandy managed templates (outside the workspace)"}</dd>
          </dl>
          <ul className="catalog-reference-list">
            {selectedKits.map((kit) => (
              <li key={kit.id}>
                <strong>{kit.name}</strong>
                <code>{kit.source}</code>
              </li>
            ))}
          </ul>
          <h3>YAML preview</h3>
          <pre className="catalog-yaml" aria-label="Environment YAML preview">
            <code>{previewTemplate(request)}</code>
          </pre>
          <p className="catalog-help">
            The saved file’s actual YAML and final path will appear after saving.
          </p>
        </div>
        <div className="catalog-actions">
          <button type="button" className="ghost" onClick={onBack}>
            Back
          </button>
          <button
            type="submit"
            name="action"
            value="save"
            className="ghost"
            disabled={!name.trim() || !workspace.trim()}
          >
            Save template
          </button>
          <button
            type="submit"
            name="action"
            value="launch"
            className="primary"
            disabled={!name.trim() || !workspace.trim()}
          >
            {busy ? "Saving template…" : "Save & launch"} <Icon name="arrow" size={16} />
          </button>
        </div>
      </fieldset>
    </form>
  );
}
