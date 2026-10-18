import type { KitArgument, KitArguments, KitOption } from "./catalog";
import type { SaveTemplateRequest } from "./templates";

export type { KitArgument, KitArguments, KitOption, SaveTemplateRequest };

// Documented Docker shortcuts, used only if discovery doesn't return built-ins.
// https://docs.docker.com/ai/sandboxes/customize/kits/#sandbox-managed-agent-configuration
// https://docs.docker.com/ai/sandboxes/configuration/environment-files/#use-a-cloud-environment
// These are v2 agent identities, never inferred Docker Hub image tags.
export const BUILTIN_AGENTS: KitOption[] = [
  "claude",
  "codex",
  "copilot",
  "cursor",
  "devin",
  "gemini",
  "kiro",
  "opencode",
  "shell",
].map((name) => ({
  id: `builtin:${name}`,
  name,
  description: `Docker Sandboxes built-in ${name} environment.`,
  source: name,
  schema_version: 2,
  kind: "agent",
  provider_id: "builtin",
  required_agent: null,
  arguments: [],
}));

/** Unknown or incomplete metadata must never turn into a selectable kit. */
export function hasKitMetadata(kit: KitOption): boolean {
  return !!(
    kit &&
    typeof kit.id === "string" &&
    kit.id.trim() &&
    typeof kit.name === "string" &&
    kit.name.trim() &&
    typeof kit.source === "string" &&
    kit.source.trim() &&
    typeof kit.description === "string" &&
    typeof kit.provider_id === "string" &&
    kit.provider_id.trim() &&
    (kit.schema_version === 2 || kit.schema_version === 3) &&
    (kit.kind === "mixin" ||
      (kit.schema_version === 2 && kit.kind === "agent") ||
      (kit.schema_version === 3 && kit.kind === "workload")) &&
    (kit.required_agent === null || typeof kit.required_agent === "string") &&
    Array.isArray(kit.arguments) &&
    kit.arguments.every(
      (argument) =>
        argument &&
        typeof argument.name === "string" &&
        argument.name.trim() &&
        typeof argument.description === "string" &&
        typeof argument.required === "boolean" &&
        (argument.default === null || typeof argument.default === "string") &&
        Array.isArray(argument.choices) &&
        argument.choices.every((choice) => typeof choice === "string"),
    )
  );
}

export function isCompatibleMixin(mixin: KitOption, agent: KitOption): boolean {
  return (
    hasKitMetadata(agent) &&
    hasKitMetadata(mixin) &&
    agent.kind !== "mixin" &&
    mixin.kind === "mixin" &&
    mixin.schema_version === agent.schema_version &&
    (mixin.required_agent === null || mixin.required_agent === agent.name)
  );
}

export function argumentValue(values: KitArguments, kit: KitOption, argument: KitArgument): string {
  return values[kit.id]?.[argument.name] ?? argument.default ?? "";
}

export function argumentsAreValid(kits: KitOption[], values: KitArguments): boolean {
  return kits.every((kit) =>
    kit.arguments.every((argument) => {
      const value = argumentValue(values, kit, argument);
      if (argument.required && !value.trim()) return false;
      return (
        !argument.choices.length ||
        (!value && !argument.required && argument.default === null) ||
        argument.choices.includes(value)
      );
    }),
  );
}

export function resolvedArguments(kits: KitOption[], values: KitArguments): KitArguments {
  return Object.fromEntries(
    kits.map((kit) => [
      kit.id,
      Object.fromEntries(
        kit.arguments
          .filter(
            (argument) =>
              argumentValue(values, kit, argument) !== "" ||
              argument.default !== null ||
              argument.required,
          )
          .map((argument) => [argument.name, argumentValue(values, kit, argument)]),
      ),
    ]),
  );
}

/** Mirrors sbxenv's documented format; saved previews always use backend content. */
export function previewTemplate(request: SaveTemplateRequest): string {
  const quote = (value: string) => JSON.stringify(value);
  const escape = (value: string) => value.replace(/\$\{\{/g, () => "$${{");
  const { agent } = request;
  const builtin =
    agent.schema_version === 2 &&
    agent.kind === "agent" &&
    agent.provider_id === "builtin" &&
    agent.source === agent.name;
  const kits = [...(builtin ? [] : [agent]), ...request.mixins];
  const lines = [
    'schemaVersion: "1"',
    `name: ${quote(request.name)}`,
    `agent: ${quote(agent.name)}`,
    `workspace: ${quote(escape(request.workspace))}`,
  ];
  if (kits.length) {
    lines.push("kits:");
    for (const kit of kits) {
      const args = Object.entries(request.arguments[kit.id] ?? {}).sort(([left], [right]) =>
        left.localeCompare(right),
      );
      if (!args.length) {
        lines.push(`  - ${quote(kit.source)}`);
      } else {
        lines.push(`  - source: ${quote(kit.source)}`, "    args:");
        for (const [name, value] of args) {
          lines.push(`      ${quote(name)}: ${quote(escape(value))}`);
        }
      }
    }
  }
  return `${lines.join("\n")}\n`;
}
