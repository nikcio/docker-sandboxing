import { useEffect, useId, useRef, useState } from "react";
import type { FormEvent } from "react";
import {
  argumentValue,
  argumentsAreValid,
  BUILTIN_AGENTS,
  discoverKits,
  hasKitMetadata,
  isCompatibleMixin,
  launchTemplate,
  pickTemplatePath,
  pickWorkspace,
  previewTemplate,
  resolvedArguments,
  saveProviders,
  saveTemplate,
} from "./catalog-api";
import type {
  Catalog,
  KitArguments,
  KitOption,
  Provider,
  SavedTemplate,
  SaveTemplateRequest,
} from "./catalog-api";
import Icon from "./Icon";
import Providers from "./Providers";

export interface CreateSandboxProps {
  onClose: () => void;
  /** Called only after a successful launch; the completed page stays visible. */
  onCreated: () => void;
  onTemplates?: () => void;
  onBusyChange?: (busy: boolean) => void;
}

const STEPS = [
  "Providers",
  "Agent or workload",
  "Mixins & inputs",
  "Project & review",
];

function KitDetails({ kit }: { kit: KitOption }) {
  return (
    <span className="catalog-kit-details">
      <span className="catalog-kit-title">
        {kit.name}
        <span className="catalog-badge">
          Kit v{kit.schema_version} ·{" "}
          {kit.kind === "workload"
            ? "Workload"
            : kit.kind === "agent"
              ? "Agent"
              : "Mixin"}
        </span>
      </span>
      <span className="catalog-muted">
        {kit.description || "No description provided."}
      </span>
      <code className="catalog-source">{kit.source}</code>
      {kit.required_agent && (
        <span className="catalog-help">
          Requires agent: {kit.required_agent}
        </span>
      )}
    </span>
  );
}

function KitInputs({
  kits,
  values,
  onChange,
  disabled,
}: {
  kits: KitOption[];
  values: KitArguments;
  onChange: (kitId: string, argument: string, value: string) => void;
  disabled: boolean;
}) {
  const id = useId();
  const configurable = kits.filter((kit) => kit.arguments.length > 0);
  if (!configurable.length) {
    return (
      <p className="catalog-help">
        The selected kits don’t require any inputs.
      </p>
    );
  }
  return (
    <div className="catalog-kit-inputs">
      <h3>Kit inputs</h3>
      <p className="catalog-muted">
        Required inputs are marked. Defaults come from each kit’s metadata.
      </p>
      {configurable.map((kit, kitIndex) => (
        <fieldset
          className="catalog-input-group"
          key={kit.id}
          disabled={disabled}
        >
          <legend>{kit.name}</legend>
          <div className="catalog-fields">
            {kit.arguments.map((argument, argumentIndex) => {
              const inputId = `${id}-${kitIndex}-${argumentIndex}`;
              const value = argumentValue(values, kit, argument);
              return (
                <label
                  className="catalog-field"
                  key={argument.name}
                  htmlFor={inputId}
                >
                  <span>
                    {argument.name}{" "}
                    {argument.required ? (
                      <span className="catalog-required">(required)</span>
                    ) : (
                      <span className="catalog-help">(optional)</span>
                    )}
                  </span>
                  {argument.choices.length ? (
                    <select
                      id={inputId}
                      value={value}
                      required={argument.required}
                      aria-describedby={`${inputId}-help`}
                      onChange={(event) =>
                        onChange(kit.id, argument.name, event.target.value)
                      }
                    >
                      {!argument.choices.includes("") && (
                        <option value="">
                          {argument.required ? "Choose a value" : "Use kit default"}
                        </option>
                      )}
                      {argument.choices.map((choice) => (
                        <option key={choice} value={choice}>
                          {choice || "Empty value"}
                        </option>
                      ))}
                    </select>
                  ) : (
                    <input
                      id={inputId}
                      value={value}
                      required={argument.required}
                      placeholder="Value"
                      aria-describedby={`${inputId}-help`}
                      onChange={(event) =>
                        onChange(kit.id, argument.name, event.target.value)
                      }
                    />
                  )}
                  <span className="catalog-help" id={`${inputId}-help`}>
                    {argument.description}
                    {argument.default !== null && (
                      <>
                        {" "}Default:{" "}
                        <code>
                          {argument.default === "" ? "(empty)" : argument.default}
                        </code>
                        .
                      </>
                    )}
                  </span>
                </label>
              );
            })}
          </div>
        </fieldset>
      ))}
    </div>
  );
}

