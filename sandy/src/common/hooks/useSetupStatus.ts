import { useCallback, useEffect, useRef, useState } from "react";
import type { SetupStatus } from "../../lib/sandbox";
import { checkSetup } from "../../lib/sandbox";

export interface SetupStatusState {
  status: SetupStatus | null;
  statusError: string | null;
  /** True until the first setup check resolves. */
  loading: boolean;
  busy: boolean;
  setBusy: (busy: boolean) => void;
  /**
   * Run the setup check (a full `sbx` diagnostics batch). Only call this
   * where the answer can change: after a setup action, or on an explicit
   * user request. The sandbox list polls `sbx ls` on its own.
   */
  checkSetup: () => Promise<SetupStatus | null>;
  /** Whether a setup check is needed before the wizard opens. */
  checkingSetup: boolean;
  openSetup: () => Promise<SetupStatus | null>;
  /** Steps confirmed done by the wizard, trusted briefly while checks settle. */
  onStepDone: (step: string) => void;
}

type CompletedStep = "sbx_installed" | "signed_in" | "image_paste_on" | "ssh_set_up";

/**
 * The setup-status state machine: one setup check at startup decides the
 * landing view (setup wizard when incomplete), and later checks run only
 * when setup-relevant actions run — wizard steps, the connect dialog's SSH
 * check, the Settings "check again" buttons. Trusts recent wizard
 * confirmations for a grace period. Nothing polls: setup state only changes
 * through setup actions, and the sandbox list has its own `sbx ls` refresh.
 */
export function useSetupStatus(): SetupStatusState {
  const [status, setStatus] = useState<SetupStatus | null>(null);
  const [statusError, setStatusError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [checkingSetup, setCheckingSetup] = useState(false);
  const openingSetup = useRef(false);
  const completedSteps = useRef(new Map<CompletedStep, number>());

  const checkSetupNow = useCallback(async () => {
    try {
      const next = await checkSetup();
      // Successful commands are authoritative while the daemon's checks settle.
      // Drop each confirmation once observed, or after a short grace period, so
      // later logout/configuration changes still appear accurately.
      for (const [key, expires] of completedSteps.current) {
        if (next[key] === true || Date.now() >= expires) {
          completedSteps.current.delete(key);
        } else {
          next[key] = true;
        }
      }
      // A temporarily unavailable check must not erase a previously verified result.
      setStatus((previous) =>
        previous && next.sbx_installed
          ? {
              ...next,
              signed_in: next.signed_in ?? previous.signed_in,
              image_paste_on: next.image_paste_on ?? previous.image_paste_on,
              ssh_set_up: next.ssh_set_up ?? previous.ssh_set_up,
            }
          : next,
      );
      setStatusError(null);
      return next;
    } catch (error) {
      setStatusError(String(error));
      return null;
    }
  }, []);

  useEffect(() => {
    let active = true;
    void checkSetupNow().then(() => {
      if (!active) return;
      setLoading(false);
    });
    return () => {
      active = false;
    };
  }, [checkSetupNow]);

  const openSetup = async () => {
    if (openingSetup.current || busy) return null;
    openingSetup.current = true;
    setCheckingSetup(true);
    try {
      return await checkSetupNow();
    } finally {
      openingSetup.current = false;
      setCheckingSetup(false);
    }
  };

  const onStepDone = (step: string) => {
    const key = (
      {
        install: "sbx_installed",
        login: "signed_in",
        imagepaste: "image_paste_on",
        ssh: "ssh_set_up",
      } as const
    )[step as "install" | "login" | "imagepaste" | "ssh"];
    if (key) {
      completedSteps.current.set(key, Date.now() + 30_000);
      setStatus((previous) => (previous ? { ...previous, [key]: true } : previous));
    }
  };

  return {
    status,
    statusError,
    loading,
    busy,
    setBusy,
    checkSetup: checkSetupNow,
    checkingSetup,
    openSetup,
    onStepDone,
  };
}
