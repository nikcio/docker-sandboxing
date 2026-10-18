import { useCallback, useEffect, useRef, useState } from "react";
import {
  checkForUpdate,
  installUpdate,
  onUpdateProgress,
  type UpdateInfo,
  type UpdateProgress,
} from "../../lib/updates";

export type UpdateStatus =
  | /** The first check has not settled yet. */ "checking"
  | /** No newer release, or the check ran in a development build. */ "up-to-date"
  | /** A newer release is available to install. */ "available"
  | /** The update is being downloaded and installed. */ "installing"
  | /** The update check or install failed. */ "error";

export interface AppUpdatesState {
  status: UpdateStatus;
  info: UpdateInfo | null;
  error: string | null;
  progress: UpdateProgress | null;
  check: () => Promise<void>;
  install: () => Promise<void>;
}

/** How long a dismissed update banner stays hidden, in milliseconds. */
const SNOOZE_MS = 24 * 60 * 60 * 1000;
const SNOOZE_KEY = "sandy.updateSnoozedVersion";

const snoozedVersion = () => {
  try {
    const raw = localStorage.getItem(SNOOZE_KEY);
    if (!raw) return null;
    const snoozed = JSON.parse(raw) as { version: string; until: number };
    if (Date.now() < snoozed.until) return snoozed.version;
    localStorage.removeItem(SNOOZE_KEY);
  } catch {
    localStorage.removeItem(SNOOZE_KEY);
  }
  return null;
};

/**
 * Update lifecycle for the whole app: one automatic check after boot, manual
 * re-checks, and installation with download progress. Development builds
 * never report an update (the backend skips the check), so the UI stays quiet.
 */
export function useAppUpdates(): AppUpdatesState {
  const [status, setStatus] = useState<UpdateStatus>("checking");
  const [info, setInfo] = useState<UpdateInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState<UpdateProgress | null>(null);
  const installing = useRef(false);

  const check = useCallback(async () => {
    try {
      const update = await checkForUpdate();
      setError(null);
      if (update) {
        setInfo(update);
        setStatus(snoozedVersion() === update.version ? "up-to-date" : "available");
      } else {
        setInfo(null);
        setStatus("up-to-date");
      }
    } catch (cause) {
      setInfo(null);
      setError(String(cause));
      setStatus("error");
    }
  }, []);

  const install = useCallback(async () => {
    if (installing.current) return;
    installing.current = true;
    setProgress(null);
    setError(null);
    setStatus("installing");
    try {
      // Resolves only on failure: success relaunches the app.
      await installUpdate();
    } catch (cause) {
      setError(String(cause));
      setStatus(info ? "available" : "error");
      installing.current = false;
    }
  }, [info]);

  // One automatic check shortly after boot, so the splash has settled and the
  // dashboard is up before a banner could appear.
  useEffect(() => {
    const timer = window.setTimeout(() => void check(), 2_500);
    return () => window.clearTimeout(timer);
  }, [check]);

  useEffect(
    () => {
      // `listen` resolves asynchronously; the cleanup drops both the pending
      // and the active subscription.
      let unlisten: (() => void) | null = null;
      let cancelled = false;
      onUpdateProgress((updateProgress) => {
        setProgress(updateProgress);
      }).then((stop) => {
        if (cancelled) {
          void stop();
        } else {
          unlisten = stop;
        }
      });
      return () => {
        cancelled = true;
        unlisten?.();
      };
    },
    [],
  );

  return { status, info, error, progress, check, install };
}

/** Marks the announced version as dismissed for the current user session day. */
export function snoozeUpdate(version: string) {
  try {
    localStorage.setItem(
      SNOOZE_KEY,
      JSON.stringify({ version, until: Date.now() + SNOOZE_MS }),
    );
  } catch {
    // A blocked localStorage only means the banner returns next launch.
  }
}
