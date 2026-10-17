import { invoke } from "@tauri-apps/api/core";

export interface SetupStatus {
  sbx_installed: boolean;
  signed_in: boolean;
  image_paste_on: boolean;
  ssh_set_up: boolean;
}

export interface SandboxInfo {
  name: string;
  state: string;
  agent: string;
  workspace: string;
  raw: string;
}

export const getStatus = () => invoke<SetupStatus>("status");
export const installSbx = () => invoke<string>("install_sbx");
export const enableImagePaste = () => invoke<void>("enable_image_paste");
export const setupSsh = () => invoke<string>("setup_ssh");
export const login = () => invoke<string>("login");
export const listSandboxes = () => invoke<SandboxInfo[]>("sandboxes");
