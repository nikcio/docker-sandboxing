//! Health, authentication, daemon, and settings checks via `sbx diagnose`
//! and `sbx settings`.

use super::cli::{run_capture, sbx_command};

/// Machine-readable health/auth report from `sbx diagnose --format json`.
/// A nonzero exit can describe an unrelated failed check (e.g. missing KVM),
/// while Authentication and SSH client config are still authoritative.
fn diagnose_report() -> Result<Vec<DiagnoseCheck>, String> {
    let result = (|| {
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
    })();
    crate::diaglog::cli_call_with(&["diagnose", "--format", "json"], &result);
    result
}

#[derive(Debug, serde::Deserialize)]
struct DiagnoseReport {
    checks: Vec<DiagnoseCheck>,
}

#[derive(Debug, serde::Deserialize)]
struct DiagnoseCheck {
    name: String,
    status: String,
    message: String,
    #[serde(default)]
    detail: Option<String>,
    #[serde(default)]
    hint: Option<String>,
}

/// A diagnose check's human-facing explanation: the one-line message, the
/// deeper `detail`, and the CLI's suggested fix (`hint`). `detail` and
/// `hint` are optional; blank values collapse to `None`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CheckExplanation {
    pub message: String,
    pub detail: Option<String>,
    pub hint: Option<String>,
}

impl CheckExplanation {
    fn of(check: &DiagnoseCheck) -> Self {
        let trimmed = |value: &Option<String>| -> Option<String> {
            value
                .as_deref()
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .map(str::to_string)
        };
        Self {
            message: check.message.trim().to_string(),
            detail: trimmed(&check.detail),
            hint: trimmed(&check.hint),
        }
    }

    /// The explanation as a single line for issue/warning lists: the CLI's
    /// message first, then its detail and suggested fix when present.
    fn to_line(&self) -> String {
        let mut line = self.message.clone();
        if let Some(detail) = &self.detail {
            line.push_str(&format!(" — {detail}"));
        }
        if let Some(hint) = &self.hint {
            line.push_str(&format!(". {hint}"));
        }
        line
    }
}

/// The setup checks Sandy derives from a single `sbx diagnose` report.
/// Authentication and SSH client config live in the same report, so one run
/// answers both; when the run itself fails, both checks share that error.
#[derive(Debug)]
pub struct DiagnoseChecks {
    pub signed_in: Result<bool, String>,
    pub ssh_set_up: Result<bool, String>,
    /// The CLI's own explanation of the current Authentication verdict.
    pub auth_explanation: Option<CheckExplanation>,
    /// The CLI's own explanation of the current SSH config verdict.
    pub ssh_explanation: Option<CheckExplanation>,
    /// Set when the SSH verdict was downgraded from a `fail` to a
    /// "most likely fine" warning (see `ssh_check`).
    pub ssh_warning: Option<String>,
}

/// Bring up the daemon and run `sbx diagnose` once, deriving both setup
/// checks from the one report. Authentication comes from the `Authentication`
/// check — the CLI has no `whoami` command, and exit-code probing of `login`
/// would block on the browser flow, so diagnose is the only non-interactive
/// detector. The CLI's own `status` field decides: `pass` is set up, `fail`
/// is not — except for the SSH check, whose `fail` is downgraded (see
/// `ssh_check`). Unknown statuses are errors, never an explicit sign-out.
/// Every call refreshes the report, including after login/logout.
pub fn diagnose_setup_checks() -> DiagnoseChecks {
    let failure = |error: String| DiagnoseChecks {
        signed_in: Err(error.clone()),
        ssh_set_up: Err(error),
        auth_explanation: None,
        ssh_explanation: None,
        ssh_warning: None,
    };
    match ensure_daemon().and_then(|_| diagnose_report()) {
        Ok(checks) => {
            let (ssh_set_up, ssh_warning) = ssh_check(&checks);
            let named = |name: &str| {
                checks
                    .iter()
                    .find(|c| c.name.eq_ignore_ascii_case(name))
                    .map(CheckExplanation::of)
            };
            DiagnoseChecks {
                signed_in: check_status(&checks, "Authentication"),
                ssh_set_up,
                auth_explanation: named("Authentication"),
                ssh_explanation: named("SSH client config"),
                ssh_warning,
            }
        }
        Err(error) => failure(error),
    }
}

