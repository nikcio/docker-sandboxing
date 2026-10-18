import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface UpdateInfo {
  /** The version announced by the update endpoint. */
  version: string;
  /** Optional release notes. */
  notes: string | null;
}

export interface UpdateProgress {
  /** Bytes downloaded so far. */
  downloaded: number;
  /** Total bytes when known, `null` otherwise. */
  total: number | null;
}

export const checkForUpdate = () => invoke<UpdateInfo | null>("check_for_update");
export const installUpdate = () => invoke<void>("install_update");
export const currentAppVersion = () => invoke<string>("app_version");

/**
 * Subscribes to download progress for an in-flight update installation.
 * Resolves with an unsubscribe function.
 */
export const onUpdateProgress = (callback: (progress: UpdateProgress) => void) =>
  listen<UpdateProgress>("update://progress", (event) => callback(event.payload));
