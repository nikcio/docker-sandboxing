import { invoke } from "@tauri-apps/api/core";

export interface SetupStatus {
  sbx_installed: boolean;
  signed_in: boolean | null;
  image_paste_on: boolean | null;
  ssh_set_up: boolean | null;
  issues?: string[];
}

export interface SandboxInfo {
  name: string;
  state: string;
  agent: string;
  workspace: string;
  raw: string;
}

export interface ConnectionInfo {
  host: string;
  user: string;
  port?: number;
  command: string;
}

export const getStatus = () => invoke<SetupStatus>("status");
export const installSbx = () => invoke<string>("install_sbx");
export const enableImagePaste = () => invoke<void>("enable_image_paste");
export const setupSsh = () => invoke<string>("setup_ssh");
export const login = () => invoke<string>("login");
export const listSandboxes = () => invoke<SandboxInfo[]>("sandboxes");
export const startSandbox = (name: string) => invoke<string>("start_sandbox", { name });
export const stopSandbox = (name: string) => invoke<string>("stop_sandbox", { name });
export const restartSandbox = (name: string) => invoke<string>("restart_sandbox", { name });
export const openTerminal = (name: string) => invoke<void>("open_terminal", { name });
export const connectionInfo = (name: string) =>
  invoke<ConnectionInfo>("connection_info", { name });