export default function CreateSandbox({
  onClose,
  onCreated,
  onTemplates,
  onBusyChange,
}: CreateSandboxProps) {
  const id = useId();
  const [step, setStep] = useState(0);
  const [providers, setProviders] = useState<Provider[] | null>(null);
  const [providersDirty, setProvidersDirty] = useState(true);
  const [hubQuery, setHubQuery] = useState("");
  const [catalog, setCatalog] = useState<Catalog | null>(null);
  const [agent, setAgent] = useState<KitOption | null>(null);
  const [mixins, setMixins] = useState<KitOption[]>([]);
  const [values, setValues] = useState<KitArguments>({});
  const [agentSearch, setAgentSearch] = useState("");
  const [mixinSearch, setMixinSearch] = useState("");
  const [providerFilter, setProviderFilter] = useState("");
  const [name, setName] = useState("");
  const [workspace, setWorkspace] = useState("");
  const [savePath, setSavePath] = useState("");
  const [busy, setBusy] = useState<
    "discover" | "save" | "launch" | "workspace" | "path" | null
  >(null);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState<SavedTemplate | null>(null);
  const [launched, setLaunched] = useState(false);
  const [launchOutput, setLaunchOutput] = useState("");
  const [copyNotice, setCopyNotice] = useState("");
  const lock = useRef(false);
  const heading = useRef<HTMLHeadingElement>(null);
  const mounted = useRef(true);
  const busyCallback = useRef(onBusyChange);
  busyCallback.current = onBusyChange;

  useEffect(() => {
    busyCallback.current?.(busy !== null);
  }, [busy]);

  useEffect(() => {
    return () => {
      busyCallback.current?.(false);
    };
  }, []);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  useEffect(() => {
    heading.current?.focus();
  }, [step, saved]);

  const kits = catalog?.kits ?? [];
  const selectedKits = agent ? [agent, ...mixins] : [];
  const compatible = agent
    ? kits.filter((kit) => isCompatibleMixin(kit, agent))
    : [];
  const matchingMixins = compatible.filter(
    (kit) =>
      (!providerFilter || kit.provider_id === providerFilter) &&
      `${kit.name} ${kit.description} ${kit.source}`
        .toLowerCase()
        .includes(mixinSearch.trim().toLowerCase()),
  );
  const mixinGroups = [...new Set(matchingMixins.map((kit) => kit.provider_id))];
  const agents = kits.filter(
    (kit) =>
      kit.kind !== "mixin" &&
      `${kit.name} ${kit.description} ${kit.source}`
        .toLowerCase()
        .includes(agentSearch.trim().toLowerCase()),
  );
  const providerLabel = (providerId: string) =>
    providers?.find((provider) => provider.id === providerId)?.label ??
    (providerId === "builtin" ? "Built-in agents" : providerId);
  const inputValid = argumentsAreValid(selectedKits, values);
  const request: SaveTemplateRequest | null = agent
    ? {
        name: name.trim(),
        workspace: workspace.trim(),
        agent,
        mixins,
        arguments: resolvedArguments(selectedKits, values),
        ...(savePath.trim() ? { save_path: savePath.trim() } : {}),
      }
    : null;

  const moveTo = (next: number) => {
    setError(null);
    setStep(next);
  };

  const discover = async () => {
    if (lock.current || !providers || providersDirty) return;
    lock.current = true;
    setBusy("discover");
    setError(null);
    try {
      // Defaults from get_providers may not have a file yet. Persist before discovery.
      const persistedProviders = await saveProviders(providers);
      const result = await discoverKits(persistedProviders, hubQuery);
      if (!mounted.current) return;
      const seen = new Set<string>();
      const verified = result.kits.filter((kit) => {
        if (!hasKitMetadata(kit) || seen.has(kit.id)) return false;
        seen.add(kit.id);
        return true;
      });
      const skipped = result.kits.length - verified.length;
      const usedBuiltinFallback = !verified.some(
        (kit) => kit.provider_id === "builtin" && kit.kind === "agent",
      );
      if (usedBuiltinFallback) {
        verified.push(...BUILTIN_AGENTS);
      }
      setCatalog({
        kits: verified,
        warnings: [
          ...result.warnings,
          ...(usedBuiltinFallback
            ? [
                "Built-in agent shortcuts are from Docker’s documentation because the CLI did not report its available agents. These v2 shortcuts have not been verified against this installation.",
              ]
            : []),
          ...(skipped
            ? [
                `${skipped} entries with incomplete, unsupported, or duplicate kit metadata were excluded.`,
              ]
            : []),
        ],
      });
      // Discovery is a new snapshot. Old selections cannot carry stale source references.
      setAgent(null);
      setMixins([]);
      setValues({});
      setProviderFilter("");
      setStep(1);
    } catch (failure) {
      if (mounted.current) setError(String(failure));
    } finally {
      lock.current = false;
      if (mounted.current) setBusy(null);
    }
  };

  const browse = async (target: "workspace" | "path") => {
    if (lock.current) return;
    lock.current = true;
    setBusy(target);
    setError(null);
    try {
      const path = await (
        target === "workspace" ? pickWorkspace() : pickTemplatePath()
      );
      if (path && mounted.current) {
        if (target === "workspace") {
          setWorkspace(path);
        } else {
          setSavePath(path);
        }
      }
    } catch (failure) {
      if (mounted.current) setError(String(failure));
    } finally {
      lock.current = false;
      if (mounted.current) setBusy(null);
    }
  };

  const launch = async (template: SavedTemplate) => {
    // Always pass the saved configuration's name, not a generated launch label.
    const output = await launchTemplate({
      id: template.id,
      name: template.name,
      workspace: template.workspace,
    });
    if (!mounted.current) return;
    setLaunchOutput(output);
    setLaunched(true);
    onCreated();
  };

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (lock.current || !request) return;
    if (
      !inputValid ||
      !request.name ||
      !request.workspace ||
      !mixins.every((kit) => isCompatibleMixin(kit, request.agent))
    ) {
      setError(
        "Choose a compatible agent and mixins, complete required kit inputs, and enter a project name and workspace.",
      );
      return;
    }
    const shouldLaunch =
      (event.nativeEvent as SubmitEvent).submitter?.getAttribute("value") ===
      "launch";
    lock.current = true;
    setBusy("save");
    setError(null);
    let persisted = false;
    try {
      const template = await saveTemplate(request);
      persisted = true;
      if (!mounted.current) return;
      setSaved(template);
      if (shouldLaunch) {
        setBusy("launch");
        await launch(template);
      }
    } catch (failure) {
      if (mounted.current) {
        setError(
          `${persisted ? "Template saved, but the sandbox could not be launched. " : ""}${String(failure)}`,
        );
      }
    } finally {
      lock.current = false;
      if (mounted.current) setBusy(null);
    }
  };

  const retryLaunch = async () => {
    if (lock.current || !saved) return;
    lock.current = true;
    setBusy("launch");
    setError(null);
    try {
      await launch(saved);
    } catch (failure) {
      if (mounted.current) setError(String(failure));
    } finally {
      lock.current = false;
      if (mounted.current) setBusy(null);
    }
  };

  const copy = async () => {
    if (!saved) return;
    try {
      await navigator.clipboard.writeText(saved.content);
      setCopyNotice("YAML copied to clipboard.");
    } catch (failure) {
      setCopyNotice(
        `Couldn’t copy YAML. Select the preview to copy it manually. ${String(failure)}`,
      );
    }
  };

  return (
    <section
      className="standalone-page catalog-page"
      aria-labelledby={`${id}-title`}
      aria-busy={busy !== null}
    >
      <header className="page-head catalog-page-heading">
        <div>
          <span className="eyebrow">YOUR NEXT WORKSPACE</span>
          <h1 id={`${id}-title`}>
            {saved ? "Your template is ready" : "Create a sandbox"}
          </h1>
          <p className="page-sub">
            {saved
              ? "A reusable starting point, saved on your computer."
              : "Choose your tools. Bring your project. Make it yours."}
          </p>
        </div>
        <button
          type="button"
          className="ghost"
          disabled={busy !== null}
          onClick={onClose}
        >
          <Icon name="close" size={16} /> {saved ? "Close" : "Cancel"}
        </button>
      </header>
      {!saved && (
        <ol
          className="setup-steps catalog-steps"
          aria-label="Sandbox creation steps"
        >
          {STEPS.map((label, index) => (
            <li key={label}>
              <button
                type="button"
                className={`setup-step ${step === index ? "current" : ""} ${step > index ? "complete" : ""}`}
                aria-current={step === index ? "step" : undefined}
                disabled={busy !== null || index > step}
                onClick={() => moveTo(index)}
              >
                <span className="step-number">
                  {step > index ? <Icon name="check" size={14} /> : index + 1}
                </span>
                {label}
              </button>
            </li>
          ))}
        </ol>
      )}
      {error && (
        <div className="error-banner" role="alert">
          <strong>
            {saved ? "Launch needs attention." : "We couldn’t complete that step."}
          </strong>
          <p>{error}</p>
          {step === 0 && !saved && (
            <button
              type="button"
              className="ghost"
              disabled={busy !== null || providersDirty || !providers}
              onClick={discover}
            >
              Retry discovery
            </button>
          )}
        </div>
      )}
      {saved ? (
        <div className="card catalog-panel catalog-finished">
          <span className={`empty-icon ${launched ? "green" : ""}`}>
            <Icon name={launched ? "check" : "template"} size={30} />
          </span>
          <h2 ref={heading} tabIndex={-1}>
            {launched ? "Sandbox launched" : "Template saved"}
          </h2>
          <p className="catalog-muted">
            {busy === "launch"
              ? "Launching your sandbox…"
              : launched
                ? `${saved.name} is ready in your sandboxes.`
                : "You can launch this template now, or reuse it from Templates later."}
          </p>
          <dl className="catalog-review-grid">
            <dt>Name</dt>
            <dd>{saved.name}</dd>
            <dt>File</dt>
            <dd>
              <code>{saved.path}</code>
            </dd>
            <dt>Agent</dt>
            <dd>{saved.agent}</dd>
            <dt>Kit format</dt>
            <dd>v{saved.kit_version}</dd>
            <dt>Workspace</dt>
            <dd>{saved.workspace}</dd>
          </dl>
          <div className="catalog-section-heading">
            <h3>Saved YAML</h3>
            <button
              type="button"
              className="ghost"
              disabled={busy !== null}
              onClick={copy}
            >
              <Icon name="clipboard" size={16} /> Copy YAML
            </button>
          </div>
          <pre className="catalog-yaml" aria-label="Saved template YAML">
            <code>{saved.content}</code>
          </pre>
          {copyNotice && (
            <p className="catalog-help" role="status">
              {copyNotice}
            </p>
          )}
          {launchOutput && (
            <details className="catalog-output">
              <summary>Launch output</summary>
              <pre>{launchOutput}</pre>
            </details>
          )}
          <div className="catalog-actions">
            {!launched && (
              <button
                type="button"
                className="primary"
                disabled={busy !== null}
                onClick={retryLaunch}
              >
                {busy === "launch" ? (
                  <>
                    <span className="spinner" /> Launching…
                  </>
                ) : error ? (
                  "Retry launch"
                ) : (
                  "Launch sandbox"
                )}
              </button>
            )}
            {onTemplates && (
              <button
                type="button"
                className={launched ? "primary" : "ghost"}
                disabled={busy !== null}
                onClick={onTemplates}
              >
                View saved templates
              </button>
            )}
            <button
              type="button"
              className="ghost"
              disabled={busy !== null}
              onClick={onClose}
            >
              Done
            </button>
          </div>
        </div>
      ) : (
        <>
          <div hidden={step !== 0} className="card catalog-panel">
            <h2 ref={step === 0 ? heading : undefined} tabIndex={-1}>
              Start with your kit sources
            </h2>
            <p className="catalog-muted">
              Enable the providers you want to use and choose their versions.
              Discovery reads kit descriptors from those saved references.
            </p>
            <Providers
              disabled={busy !== null}
              onChange={(next) => {
                setProviders(next);
                setCatalog(null);
              }}
              onDirtyChange={setProvidersDirty}
            />
            {providers?.some(
              (provider) => provider.kind === "dockerhub" && provider.enabled,
            ) && (
              <label
                className="catalog-field catalog-hub-query"
                htmlFor={`${id}-hub-query`}
              >
                Docker Hub search (optional)
                <input
                  id={`${id}-hub-query`}
                  value={hubQuery}
                  placeholder="Search kits"
                  disabled={busy !== null}
                  onChange={(event) => setHubQuery(event.target.value)}
                />
                <span className="catalog-help">
                  Narrows discovery results, including Docker Hub. Search runs
                  only when you choose Discover kits. Specific repositories use
                  their selected reference.
                </span>
              </label>
            )}
            <div className="catalog-actions">
              <button
                type="button"
                className="primary"
                disabled={
                  busy !== null ||
                  !providers ||
                  providersDirty ||
                  !providers.some((provider) => provider.enabled)
                }
                onClick={discover}
              >
                {busy === "discover" ? (
                  <>
                    <span className="spinner" /> Discovering kits…
                  </>
                ) : (
                  <>
                    Discover kits <Icon name="arrow" size={16} />
                  </>
                )}
              </button>
              <span className="catalog-help">
                {providersDirty
                  ? "Save provider changes before continuing."
                  : "Only kits with supported metadata will be offered."}
              </span>
            </div>
          </div>
          {step > 0 && catalog && (
            <>
              {catalog.warnings.length > 0 && (
                <details className="catalog-warnings" open>
                  <summary>Discovery notes ({catalog.warnings.length})</summary>
                  <ul>
                    {catalog.warnings.map((warning, index) => (
                      <li key={index}>{warning}</li>
                    ))}
                  </ul>
                  <button
                    type="button"
                    className="text-button"
                    disabled={busy !== null}
                    onClick={() => moveTo(0)}
                  >
                    Review sources and retry discovery
                  </button>
                </details>
              )}
              {step === 1 && (
                <div className="card catalog-panel">
                  <h2 ref={heading} tabIndex={-1}>
                    Choose an agent or workload
                  </h2>
                  <p className="catalog-muted">
                    Built-in agent shortcuts use kit v2. A v3 workload starts a
                    v3-only environment. Remote kit sources below come from
                    discovered descriptors.
                  </p>
                  <label
                    className="catalog-field catalog-search"
                    htmlFor={`${id}-agent-search`}
                  >
                    Search agents and workloads
                    <input
                      id={`${id}-agent-search`}
                      type="search"
                      placeholder="Search catalog"
                      value={agentSearch}
                      onChange={(event) => setAgentSearch(event.target.value)}
                    />
                  </label>
                  {([2, 3] as const).map((version) => {
                    const items = agents.filter(
                      (kit) => kit.schema_version === version,
                    );
                    return (
                      items.length > 0 && (
                        <fieldset
                          className="catalog-unframed catalog-choice-group"
                          key={version}
                        >
                          <legend>
                            {version === 2 ? "Kit v2 agents" : "Kit v3 workloads"}
                          </legend>
                          <div className="catalog-kit-grid">
                            {items.map((kit) => (
                              <label
                                className={`catalog-kit ${agent?.id === kit.id ? "selected" : ""}`}
                                key={kit.id}
                              >
                                <input
                                  type="radio"
                                  name={`${id}-agent`}
                                  value={kit.id}
                                  checked={agent?.id === kit.id}
                                  onChange={() => {
                                    setAgent(kit);
                                    setMixins((previous) =>
                                      previous.filter(
                                        (mixin) =>
                                          mixin.name !== kit.name &&
                                          isCompatibleMixin(mixin, kit),
                                      ),
                                    );
                                    setError(null);
                                  }}
                                />
                                <KitDetails kit={kit} />
                              </label>
                            ))}
                          </div>
                        </fieldset>
                      )
                    );
                  })}
                  {!agents.length && (
                    <div className="catalog-empty-note">
                      <h3>
                        {agentSearch
                          ? "No matching agents or workloads"
                          : "No agents or workloads were discovered"}
                      </h3>
                      <p>
                        {agentSearch
                          ? "Try a different search."
                          : "Review the discovery notes, then check your enabled providers and references."}
                      </p>
                      <button
                        type="button"
                        className="ghost"
                        onClick={() =>
                          agentSearch ? setAgentSearch("") : moveTo(0)
                        }
                      >
                        {agentSearch ? "Clear search" : "Review providers"}
                      </button>
                    </div>
                  )}
                  <div className="catalog-actions">
                    <button
                      type="button"
                      className="ghost"
                      onClick={() => moveTo(0)}
                    >
                      Back
                    </button>
                    <button
                      type="button"
                      className="primary"
                      disabled={!agent}
                      onClick={() => moveTo(2)}
                    >
                      Choose mixins <Icon name="arrow" size={16} />
                    </button>
                  </div>
                </div>
              )}
              {step === 2 && agent && (
                <form
                  className="card catalog-panel"
                  onSubmit={(event) => {
                    event.preventDefault();
                    if (inputValid) moveTo(3);
                  }}
                >
                  <div className="catalog-section-heading">
                    <div>
                      <h2 ref={heading} tabIndex={-1}>
                        Make it your own
                      </h2>
                      <p className="catalog-muted">
                        Showing only kit v{agent.schema_version} mixins compatible
                        with {agent.name}, including each mixin’s required agent.
                      </p>
                    </div>
                    <span className="catalog-badge">
                      {mixins.length} selected
                    </span>
                  </div>
                  <div className="catalog-fields">
                    <label
                      className="catalog-field"
                      htmlFor={`${id}-mixin-search`}
                    >
                      Search mixins
                      <input
                        id={`${id}-mixin-search`}
                        type="search"
                        placeholder="Search mixins"
                        value={mixinSearch}
                        onChange={(event) => setMixinSearch(event.target.value)}
                      />
                    </label>
                    <label
                      className="catalog-field"
                      htmlFor={`${id}-provider-filter`}
                    >
                      Group by provider
                      <select
                        id={`${id}-provider-filter`}
                        value={providerFilter}
                        onChange={(event) => setProviderFilter(event.target.value)}
                      >
                        <option value="">All providers</option>
                        {[...new Set(compatible.map((kit) => kit.provider_id))].map(
                          (providerId) => (
                            <option key={providerId} value={providerId}>
                              {providerLabel(providerId)}
                            </option>
                          ),
                        )}
                      </select>
                    </label>
                  </div>
                  {mixinGroups.map((providerId) => (
                    <fieldset
                      className="catalog-unframed catalog-choice-group"
                      key={providerId}
                    >
                      <legend>{providerLabel(providerId)}</legend>
                      <div className="catalog-kit-grid">
                        {matchingMixins
                          .filter((kit) => kit.provider_id === providerId)
                          .map((kit) => {
                            const selected = mixins.some(
                              (item) => item.id === kit.id,
                            );
                            const duplicateName =
                              !selected &&
                              (agent.name === kit.name ||
                                mixins.some((item) => item.name === kit.name));
                            return (
                              <label
                                className={`catalog-kit ${selected ? "selected" : ""}`}
                                key={kit.id}
                              >
                                <input
                                  type="checkbox"
                                  checked={selected}
                                  disabled={duplicateName}
                                  onChange={(event) =>
                                    setMixins((previous) =>
                                      event.target.checked
                                        ? [...previous, kit]
                                        : previous.filter(
                                            (item) => item.id !== kit.id,
                                          ),
                                    )
                                  }
                                />
                                <span className="catalog-kit-details">
                                  <KitDetails kit={kit} />
                                  {duplicateName && (
                                    <span className="catalog-help">
                                      Another kit with this name is already
                                      selected.
                                    </span>
                                  )}
                                </span>
                              </label>
                            );
                          })}
                      </div>
                    </fieldset>
                  ))}
                  {!matchingMixins.length && (
                    <div className="catalog-empty-note">
                      <p>
                        {compatible.length
                          ? "No mixins match these filters."
                          : "No compatible mixins were discovered. You can continue with the selected agent or workload alone."}
                      </p>
                      {(mixinSearch || providerFilter) && (
                        <button
                          type="button"
                          className="ghost"
                          onClick={() => {
                            setMixinSearch("");
                            setProviderFilter("");
                          }}
                        >
                          Clear filters
                        </button>
                      )}
                    </div>
                  )}
                  {mixins.length > 0 && (
                    <div className="catalog-selected">
                      <h3>Selected mixins · applied in this order</h3>
                      <ol>
                        {mixins.map((kit) => (
                          <li key={kit.id}>
                            <span>{kit.name}</span>
                            <button
                              type="button"
                              className="text-button"
                              onClick={() =>
                                setMixins((previous) =>
                                  previous.filter((item) => item.id !== kit.id),
                                )
                              }
                              aria-label={`Remove ${kit.name}`}
                            >
                              Remove
                            </button>
                          </li>
                        ))}
                      </ol>
                    </div>
                  )}
                  <KitInputs
                    kits={selectedKits}
                    values={values}
                    disabled={false}
                    onChange={(kitId, argument, value) =>
                      setValues((previous) => ({
                        ...previous,
                        [kitId]: { ...previous[kitId], [argument]: value },
                      }))
                    }
                  />
                  <div className="catalog-actions">
                    <button
                      type="button"
                      className="ghost"
                      onClick={() => moveTo(1)}
                    >
                      Back
                    </button>
                    <button
                      type="submit"
                      className="primary"
                      disabled={!inputValid}
                    >
                      Project details <Icon name="arrow" size={16} />
                    </button>
                    {!inputValid && (
                      <span className="catalog-help" role="status">
                        Complete the required kit inputs to continue.
                      </span>
                    )}
                  </div>
                </form>
              )}
              {step === 3 && request && (
                <form className="card catalog-panel" onSubmit={submit}>
                  <h2 ref={heading} tabIndex={-1}>
                    Bring your project
                  </h2>
                  <p className="catalog-muted">
                    Choose the project folder to mount, then review the exact kit
                    references. Your configuration is saved as a reusable template.
                  </p>
                  <fieldset className="catalog-unframed" disabled={busy !== null}>
                    <legend className="catalog-sr-only">Project details</legend>
                    <label className="catalog-field" htmlFor={`${id}-name`}>
                      Sandbox & template name{" "}
                      <span className="catalog-sr-only">(required)</span>
                      <input
                        id={`${id}-name`}
                        value={name}
                        required
                        maxLength={128}
                        pattern="[a-zA-Z0-9][a-zA-Z0-9_.\\-]*"
                        placeholder="Project name"
                        onChange={(event) => setName(event.target.value)}
                      />
                      <span className="catalog-help">
                        Start with a letter or number; use letters, numbers, dots,
                        underscores, or hyphens. This is the default launch name.
                      </span>
                    </label>
                    <label className="catalog-field" htmlFor={`${id}-workspace`}>
                      Workspace folder{" "}
                      <span className="catalog-sr-only">(required)</span>
                    </label>
                    <div className="catalog-path-row">
                      <input
                        id={`${id}-workspace`}
                        value={workspace}
                        required
                        placeholder="Absolute project path"
                        aria-describedby={`${id}-workspace-help`}
                        onChange={(event) => setWorkspace(event.target.value)}
                      />
                      <button
                        type="button"
                        className="ghost"
                        onClick={() => browse("workspace")}
                      >
                        Browse…
                      </button>
                    </div>
                    <p className="catalog-help" id={`${id}-workspace-help`}>
                      Choose an existing folder on this computer. The template
                      stores this workspace for reuse.
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
                        onChange={(event) => setSavePath(event.target.value)}
                      />
                      <button
                        type="button"
                        className="ghost"
                        onClick={() => browse("path")}
                      >
                        Choose file…
                      </button>
                    </div>
                    <p className="catalog-help" id={`${id}-path-help`}>
                      Leave blank to use Sandy’s managed templates outside your
                      workspace.
                    </p>
                    {savePath && (
                      <button
                        type="button"
                        className="text-button"
                        onClick={() => setSavePath("")}
                      >
                        Use managed storage
                      </button>
                    )}
                    <div className="catalog-review">
                      <h3>Review your environment</h3>
                      <dl className="catalog-review-grid">
                        <dt>Agent</dt>
                        <dd>{agent?.name}</dd>
                        <dt>Kit format</dt>
                        <dd>
                          v{agent?.schema_version} · environment schema v1
                        </dd>
                        <dt>Mixins</dt>
                        <dd>{mixins.length} selected</dd>
                        <dt>File location</dt>
                        <dd>
                          {savePath.trim() ||
                            "Sandy managed templates (outside the workspace)"}
                        </dd>
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
                      <pre
                        className="catalog-yaml"
                        aria-label="Environment YAML preview"
                      >
                        <code>{previewTemplate(request)}</code>
                      </pre>
                      <p className="catalog-help">
                        The saved file’s actual YAML and final path will appear
                        after saving.
                      </p>
                    </div>
                    <div className="catalog-actions">
                      <button
                        type="button"
                        className="ghost"
                        onClick={() => moveTo(2)}
                      >
                        Back
                      </button>
                      <button
                        type="submit"
                        name="action"
                        value="save"
                        className="ghost"
                        disabled={!inputValid || !name.trim() || !workspace.trim()}
                      >
                        Save template
                      </button>
                      <button
                        type="submit"
                        name="action"
                        value="launch"
                        className="primary"
                        disabled={!inputValid || !name.trim() || !workspace.trim()}
                      >
                        {busy === "save" ? "Saving template…" : "Save & launch"}{" "}
                        <Icon name="arrow" size={16} />
                      </button>
                    </div>
                  </fieldset>
                </form>
              )}
            </>
          )}
        </>
      )}
    </section>
  );
}
