//! Opening the diagnostic log's folder in the platform file manager.
//!
//! The reveal is best-effort on desktop selection semantics: where the
//! platform file manager supports it, the log file itself is selected so it
//! stands out among its neighbors; otherwise the folder simply opens.

use std::path::Path;
use std::process::Command;

#[cfg(target_os = "windows")]
use super::super::sbx::cli::HideConsole;

/// Wait briefly for launch failures, then transfer ownership to a waiter.
/// A file-manager window may stay open for hours; dropping Child without
/// waiting leaks zombies on Unix. Mirrors `sbx::terminal::spawn_terminal`.
#[cfg(any(target_os = "linux", target_os = "windows"))]
fn spawn_manager(mut command: Command) -> Result<(), String> {
    command.stdin(std::process::Stdio::null());
    let mut child = command
        .spawn()
        .map_err(|e| format!("could not launch file manager: {e}"))?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    match child.try_wait() {
        Ok(Some(status)) if status.success() => Ok(()),
        Ok(Some(status)) => Err(format!(
            "file manager launcher failed (exit {})",
            status.code().unwrap_or(-1)
        )),
        result => {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
            result
                .map(|_| ())
                .map_err(|e| format!("could not check file manager launcher: {e}"))
        }
    }
}

/// Open the folder containing `path` in the platform file manager,
/// selecting the file where the platform supports it. The file need not
/// exist; the folder is still opened.
pub fn reveal_in_folder(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|directory| !directory.as_os_str().is_empty())
        .ok_or_else(|| format!("{} has no parent folder", path.display()))?;
    #[cfg(target_os = "linux")]
    {
        let mut failures = Vec::new();
        for (opener, args) in [
            // Each of these accepts a file to select, not a shell command
            // string. `gvfs-open`/`xdg-open` accept a file or folder, but
            // cannot select, so they open the folder itself as a fallback.
            (
                "nautilus",
                vec!["--select", path.as_os_str().to_string_lossy().as_ref()],
            ),
            (
                "dolphin",
                vec!["--select", path.as_os_str().to_string_lossy().as_ref()],
            ),
            ("xdg-open", vec![parent.to_string_lossy().as_ref()]),
        ] {
            if let Ok(program) = which::which(opener) {
                let mut command = Command::new(program);
                command.args(args);
                match spawn_manager(command) {
                    Ok(()) => return Ok(()),
                    Err(e) => failures.push(format!("{opener}: {e}")),
                }
            }
        }
        Err(format!(
            "Could not open a file manager. Install one of Nautilus, Dolphin, or xdg-open.\n{}",
            failures.join("\n")
        ))
    }
    #[cfg(target_os = "macos")]
    {
        // Finder reveals and selects the file; the file need not exist for
        // `reveal` to resolve it.
        let quoted = format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"));
        let script = format!("tell application \"Finder\"\nactivate\nreveal {quoted}\nend tell");
        let mut command = Command::new("/usr/bin/osascript");
        command.args(["-e", &script]);
        // osascript exits after the Apple event; report automation permission
        // failures synchronously rather than silently swallowing them.
        let output = command
            .output()
            .map_err(|e| format!("could not open Finder: {e}"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "could not open Finder: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }
    #[cfg(target_os = "windows")]
    {
        // Explorer's /select switch takes an unquoted comma-separated list;
        // a single quoted path works. Quote for PowerShell so spaces and
        // apostrophes survive the extra hop.
        let quoted = format!("'{}'", path.to_string_lossy().replace('\'', "''"));
        let script = format!("explorer.exe /select,{quoted}");
        let mut command = Command::new("powershell.exe");
        command
            .args(["-NoProfile", "-Command", &script])
            .hide_console();
        // Explorer's launcher stub exits before the window exists, so the
        // exit status cannot distinguish success; the spawn itself is the
        // signal.
        command
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("could not open Explorer: {e}"))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = parent;
        Err("Revealing files is not supported on this platform".to_string())
    }
}
