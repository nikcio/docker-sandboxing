import type { SetupStatus } from "../../../lib/sandbox";
import Icon from "../../../common/components/Icon";
import Providers from "./Providers";
import Advanced from "./Advanced";
import SetupWizard from "../../setup/components/SetupWizard";
import { isSetupComplete } from "../../setup/setupSteps";
export interface SettingsPageProps {
  status: SetupStatus | null;
  statusError: string | null;
  busy: boolean;
  checkingSetup: boolean;
  /** Whether the setup wizard is currently open (vs. the summary card). */
  setupOpen: boolean;
  /** Bumped to remount the wizard for a fresh setup pass. */
  setupSession: number;
  /** Whether Settings is the visible page (providers only load when active). */
  active: boolean;
  onCheckSetup: () => Promise<SetupStatus | null>;
  onOpenSetup: () => void;
  onFinishSetup: () => void;
  onBusyChange: (busy: boolean) => void;
  onStepDone: (step: string) => void;
}

/**
 * The Settings area: setup summary or wizard, status banners, and provider
 * configuration. State orchestration stays in `App`; the wizard flow itself
 * lives in `features/setup`.
 */
export default function SettingsPage({
  status,
  statusError,
  busy,
  checkingSetup,
  setupOpen,
  setupSession,
  active,
  onCheckSetup,
  onOpenSetup,
  onFinishSetup,
  onBusyChange,
  onStepDone,
}: SettingsPageProps) {
  return (
    <div className="page standalone-page settings-page">
      <div className="page-head">
        <h1>Settings</h1>
        <p className="page-sub">Make Sandy feel at home on your computer.</p>
      </div>
      {statusError && (
        <div className="error-banner" role="alert">
          <strong>We couldn’t check your setup.</strong>
          <p>{statusError}</p>
          <button className="ghost" disabled={busy || checkingSetup} onClick={() => void onCheckSetup()}>
            Try again
          </button>
        </div>
      )}
      {!!status?.issues?.length && (
        <div className="error-banner" role="status">
          <strong>Some setup checks are temporarily unavailable.</strong>
          <p>{status.issues.join("\n")}</p>
          <button className="ghost" disabled={busy || checkingSetup} onClick={() => void onCheckSetup()}>
            Check again
          </button>
        </div>
      )}
      {!!status?.warnings?.length && (
        <div className="warning-banner" role="status">
          <strong>A setup check may be a false alarm.</strong>
          {status.warnings.map((warning) => (
            <p key={warning}>{warning}</p>
          ))}
        </div>
      )}
      {!!status?.ssh_explanation?.detail && status.ssh_explanation.detail !== status.ssh_explanation.message && (
        <details className="explanation-details">
          <summary>SSH check details</summary>
          <pre>{status.ssh_explanation.detail}</pre>
          {status.ssh_explanation.hint && <p>{status.ssh_explanation.hint}</p>}
        </details>
      )}
      {setupOpen ? (
        <SetupWizard
          key={setupSession}
          status={status}
          onCheckSetup={onCheckSetup}
          onDone={onFinishSetup}
          onBusyChange={onBusyChange}
          onStepDone={onStepDone}
        />
      ) : (
        <section className="card setup-summary" aria-busy={checkingSetup}>
          <span className="empty-icon green">
            <Icon name="shield" size={28} />
          </span>
          <div>
            <h2>
              {status &&
              [status.signed_in, status.image_paste_on, status.ssh_set_up].some(
                (value) => value === null,
              )
                ? "Setup status is temporarily unavailable"
                : isSetupComplete(status)
                  ? "You’re all set"
                  : "Let’s get you set up"}
            </h2>
            <p>
              {isSetupComplete(status)
                ? "Your local sandbox tools are configured and ready to use."
                : "A few simple steps to get your sandbox tools ready."}
            </p>
          </div>
          <button className="primary" disabled={busy || checkingSetup} onClick={onOpenSetup}>
            {checkingSetup ? (
              <>
                <span className="spinner" /> Checking setup…
              </>
            ) : (
              <>
                {isSetupComplete(status) ? "Review setup" : "Continue setup"}
                <Icon name="arrow" size={17} />
              </>
            )}
          </button>
        </section>
      )}
      {active && (
        <>
          <div className="card settings-info">
            <Providers disabled={busy || checkingSetup} />
          </div>
          <div className="card settings-info">
            <Advanced />
          </div>
        </>
      )}
    </div>
  );
}
