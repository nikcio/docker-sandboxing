//! Setup status: whether the `sbx` CLI is installed, and how far the local
//! configuration (authentication, clipboard setting, SSH) has come.

use serde::Serialize;

use crate::sbx;

#[derive(Serialize)]
pub struct SetupStatus {
    pub sbx_installed: bool,
    // Unavailable checks are not evidence that the user signed out or disabled setup.
    pub signed_in: Option<bool>,
    pub image_paste_on: Option<bool>,
    pub ssh_set_up: Option<bool>,
    pub issues: Vec<String>,
}

pub(crate) fn read_status() -> Result<SetupStatus, String> {
    let sbx_installed = sbx::is_sbx_installed();
    if !sbx_installed {
        return Ok(SetupStatus {
            sbx_installed,
            signed_in: Some(false),
            image_paste_on: Some(false),
            ssh_set_up: Some(false),
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
    let signed_in = checked(sbx::is_signed_in(), &mut issues);
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
    let ssh_set_up = checked(sbx::is_ssh_set_up(), &mut issues);
    Ok(SetupStatus {
        sbx_installed,
        signed_in,
        image_paste_on,
        ssh_set_up,
        issues,
    })
}
