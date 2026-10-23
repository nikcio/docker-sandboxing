import { useEffect, useRef, useState } from "react";
import type { Catalog, KitArguments, KitOption, Provider } from "../../../lib/catalog";
import {
  discoverKits,
  getProviders,
  pickTemplatePath,
  pickWorkspace,
  saveProviders,
} from "../../../lib/catalog";
import {
  argumentsAreValid,
  BUILTIN_AGENTS,
  hasKitMetadata,
  isCompatibleMixin,
  resolvedArguments,
} from "../../../lib/kit";
import { launchTemplate, saveTemplate } from "../../../lib/templates";
import type { SavedTemplate, SaveTemplateRequest } from "../../../lib/templates";
import { useMountedRef } from "../../../common/hooks/useMountedRef";
import { useBusyReporting } from "../../../common/hooks/useBusyReporting";

export interface CreateFlowOptions {
  onBusyChange?: (busy: boolean) => void;
}

/**
 * The create-wizard state machine: kit discovery, agent/mixin selection,
 * template saving, and launching. The page component only renders steps.
 */
export function useCreateFlow({ onBusyChange }: CreateFlowOptions) {
  const mounted = useMountedRef();
  const [step, setStep] = useState(0);
  const [providers, setProviders] = useState<Provider[] | null>(null);
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

  // Load the saved provider configuration up front. The Providers editor still
  // reports changes back; this only guarantees step 0 can render (with a
  // surfaced error) even if that load fails.
  useEffect(() => {
    let active = true;
    getProviders()
      .then((result) => {
        if (active) setProviders((previous) => previous ?? result);
      })
      .catch((failure) => {
        if (active) setError(String(failure));
      });
    return () => {
      active = false;
    };
  }, []);

  useBusyReporting(busy !== null, onBusyChange);

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
    // Later steps render from the discovery snapshot; without one, send the
    // user back to the provider step instead of showing a blank page.
    if (next > 0 && !catalog) {
      setStep(0);
      return;
    }
    setStep(next);
  };

  const discover = async () => {
    // Provider edits are persisted by saveProviders below, so unsaved changes
    // do not need to block discovery.
    if (lock.current || !providers) return;
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

  const onProvidersChange = (next: Provider[]) => {
    // Only invalidate the discovery snapshot when the configuration actually
    // differs from the discovered one. Remounting the editor re-reports the
    // unchanged saved configuration; that must not reset the wizard's progress.
    const changed = providers !== null && JSON.stringify(providers) !== JSON.stringify(next);
    setProviders(next);
    if (changed && catalog) {
      setCatalog(null);
      setStep(0);
      setError(
        "Provider changes need a new discovery. Review your sources and discover kits again.",
      );
    }
  };

  const selectAgent = (kit: KitOption) => {
    setAgent(kit);
    setMixins((previous) =>
      previous.filter((mixin) => mixin.name !== kit.name && isCompatibleMixin(mixin, kit)),
    );
    setError(null);
  };

  const toggleMixin = (kit: KitOption, checked: boolean) => {
    setMixins((previous) =>
      checked ? [...previous, kit] : previous.filter((item) => item.id !== kit.id),
    );
  };

  const removeMixin = (kit: KitOption) => {
    setMixins((previous) => previous.filter((item) => item.id !== kit.id));
  };

  const changeArgument = (kitId: string, argument: string, value: string) => {
    setValues((previous) => ({
      ...previous,
      [kitId]: { ...previous[kitId], [argument]: value },
    }));
  };

  return {
    // Wizard position and shared feedback state.
    step,
    moveTo,
    busy,
    error,
    /** All selected kits (agent first); drives argument validity and the review. */
    selectedKits,
    values,
    // Step 0: providers and discovery.
    providers,
    hubQuery,
    setHubQuery,
    catalog,
    onProvidersChange,
    discover,
    // Step 1: agent selection.
    agents,
    agent,
    agentSearch,
    setAgentSearch,
    selectAgent,
    // Step 2: mixin selection and kit inputs.
    compatible,
    matchingMixins,
    mixinGroups,
    mixins,
    mixinSearch,
    setMixinSearch,
    providerFilter,
    setProviderFilter,
    providerLabel,
    toggleMixin,
    removeMixin,
    changeArgument,
    // Step 3: project details and review.
    inputValid,
    request,
    name,
    setName,
    workspace,
    setWorkspace,
    savePath,
    setSavePath,
    submit,
    browse,
    // Finished panel.
    saved,
    launched,
    launchOutput,
    copyNotice,
    retryLaunch,
    copy,
  };
}
