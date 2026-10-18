import { useEffect, useId, useRef, useState } from "react";
import type { Catalog, KitOption, Provider } from "../../../lib/catalog";
import {
  argumentsAreValid,
  BUILTIN_AGENTS,
  hasKitMetadata,
  isCompatibleMixin,
  resolvedArguments,
} from "../../../lib/kit";
import { discoverKits } from "../../../lib/catalog";
import type { KitArguments } from "../../../lib/kit";
import {
  launchTemplate,
  saveTemplate,
} from "../../../lib/templates";
import { pickTemplatePath, pickWorkspace, saveProviders } from "../../../lib/catalog";
import type { SavedTemplate, SaveTemplateRequest } from "../../../lib/templates";
import Icon from "../../../common/components/Icon";
import { ErrorBanner } from "../../../common/components/ErrorBanner";
import { useMountedRef } from "../../../common/hooks/useMountedRef";
import { useBusyReporting } from "../../../common/hooks/useBusyReporting";
import Providers from "../../settings/components/Providers";
import { DiscoveryStep } from "./DiscoveryStep";
import { AgentStep } from "./AgentStep";
import { MixinStep } from "./MixinStep";
import { ReviewStep } from "./ReviewStep";
import { FinishedPanel } from "./FinishedPanel";

export interface CreatePageProps {
  onClose: () => void;
  /** Called only after a successful launch; the completed page stays visible. */
  onCreated: () => void;
  onTemplates?: () => void;
  onBusyChange?: (busy: boolean) => void;
}

const STEPS = ["Providers", "Agent or workload", "Mixins & inputs", "Project & review"];

export default function CreatePage({ onClose, onCreated, onTemplates, onBusyChange }: CreatePageProps) {
  const id = useId();
  const mounted = useMountedRef();
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
  const [busy, setBusy] = useState<"discover" | "save" | "launch" | "workspace" | "path" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState<SavedTemplate | null>(null);
  const [launched, setLaunched] = useState(false);
  const [launchOutput, setLaunchOutput] = useState("");
  const [copyNotice, setCopyNotice] = useState("");
  const lock = useRef(false);
  const heading = useRef<HTMLHeadingElement>(null);

  useBusyReporting(busy !== null, onBusyChange);

  useEffect(() => {
    heading.current?.focus();
  }, [step, saved]);

  const kits = catalog?.kits ?? [];
  const selectedKits = agent ? [agent, ...mixins] : [];
  const compatible = agent ? kits.filter((kit) => isCompatibleMixin(kit, agent)) : [];
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
      const path = await (target === "workspace" ? pickWorkspace() : pickTemplatePath());
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

  const submit = async (shouldLaunch: boolean) => {
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
      setCopyNotice(`Couldn’t copy YAML. Select the preview to copy it manually. ${String(failure)}`);
    }
  };

  return (
    <section className="standalone-page catalog-page" aria-labelledby={`${id}-title`} aria-busy={busy !== null}>
      <header className="page-head catalog-page-heading">
        <div>
          <span className="eyebrow">YOUR NEXT WORKSPACE</span>
          <h1 id={`${id}-title`}>{saved ? "Your template is ready" : "Create a sandbox"}</h1>
          <p className="page-sub">
            {saved
              ? "A reusable starting point, saved on your computer."
              : "Choose your tools. Bring your project. Make it yours."}
          </p>
        </div>
        <button type="button" className="ghost" disabled={busy !== null} onClick={onClose}>
          <Icon name="close" size={16} /> {saved ? "Close" : "Cancel"}
        </button>
      </header>
      {!saved && (
        <ol className="setup-steps catalog-steps" aria-label="Sandbox creation steps">
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
        <ErrorBanner
          title={saved ? "Launch needs attention." : "We couldn’t complete that step."}
          actionLabel={
            step === 0 && !saved ? "Retry discovery" : undefined
          }
          disabled={busy !== null || providersDirty || !providers}
          onAction={step === 0 && !saved ? () => void discover() : undefined}
        >
          <p>{error}</p>
        </ErrorBanner>
      )}
      {saved ? (
        <FinishedPanel
          saved={saved}
          launched={launched}
          launchOutput={launchOutput}
          busy={busy !== null}
          error={error}
          copyNotice={copyNotice}
          onRetryLaunch={() => void retryLaunch()}
          onCopy={() => void copy()}
          onTemplates={onTemplates}
          onClose={onClose}
        />
      ) : (
        <>
          {step === 0 && providers && (
            <div className="card catalog-panel">
              <h2>Start with your kit sources</h2>
              <p className="catalog-muted">
                Enable the providers you want to use and choose their versions. Discovery reads kit
                descriptors from those saved references.
              </p>
              <Providers
                disabled={busy !== null}
                onChange={(next) => {
                  setProviders(next);
                  setCatalog(null);
                }}
                onDirtyChange={setProvidersDirty}
              />
              <DiscoveryStep
                providers={providers}
                providersDirty={providersDirty}
                hubQuery={hubQuery}
                onHubQuery={setHubQuery}
                busy={busy !== null}
                onDiscover={() => void discover()}
              />
            </div>
          )}
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
                <AgentStep
                  agents={agents}
                  agent={agent}
                  search={agentSearch}
                  onSearch={setAgentSearch}
                  onSelect={(kit) => {
                    setAgent(kit);
                    setMixins((previous) =>
                      previous.filter(
                        (mixin) => mixin.name !== kit.name && isCompatibleMixin(mixin, kit),
                      ),
                    );
                    setError(null);
                  }}
                  onBack={() => moveTo(0)}
                  onContinue={() => moveTo(2)}
                  onReviewSources={() => moveTo(0)}
                />
              )}
              {step === 2 && agent && (
                <MixinStep
                  agent={agent}
                  compatible={compatible}
                  matchingMixins={matchingMixins}
                  mixinGroups={mixinGroups}
                  mixins={mixins}
                  values={values}
                  search={mixinSearch}
                  onSearch={setMixinSearch}
                  providerFilter={providerFilter}
                  onProviderFilter={setProviderFilter}
                  providerLabel={providerLabel}
                  onToggleMixin={(kit, checked) =>
                    setMixins((previous) =>
                      checked ? [...previous, kit] : previous.filter((item) => item.id !== kit.id),
                    )
                  }
                  onRemoveMixin={(kit) =>
                    setMixins((previous) => previous.filter((item) => item.id !== kit.id))
                  }
                  onArgumentChange={(kitId, argument, value) =>
                    setValues((previous) => ({
                      ...previous,
                      [kitId]: { ...previous[kitId], [argument]: value },
                    }))
                  }
                  onBack={() => moveTo(1)}
                  onContinue={() => inputValid && moveTo(3)}
                />
              )}
              {step === 3 && request && agent && (
                <ReviewStep
                  agent={agent}
                  mixins={mixins}
                  selectedKits={selectedKits}
                  name={name}
                  onName={setName}
                  workspace={workspace}
                  onWorkspace={setWorkspace}
                  savePath={savePath}
                  onSavePath={setSavePath}
                  busy={busy !== null}
                  request={request}
                  onSubmit={(launch) => void submit(launch)}
                  onBrowse={(target) => void browse(target)}
                  onBack={() => moveTo(2)}
                />
              )}
            </>
          )}
        </>
      )}
    </section>
  );
}
