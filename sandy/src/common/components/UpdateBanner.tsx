import Icon from "./Icon";
import type { AppUpdatesState } from "../hooks/useAppUpdates";

const MB = 1024 * 1024;

const formatProgress = (downloaded: number, total: number | null) => {
  const downloadedMb = (downloaded / MB).toFixed(1);
  if (total === null) return `${downloadedMb} MB downloaded…`;
  return `${downloadedMb} of ${(total / MB).toFixed(1)} MB`;
};

/**
 * Banner shown on the dashboard while an update is available or installing.
 * Update-check errors are never shown here: the automatic background check
 * must stay silent, and Settings surfaces failures on a manual check.
 */
export default function UpdateBanner({ updates }: { updates: AppUpdatesState }) {
  const { status, info, progress } = updates;
  if (status !== "available" && status !== "installing") return null;
  return (
    <div className="setup-banner update-banner" role="status">
      <Icon name="download" size={20} />
      {status === "installing" ? (
        <span>
          Downloading Sandy {info?.version}…
          {progress ? ` ${formatProgress(progress.downloaded, progress.total)}` : ""}
        </span>
      ) : (
        <>
          <span>
            <strong>Sandy {info?.version}</strong> is available. Updating keeps your sandboxes
            and settings.
          </span>
          <button className="text-button" onClick={updates.snooze}>
            Later
          </button>
          <button className="primary" onClick={() => void updates.install()}>
            Update now
          </button>
        </>
      )}
    </div>
  );
}