/// A check's CLI verdict from the `status` field: `pass` means the check is
/// satisfied, `fail` means it is not. Message text stays display-only:
/// matching CLI wording would break whenever the CLI rephrases it.
fn check_status(checks: &[DiagnoseCheck], name: &str) -> Result<bool, String> {
    let check = checks
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| format!("sbx diagnose did not include the {name} check"))?;
    match check.status.trim().to_ascii_lowercase().as_str() {
        "pass" => Ok(true),
        "fail" => Ok(false),
        _ => Err(format!(
            "Could not determine {name}: {} ({})",
            CheckExplanation::of(check).to_line(),
            check.status
        )),
    }
}

/// The SSH client config check's verdict and, for a `fail`, the user-facing
/// warning that downgrades it. The check has a known false alarm: adding any
/// `<name>.sbx` host to ~/.ssh/config makes it report `fail` even when
/// `sbx setup ssh` produced a working configuration. A `fail` therefore
/// counts as set up and is surfaced as a warning instead of an unfinishable
/// setup step.
fn ssh_check(checks: &[DiagnoseCheck]) -> (Result<bool, String>, Option<String>) {
    match check_status(checks, "SSH client config") {
        Ok(false) => {
            let explanation = checks
                .iter()
                .find(|c| c.name.eq_ignore_ascii_case("SSH client config"))
                .map(CheckExplanation::of);
            let problem = explanation
                .as_ref()
                .map(CheckExplanation::to_line)
                .unwrap_or_default();
            (
                Ok(true),
                Some(format!(
                    "The SSH client config check reported a problem ({problem}). This detection is not always accurate — if you have run the SSH setup command (sbx setup ssh), everything is most likely working as expected."
                )),
            )
        }
        other => (other, None),
    }
}

/// JSON-only: a human-text fallback would guess at CLI wording that can
/// change between versions, so an unreadable report is an explicit error
/// pointing at an upgrade instead of a silent misclassification.
fn daemon_running() -> Result<bool, String> {
    let result = (|| {
        let out = sbx_command(&["daemon", "status", "--format", "json"])?
            .output()
            .map_err(|e| format!("could not check sbx daemon: {e}"))?;
        let value: serde_json::Value = serde_json::from_slice(&out.stdout).map_err(|e| {
            format!(
                "sbx daemon status did not return a usable JSON report (the CLI may be too old — please upgrade sbx) (exit {}): {e}\n{}",
                out.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&out.stderr).trim()
            )
        })?;
        let status = value
            .get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                format!(
                    "sbx daemon status returned JSON without a \"status\" string field (the CLI may be too old — please upgrade sbx) (exit {}): {}",
                    out.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&out.stderr).trim()
                )
            })?;
        Ok(out.status.success() && status.eq_ignore_ascii_case("running"))
    })();
    crate::diaglog::cli_call_with(&["daemon", "status", "--format", "json"], &result);
    result
}

