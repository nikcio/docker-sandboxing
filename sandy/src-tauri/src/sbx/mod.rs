//! Integration with the `sbx` CLI (Docker Sandboxes).
//!
//! Sandy shells out to `sbx` for everything: installation checks, settings,
//! SSH setup, login, and listing local sandboxes. The submodules own one
//! concern each:
//!
//! - [`cli`]: locating the executable and running commands
//! - [`status`]: daemon, authentication, and settings checks (with short
//!   reuse of a confirmed daemon probe so one status read spawns one probe)
//! - [`lifecycle`]: start/stop/restart/remove and connection info
//! - [`listing`]: parsing `sbx ls` output
//! - [`terminal`]: opening an interactive terminal into a sandbox
//! - [`install`]: installing the CLI per platform

mod cli;
mod install;
mod lifecycle;
mod listing;
mod status;
mod terminal;

#[cfg(test)]
#[allow(unsafe_code)]
pub(crate) mod test_env;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests;

pub use cli::{SSH_SETUP_COMMAND, is_sbx_installed, run_capture};
pub use install::ensure_sbx_installed;
pub use lifecycle::{
    ConnectionInfo, connection_info, remove_sandbox, restart_sandbox, start_sandbox, stop_sandbox,
};
pub use listing::{SandboxInfo, list_sandboxes};
pub use status::{
    CheckExplanation, diagnose_setup_checks, get_setting, run_login, set_clipboard_image_paste,
};
pub use terminal::open_terminal;
