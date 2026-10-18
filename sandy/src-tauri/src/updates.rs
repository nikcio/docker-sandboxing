//! In-app update detection and installation.
//!
//! Sandy uses Tauri's signed updater: an update is only announced by a
//! manifest fetched over HTTPS, and the downloaded installer is verified
//! against a release public key baked into the binary before anything runs.
//! The frontend asks via [`check_for_update`] and [`install_update`]; all
//! blocking and network work happens on Tauri's async runtime.
//!
//! Development builds run from `tauri dev` with a placeholder version, so
//! update checks are disabled there — the check would compare `0.1.0`
//! against released versions and offer to replace the developer's binary.

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

/// How long a check request may run before giving up. The download is left
/// unbounded: reqwest's timeout covers the whole body read, and a slow
/// connection must not fail a large installer mid-download.
const CHECK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct UpdateInfo {
    version: String,
    notes: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case")]
struct UpdateProgress {
    downloaded: u64,
    total: Option<u64>,
}

fn check_updater(app: &AppHandle) -> Result<tauri_plugin_updater::Updater, String> {
    app.updater_builder()
        .timeout(CHECK_TIMEOUT)
        .build()
        .map_err(|error| error.to_string())
}

/// Like [`check_updater`], but for downloading and installing: no timeout, so
/// a large installer on a slow connection can finish.
fn install_updater(app: &AppHandle) -> Result<tauri_plugin_updater::Updater, String> {
    app.updater_builder()
        .build()
        .map_err(|error| error.to_string())
}

/// True when this is a development build (`tauri dev`), which reports the
/// placeholder manifest version instead of a released one.
fn is_dev_build() -> bool {
    cfg!(debug_assertions)
}

#[tauri::command]
pub(crate) async fn app_version(app: AppHandle) -> Result<String, String> {
    Ok(app.package_info().version.to_string())
}

/// Asks the update endpoint whether a newer release exists. Returns its
/// version and notes, or `None` when Sandy is up to date (or when checks are
/// disabled, e.g. in a development build). Fails with the updater's error so
/// the UI can show why the check did not succeed.
#[tauri::command]
pub(crate) async fn check_for_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    if is_dev_build() {
        return Ok(None);
    }
    let update = check_updater(&app)?
        .check()
        .await
        .map_err(|error| error.to_string())?;
    Ok(update.map(|update| UpdateInfo {
        version: update.version,
        notes: update.body,
    }))
}

/// Downloads, signature-verifies, and installs the announced update, then
/// relaunches Sandy. Progress is emitted as `update://progress` events.
///
/// On Windows the installer takes over and the app exits by itself; on macOS
/// and Linux the new version is in place when the relaunch happens.
#[tauri::command]
pub(crate) async fn install_update(app: AppHandle) -> Result<(), String> {
    let update = install_updater(&app)?
        .check()
        .await
        .map_err(|error| error.to_string())?;
    let Some(update) = update else {
        return Err("No update is available".into());
    };
    let handle = app.clone();
    let mut downloaded = 0u64;
    update
        .download_and_install(
            |chunk, total| {
                downloaded += chunk as u64;
                let _ = handle.emit("update://progress", UpdateProgress { downloaded, total });
            },
            || {},
        )
        .await
        .map_err(|error| error.to_string())?;
    // On Windows the installer exits the process before this returns; on
    // macOS and Linux `restart` never returns either — it relaunches Sandy
    // with the freshly installed version and terminates this instance.
    app.restart();
}
