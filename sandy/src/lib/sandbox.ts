import { invoke } from "@tauri-apps/api/core";
import { shareInFlight } from "./inFlight";

/** The CLI's own explanation of a diagnose check's verdict. */
export interface CheckExplanation {
  message: string;
  detail?: string | null;
  hint?: string | null;
}

export interface SetupStatus {
  sbx_installed: boolean;
  signed_in: boolean | null;
  image_paste_on: boolean | null;
  ssh_set_up: boolean | null;
  /** The CLI's explanation of the Authentication verdict. */
  auth_explanation?: CheckExplanation | null;
  /** The CLI's explanation of the SSH config verdict. */
  ssh_explanation?: CheckExplanation | null;
  /** Non-blocking notes about checks whose verdicts may be false alarms. */
  warnings?: string[];
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

// The full setup check (daemon probe, diagnose, settings read) shares one
// invoke: StrictMode double effects and overlapping callers must not each
// spawn the same CLI work.
export const checkSetup = shareInFlight(() => invoke<SetupStatus>("check_setup"));
export const installSbx = () => invoke<string>("install_sbx");
export const enableImagePaste = () => invoke<void>("enable_image_paste");
export const setupSsh = () => invoke<string>("setup_ssh");
export const login = () => invoke<string>("login");
// Same dedupe as checkSetup: mount + focus can overlap on one list read.
export const listSandboxes = shareInFlight(() => invoke<SandboxInfo[]>("sandboxes"));
export const startSandbox = (name: string) => invoke<string>("start_sandbox", { name });
export const stopSandbox = (name: string) => invoke<string>("stop_sandbox", { name });
export const restartSandbox = (name: string) => invoke<string>("restart_sandbox", { name });
export const openTerminal = (name: string) => invoke<void>("open_terminal", { name });
export const connectionInfo = (name: string) =>
  invoke<ConnectionInfo>("connection_info", { name });
