//! Sandbox lifecycle operations: start/stop/restart/remove, SSH connection
//! info, and the shared sandbox-name validation.

use serde::Serialize;

use super::cli::run_capture;
use super::listing::list_sandboxes;

/// Local names must remain safe CLI operands, SSH hosts, and terminal titles.
/// In particular, flags, control characters and shell syntax are never names.
pub(crate) fn validate_sandbox_name(name: &str) -> Result<(), String> {
    if !name
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphanumeric)
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
    {
        return Err("Invalid sandbox name: start with a letter or digit and use only letters, digits, '.', '_' and '-'.".to_string());
    }
    Ok(())
}

/// Resume an existing sandbox with its saved agent, kit and workspace data.
/// sbx has no `start` command. Do not supply an agent or workspace to `run`.
pub fn start_sandbox(name: &str) -> Result<String, String> {
    validate_sandbox_name(name)?;
    if !list_sandboxes()?.iter().any(|s| s.name == name) {
        return Err(format!("Local sandbox {name} does not exist"));
    }
    run_capture(&["run", "--detach", "--name", name])
}

pub fn stop_sandbox(name: &str) -> Result<String, String> {
    validate_sandbox_name(name)?;
    run_capture(&["stop", name])
}

pub fn restart_sandbox(name: &str) -> Result<String, String> {
    validate_sandbox_name(name)?;
    stop_sandbox(name)?;
    start_sandbox(name)
        .map_err(|e| format!("Sandbox {name} was stopped but could not be started:\n{e}"))
}

/// Remove just the requested sandbox, without an interactive CLI prompt.
pub fn remove_sandbox(name: &str) -> Result<String, String> {
    validate_sandbox_name(name)?;
    run_capture(&["rm", "--force", name])
}

#[derive(Debug, Serialize, Clone)]
pub struct ConnectionInfo {
    pub host: String,
    pub user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    pub command: String,
}

/// SSH is proxied over the daemon's local socket/named pipe, not TCP port 22.
/// `_default_user_` lets the image choose its user (not the host's username).
/// The host alias requires `sbx setup ssh`; connecting also resumes a stopped
/// sandbox. See https://docs.docker.com/ai/sandboxes/integrations/.
pub fn connection_info(name: &str) -> Result<ConnectionInfo, String> {
    validate_sandbox_name(name)?;
    let host = format!("{name}.sbx");
    Ok(ConnectionInfo {
        command: format!("ssh {host}"),
        host,
        user: "_default_user_".to_string(),
        port: None,
    })
}
