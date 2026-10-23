import { invoke } from "@tauri-apps/api/core";
import { shareInFlight } from "./inFlight";

export interface Provider {
  id: string;
  label: string;
  kind: "github" | "dockerhub";
  repository: string;
  reference: string;
  enabled: boolean;
}

export interface KitArgument {
  name: string;
  description: string;
  required: boolean;
  default: string | null;
  choices: string[];
}

export interface KitOption {
  id: string;
  name: string;
  description: string;
  source: string;
  schema_version: 2 | 3;
  kind: "agent" | "workload" | "mixin";
  provider_id: string;
  required_agent: string | null;
  arguments: KitArgument[];
}

export interface Catalog {
  kits: KitOption[];
  warnings: string[];
}

export type KitArguments = Record<string, Record<string, string>>;

// Same dedupe as sandbox.ts's checkSetup: StrictMode double mounts must not
// read the provider store twice.
export const getProviders = shareInFlight(() => invoke<Provider[]>("get_providers"));
export const saveProviders = (providers: Provider[]) =>
  invoke<Provider[]>("save_providers", { providers });
export const providerVersions = (provider: Provider) =>
  invoke<string[]>("provider_versions", { provider });
export const discoverKits = (providers: Provider[], query?: string) =>
  invoke<Catalog>("discover_kits", {
    providers,
    ...(query?.trim() ? { query: query.trim() } : {}),
  });
export const pickWorkspace = () => invoke<string | null>("pick_workspace");
export const pickTemplatePath = () => invoke<string | null>("pick_template_path");

/** One structured entry from the Sandy diagnostic log (JSON Lines on disk). */
export interface DebugLogEntry {
  timestamp: string;
  kind: "cli" | "http" | "command" | "error" | "info";
  summary: string;
  detail?: string;
  outcome?: "ok" | "error";
}

/** The whole diagnostic log file. Empty when nothing has been logged yet. */
export const readDebugLog = () => invoke<string>("read_debug_log");
export const clearDebugLog = () => invoke<void>("clear_debug_log");

export function isHubSearchProvider(provider: Provider): boolean {
  const repository = provider.repository.trim().replace(/\/+$/, "");
  return (
    provider.kind === "dockerhub" &&
    (repository === "" ||
      repository === "docker.io" ||
      repository === "https://hub.docker.com/search" ||
      repository === "https://hub.docker.com/search?type=sbx_kit")
  );
}