/// Bring up the daemon before authentication/settings checks on every OS.
/// `--detach` is provided by sbx itself, so no unreaped foreground child is
/// left behind. Calls are serialized to avoid racing daemon startups.
pub fn ensure_daemon() -> Result<(), String> {
    static START_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = START_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if daemon_recently_confirmed() {
        return Ok(());
    }
    if daemon_running()? {
        mark_daemon_confirmed();
        return Ok(());
    }
    let start = run_capture(&["daemon", "start", "--detach"]);
    // Another process may have started it while our start command failed.
    for attempt in 0..10 {
        if daemon_running()? {
            mark_daemon_confirmed();
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

/// How long a confirmed-running answer from `sbx daemon status` is reused.
/// Back-to-back checks (diagnose, settings, login in one status read) share
/// a single probe instead of each spawning its own; once the window passes,
/// the next check probes again. A daemon that dies inside the window still
/// fails loudly — through the very command that needed it.
const DAEMON_CONFIRMED_TTL: std::time::Duration = std::time::Duration::from_secs(5);

fn daemon_confirmation() -> &'static std::sync::Mutex<Option<std::time::Instant>> {
    static CONFIRMED_AT: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
    &CONFIRMED_AT
}

/// The active TTL. Test hook: command-count assertions must not depend on
/// the test body finishing inside the real 5-second window, so tests pin an
/// effectively infinite TTL.
#[cfg(test)]
fn confirmation_ttl() -> std::time::Duration {
    static TEST_TTL: std::sync::Mutex<Option<std::time::Duration>> = std::sync::Mutex::new(None);
    TEST_TTL
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .unwrap_or(DAEMON_CONFIRMED_TTL)
}

#[cfg(all(test, unix))]
pub(crate) fn set_daemon_confirmation_ttl(ttl: Option<std::time::Duration>) {
    static TEST_TTL: std::sync::Mutex<Option<std::time::Duration>> = std::sync::Mutex::new(None);
    *TEST_TTL.lock().unwrap_or_else(|e| e.into_inner()) = ttl;
}

fn daemon_recently_confirmed() -> bool {
    let confirmed = daemon_confirmation()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    #[cfg(test)]
    let ttl = confirmation_ttl();
    #[cfg(not(test))]
    let ttl = DAEMON_CONFIRMED_TTL;
    confirmed.is_some_and(|at| at.elapsed() < ttl)
}

fn mark_daemon_confirmed() {
    let mut confirmed = daemon_confirmation()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    *confirmed = Some(std::time::Instant::now());
}

/// Drop the cached daemon confirmation. Test hook: fixtures must not see
/// another test's confirmed daemon.
#[cfg(test)]
pub(crate) fn reset_daemon_cache() {
    let mut confirmed = daemon_confirmation()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    *confirmed = None;
}

/// Read a single sbx setting value (empty string if unset).
pub fn get_setting(key: &str) -> Result<String, String> {
    ensure_daemon()?;
    // Only the structured record is accepted: decorated plain-text output
    // (banners, warnings) must never leak into the parsed value, so there
    // is no plain-text fallback for CLIs that reject or ignore --json.
    normalize_setting(key, &run_capture(&["settings", "get", key, "--json"])?)
}

fn normalize_setting(key: &str, out: &str) -> Result<String, String> {
    // Only JSON is accepted: a scalar, or a setting record. Normalize both
    // so existing boolean callers can compare "true".
    let value = serde_json::from_str::<serde_json::Value>(out.trim())
        .map_err(|e| format!("sbx settings get {key} --json did not return JSON: {e}"))?;
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
    match value {
        serde_json::Value::String(s) => Ok(s.clone()),
        serde_json::Value::Bool(_) | serde_json::Value::Number(_) => Ok(value.to_string()),
        serde_json::Value::Null => Ok(String::new()),
        _ => Err(format!("sbx returned a non-scalar value for setting {key}")),
    }
}

/// `sbx settings set clipboard.imagePaste true`
pub fn set_clipboard_image_paste() -> Result<(), String> {
    ensure_daemon()
        .and_then(|_| run_capture(&["settings", "set", "clipboard.imagePaste", "true"]))
        .map(|_| ())
        .map_err(|e| format!("could not enable image paste:\n{}", e))
}

/// Open the browser for Docker OAuth and wait for completion. `sbx login`
/// exits 0 both when the flow completes and when the user is already
/// signed in (printing "You are signed in [username: …]") — so a zero exit
/// is success, and anything else is reported with the CLI's own output.
pub fn run_login() -> Result<String, String> {
    ensure_daemon()?;
    run_capture(&["login"])
}
