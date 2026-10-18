import Icon from "../../../common/components/Icon";
import type { AppUpdatesState } from "../../../common/hooks/useAppUpdates";

const headline = (status: AppUpdatesState["status"]) =>
  ({
    checking: "Checking for updates…",
    "up-to-date": "Sandy is up to date",
    available: "An update is available",
    installing: "Updating Sandy…",
    error: "We couldn’t check for updates",
  })[status];

const detail = (updates: AppUpdatesState) => {
  switch (updates.status) {
    case "available":
      // A failed install returns to "available"; show why it failed.
      if (updates.error) {
        return `The last install attempt failed: ${updates.error} You can retry from the button on the right.`;
      }
      return `Sandy ${updates.info?.version} is ready to install. Updating keeps your sandboxes, templates, and settings.`;
    case "installing":
      return "The download is verified against Sandy's release signature before anything is installed. Sandy restarts when it's done.";
    case "error":
      return "Check your internet connection and try again.";
    default:
      return "Sandy checks for updates automatically after it starts. You can also check manually.";
  }
};

/**
 * Settings card for in-app updates: shows the current state and offers a
 * manual check and, when available, the install action.
 */
export default function UpdatesCard({ updates }: { updates: AppUpdatesState }) {
  const busy = updates.status === "checking" || updates.status === "installing";
  const isError = updates.status === "error";
  return (
    <section className="card settings-info" aria-busy={busy}>
      <span className={isError ? "empty-icon" : "empty-icon green"}>
        <Icon name={isError ? "help" : "download"} size={28} />
      </span>
      <div>
        <h2>{headline(updates.status)}</h2>
        <p>{detail(updates)}</p>
        {updates.status === "error" && updates.error && (
          <p className="update-error">{updates.error}</p>
        )}
      </div>
      {updates.status === "available" ? (
        <button className="primary" onClick={() => void updates.install()}>
          Update to {updates.info?.version}
          <Icon name="arrow" size={17} />
        </button>
      ) : (
        <button className="primary" disabled={busy} onClick={() => void updates.check(true)}>
          {updates.status === "checking" ? (
            <>
              <span className="spinner" /> Checking…
            </>
          ) : (
            "Check for updates"
          )}
        </button>
      )}
    </section>
  );
}
