//! Setup status: whether the `sbx` CLI is installed, and how far the local
//! configuration (authentication, clipboard setting, SSH) has come.

use serde::Serialize;

use crate::sbx::{self, CheckExplanation};

/// Run the full setup check: whether the `sbx` CLI is installed, and how far
/// the local configuration (authentication, clipboard setting, SSH) has come.
/// Every call spawns CLI processes (daemon probe, `sbx diagnose`, a settings
/// read), so call it only where the answer can change: once at startup, after
/// a setup action (wizard step, login, SSH setup), or on an explicit user
/// request ("check again"). Never poll it — sandbox data comes from `sbx ls`.
#[derive(Serialize)]
pub struct SetupStatus {
    pub sbx_installed: bool,
    // Unavailable checks are not evidence that the user signed out or disabled setup.
    pub signed_in: Option<bool>,
    pub image_paste_on: Option<bool>,
    pub ssh_set_up: Option<bool>,
    /// The CLI's own explanation of the Authentication verdict (message,
    /// detail, hint) — available even when the check is unavailable.
    pub auth_explanation: Option<CheckExplanation>,
    /// The CLI's own explanation of the SSH config verdict.
    pub ssh_explanation: Option<CheckExplanation>,
    /// A non-blocking note about a check whose verdict may be a false alarm
    /// (e.g. the SSH config check's known Windows false alarm).
    pub warnings: Vec<String>,
    pub issues: Vec<String>,
}

pub(crate) fn check_setup() -> Result<SetupStatus, String> {
    let sbx_installed = sbx::is_sbx_installed();
    if !sbx_installed {
        return Ok(SetupStatus {
            sbx_installed,
            signed_in: Some(false),
            image_paste_on: Some(false),
            ssh_set_up: Some(false),
            auth_explanation: None,
            ssh_explanation: None,
            warnings: Vec::new(),
            issues: Vec::new(),
        });
    }
    let mut issues = Vec::new();
    fn checked(result: Result<bool, String>, issues: &mut Vec<String>) -> Option<bool> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                issues.push(error);
                None
            }
        }
    }
    // One diagnose run answers both sign-in and SSH; a shared failure is
    // reported once.
    let checks = sbx::diagnose_setup_checks();
    let mut shared_issue: Option<String> = None;
    let signed_in = match checks.signed_in {
        Ok(value) => Some(value),
        Err(error) => {
            shared_issue = Some(error);
            None
        }
    };
    let ssh_set_up = match checks.ssh_set_up {
        Ok(value) => Some(value),
        Err(error) => {
            // The two checks fail together whenever the diagnose run itself
            // failed (identical errors). Differing errors mean two distinct
            // problems, so keep both.
            if shared_issue.as_deref() != Some(error.as_str()) {
                if let Some(previous) = shared_issue.take() {
                    issues.push(previous);
                }
                shared_issue = Some(error);
            }
            None
        }
    };
    if let Some(error) = shared_issue {
        issues.push(error);
    }
    let warnings: Vec<String> = checks.ssh_warning.into_iter().collect();
    let auth_explanation = checks.auth_explanation;
    let ssh_explanation = checks.ssh_explanation;
    let image_paste_on = checked(
        sbx::get_setting("clipboard.imagePaste").and_then(|value| {
            if value.eq_ignore_ascii_case("true") {
                Ok(true)
            } else if value.eq_ignore_ascii_case("false") {
                Ok(false)
            } else {
                Err("Could not determine whether clipboard image pasting is enabled".into())
            }
        }),
        &mut issues,
    );
    Ok(SetupStatus {
        sbx_installed,
        signed_in,
        image_paste_on,
        ssh_set_up,
        auth_explanation,
        ssh_explanation,
        warnings,
        issues,
    })
}
