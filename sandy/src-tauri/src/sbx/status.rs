//! Health, authentication, daemon, and settings checks via `sbx diagnose`
//! and `sbx settings`.

use super::cli::{run_capture, sbx_command};

/// Machine-readable health/auth report from `sbx diagnose --format json`.
/// A nonzero exit can describe an unrelated failed check (e.g. missing KVM),
/// while Authentication and SSH client config are still authoritative.
fn diagnose_report() -> Result<Vec<DiagnoseCheck>, String> {
    let out = sbx_command(&["diagnose", "--format", "json"])?
        .output()
        .map_err(|e| format!("could not run sbx diagnose: {e}"))?;
    let report: DiagnoseReport = serde_json::from_slice(&out.stdout).map_err(|e| {
        format!(
            "sbx diagnose did not return a usable JSON report (exit {}): {e}\n{}",
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stderr).trim()
        )
    })?;
    Ok(report.checks)
}

#[derive(Debug, serde::Deserialize)]
struct DiagnoseReport {
    checks: Vec<DiagnoseCheck>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct DiagnoseCheck {
    name: String,
    status: String,
    message: String,
}

/// Whether the user is signed in to Docker, via the `Authentication` check
/// in `sbx diagnose`. The CLI has no `whoami` command; exit-code probing of
/// `login` would block on the browser flow, so diagnose is the only
/// non-interactive detector. Unknown status is an error, never an explicit
/// sign-out. Every call refreshes the report, including after login/logout.
pub fn is_signed_in() -> Result<bool, String> {
    ensure_daemon()?;
    check_configured(
        &diagnose_report()?,
        "Authentication",
        &["authenticated", "signed in"],
        &[
            "not signed in",
            "not authenticated",
            "unauthenticated",
            "signed out",
        ],
    )
}

fn check_configured(
    checks: &[DiagnoseCheck],
    name: &str,
    positive: &[&str],
    negative: &[&str],
) -> Result<bool, String> {
    let check = checks
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| format!("sbx diagnose did not include the {name} check"))?;
    let message = check.message.trim().to_ascii_lowercase();
    let status = check.status.to_ascii_lowercase();
    if matches!(status.as_str(), "pass" | "warn" | "fail") && negative.contains(&message.as_str()) {
        return Ok(false);
    }
    if check.status.eq_ignore_ascii_case("pass") && positive.contains(&message.as_str()) {
        return Ok(true);
    }
    Err(format!(
        "Could not determine {name}: {} ({})",
        check.message, check.status
    ))
}

fn daemon_running() -> Result<bool, String> {
    let out = sbx_command(&["daemon", "status", "--format", "json"])?
        .output()
        .map_err(|e| format!("could not check sbx daemon: {e}"))?;
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&out.stdout)
        && let Some(status) = value.get("status").and_then(|v| v.as_str())
    {
        return Ok(out.status.success() && status.eq_ignore_ascii_case("running"));
    }
    // Older CLIs may ignore --format. Match the complete status value, not
    // the substring "running" (which also matches "not running").
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(out.status.success()
        && text.lines().any(|line| {
            let value = line.trim().strip_prefix("Status:").unwrap_or(line).trim();
            value.eq_ignore_ascii_case("running")
        }))
}

/// Bring up the daemon before authentication/settings checks on every OS.
/// `--detach` is provided by sbx itself, so no unreaped foreground child is
/// left behind. Calls are serialized to avoid racing daemon startups.
pub fn ensure_daemon() -> Result<(), String> {
    static START_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = START_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if daemon_running()? {
        return Ok(());
    }
    let start = run_capture(&["daemon", "start", "--detach"]);
    // Another process may have started it while our start command failed.
    for attempt in 0..10 {
        if daemon_running()? {
            return Ok(());
        }
        if start.is_err() {
            break;
        }
        if attempt < 9 {
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
    }
    Err(start
        .err()
        .unwrap_or_else(|| "sbx daemon did not become ready after starting".to_string()))
}

/// Read a single sbx setting value (empty string if unset).
pub fn get_setting(key: &str) -> Result<String, String> {
    ensure_daemon()?;
    let out = run_capture(&["settings", "get", key])?;
    normalize_setting(key, &out)
}

fn normalize_setting(key: &str, out: &str) -> Result<String, String> {
    let text = out.trim();
    // Normal output is a scalar; --json/older versions may emit a setting
    // record. Normalize both so existing boolean callers can compare "true".
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
        let value = if value.is_object() {
            if let Some(actual_key) = value.get("key").and_then(|v| v.as_str())
                && actual_key != key
            {
                return Err(format!(
                    "sbx returned setting {actual_key} instead of {key}"
                ));
            }
            value
                .get("value")
                .or_else(|| value.get(key))
                .ok_or_else(|| format!("sbx returned no value for setting {key}"))?
        } else {
            &value
        };
        return match value {
            serde_json::Value::String(s) => Ok(s.clone()),
            serde_json::Value::Bool(_) | serde_json::Value::Number(_) => Ok(value.to_string()),
            serde_json::Value::Null => Ok(String::new()),
            _ => Err(format!("sbx returned a non-scalar value for setting {key}")),
        };
    }
    Ok(text.to_string())
}

/// `sbx settings set clipboard.imagePaste true`
pub fn set_clipboard_image_paste() -> Result<(), String> {
    ensure_daemon()
        .and_then(|_| run_capture(&["settings", "set", "clipboard.imagePaste", "true"]))
        .map(|_| ())
        .map_err(|e| format!("could not enable image paste:\n{}", e))
}

/// Ask the CLI instead of matching a comment in ~/.ssh/config: setup may
/// use an Include, platform-specific paths, or a generated managed block.
pub fn is_ssh_set_up() -> Result<bool, String> {
    ensure_daemon()?;
    check_configured(
        &diagnose_report()?,
        "SSH client config",
        &["configured"],
        &["not configured"],
    )
}

/// Open the browser for Docker OAuth and wait for completion. `sbx login`
/// exits 0 both when the flow completes and when the user is already
/// signed in (printing "You are signed in [username: …]") — so a zero exit
/// is success, and anything else is reported with the CLI's own output.
pub fn run_login() -> Result<String, String> {
    ensure_daemon()?;
    run_capture(&["login"])
}
