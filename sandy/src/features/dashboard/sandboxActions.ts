import {
  startSandbox,
  stopSandbox,
  restartSandbox,
  openTerminal,
} from "../../lib/sandbox";

export type SandboxAction = "start" | "stop" | "restart" | "terminal";

export const SandboxActions: Record<SandboxAction, (name: string) => Promise<string | void>> = {
  start: startSandbox,
  stop: stopSandbox,
  restart: restartSandbox,
  terminal: openTerminal,
};
