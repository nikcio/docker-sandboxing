//! Discovery of the `sbx` executable and the shared command runner.

use std::path::PathBuf;
use std::process::Command;

pub const SBX_COMMAND: &str = "sbx";
pub const SSH_SETUP_COMMAND: &str = "ssh";

pub(crate) fn sbx_executable() -> Result<PathBuf, String> {
    // Prefer the native executable over a PATH shim. Never send CLI arguments
    // through cmd /C: workspace paths and environment values can contain shell
    // metacharacters even when sandbox names have been validated.
    let mut candidates = Vec::new();
    #[cfg(windows)]
    if let Ok(path) = which::which("sbx.exe") {
        candidates.push(path);
    }
    if let Ok(path) = which::which(SBX_COMMAND) {
        #[cfg(windows)]
        let path = if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("cmd") || e.eq_ignore_ascii_case("bat"))
        {
            path.with_extension("exe")
        } else {
            path
        };
        candidates.push(path);
    }
    candidates.extend(standard_sbx_locations());
    first_executable(&candidates).ok_or_else(|| {
        "Native sbx executable not found. Install sbx in its standard location or add it to PATH (Windows requires sbx.exe, not a shell shim).".to_string()
    })
}

pub(crate) fn first_executable(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates.iter().find_map(|path| which::which(path).ok())
}

fn standard_sbx_locations() -> Vec<PathBuf> {
    // Finder apps do not inherit a shell's Homebrew PATH. Do not launch a
    // login shell or mutate the application's global environment to find sbx.
    #[cfg(target_os = "macos")]
    {
        vec!["/opt/homebrew/bin/sbx".into(), "/usr/local/bin/sbx".into()]
    }
    #[cfg(windows)]
    {
        // Documented per-user and machine MSI locations; this also finds a
        // just-installed CLI before Sandy has inherited an updated PATH.
        ["LOCALAPPDATA", "ProgramFiles"]
            .into_iter()
            .filter_map(std::env::var_os)
            .map(|root| PathBuf::from(root).join("DockerSandboxes/bin/sbx.exe"))
            .collect()
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        Vec::new()
    }
}

/// Suppress the console window Windows would otherwise allocate for a
/// console-subsystem child. Release builds run without a console
/// (`windows_subsystem = "windows"`), so every child — `sbx.exe`,
/// `powershell.exe` — would briefly flash a new terminal on screen.
/// Only for background/captured processes; an interactive terminal must
/// stay visible.
///
/// Builder-chain method returning `&mut Command`, mirroring
/// `CommandExt::creation_flags`, which it wraps on Windows. Because std's
/// builder methods also return `&mut Command`, a chain using this must end
/// in an owning call (`.spawn()`, `.output()`, ...) or the chain's value
/// borrows a temporary that dies at the end of the statement (E0716).
pub(crate) trait HideConsole {
    fn hide_console(&mut self) -> &mut Self;
}

#[cfg(windows)]
impl HideConsole for Command {
    fn hide_console(&mut self) -> &mut Self {
        use std::os::windows::process::CommandExt;

        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        self.creation_flags(CREATE_NO_WINDOW);
        self
    }
}

#[cfg(not(windows))]
impl HideConsole for Command {
    fn hide_console(&mut self) -> &mut Self {
        self
    }
}

/// Construct a direct CLI invocation; callers can also set `current_dir`.
pub(crate) fn sbx_command(args: &[&str]) -> Result<Command, String> {
    let mut cmd = Command::new(sbx_executable()?);
    cmd.args(args).hide_console();
    Ok(cmd)
}

pub fn is_sbx_installed() -> bool {
    sbx_executable().is_ok()
}

/// Run `sbx` with args, capturing stdout (stderr is merged into the error
/// path only, so progress noise never pollutes parsed output).
pub fn run_capture(args: &[&str]) -> Result<String, String> {
    let result = run_capture_inner(args);
    crate::diaglog::cli_call(args, &result);
    result
}

fn run_capture_inner(args: &[&str]) -> Result<String, String> {
    let output = sbx_command(args)?
        .output()
        .map_err(|e| format!("failed to run {}: {}", SBX_COMMAND, e))?;
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    if !output.status.success() {
        let mut msg = String::from_utf8_lossy(&output.stderr).to_string();
        if msg.trim().is_empty() {
            msg = text;
        }
        return Err(format!(
            "{} {} failed (exit {}):\n{}",
            SBX_COMMAND,
            args.join(" "),
            output.status.code().unwrap_or(-1),
            msg.trim()
        ));
    }
    Ok(text.trim().to_string())
}
