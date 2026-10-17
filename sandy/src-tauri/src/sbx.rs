//! Integration with the `sbx` CLI (Docker Sandboxes).
//!
//! Sandy shells out to `sbx` for everything: installation checks, settings,
//! SSH setup, login, and listing local sandboxes.

use serde::Serialize;
use std::process::Command;

pub const SBX_COMMAND: &str = "sbx";
pub const SSH_SETUP_COMMAND: &str = "ssh";

#[derive(Debug, Serialize, Clone)]
pub struct SandboxInfo {
    pub name: String,
    pub state: String,
    pub agent: String,
    pub workspace: String,
    pub raw: String,
}

fn is_windows() -> bool {
    cfg!(windows)
}

/// On Windows, `sbx` is `sbx.cmd`/`sbx.exe` resolved through PATH; `.cmd`
/// shims need a shell, so route through cmd.exe there.
fn sbx_command(args: &[&str]) -> Command {
    if is_windows() {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg(SBX_COMMAND);
        for a in args {
            cmd.arg(a);
        }
        cmd
    } else {
        let mut cmd = Command::new(SBX_COMMAND);
        cmd.args(args);
        cmd
    }
}

pub fn is_sbx_installed() -> bool {
    which::which(SBX_COMMAND).is_ok()
}

/// Run `sbx` with args, capturing stdout (stderr is merged into the error
/// path only, so progress noise never pollutes parsed output).
pub fn run_capture(args: &[&str]) -> Result<String, String> {
    let output = sbx_command(args)
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

/// Whether the user is signed in to Docker (`sbx whoami` succeeds).
pub fn is_signed_in() -> Result<bool, String> {
    if !is_sbx_installed() {
        return Ok(false);
    }
    Ok(sbx_command(&["whoami"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false))
}

/// Read a single sbx setting value (empty string if unset).
pub fn get_setting(key: &str) -> Result<String, String> {
    let out = run_capture(&["settings", "get", key])?;
    Ok(out.trim().to_string())
}

/// `sbx settings set clipboard.imagePaste true`
pub fn set_clipboard_image_paste() -> Result<(), String> {
    run_capture(&["settings", "set", "clipboard.imagePaste", "true"])
        .map(|_| ())
        .map_err(|e| format!("could not enable image paste:\n{}", e))
}

/// Whether `sbx setup ssh` has been applied, detected by the managed block
/// in the user's SSH config. Matched loosely so small wording or
/// capitalization changes in the managed block don't break detection.
pub fn is_ssh_set_up() -> Result<bool, String> {
    let config_path = if is_windows() {
        std::env::var("USERPROFILE")
            .map(|p| format!("{}\\.ssh\\config", p))
            .map_err(|_| "USERPROFILE not set".to_string())?
    } else {
        std::env::var("HOME")
            .map(|p| format!("{}/.ssh/config", p))
            .map_err(|_| "HOME not set".to_string())?
    };
    match std::fs::read_to_string(&config_path) {
        Ok(content) => Ok(content.to_lowercase().contains("docker sandboxes")),
        Err(_) => Ok(false),
    }
}

/// Open the browser for Docker OAuth and wait for completion.
pub fn run_login() -> Result<String, String> {
    let out = run_capture(&["login"])?;
    if !is_signed_in()? {
        return Err(format!("login did not complete:\n{}", out));
    }
    Ok(out)
}

/// Install the sbx CLI for the current platform. Returns combined command
/// output for display in the wizard.
pub fn ensure_sbx_installed() -> Result<String, String> {
    if is_sbx_installed() {
        return Ok("sbx is already installed".to_string());
    }
    let mut log = String::new();

    #[cfg(target_os = "macos")]
    {
        // Homebrew is the documented install path on macOS.
        if which::which("brew").is_err() {
            return Err(
                "Homebrew is required to install sbx on macOS.\nInstall it from https://brew.sh and try again."
                    .to_string(),
            );
        }
        log.push_str(&run_via("brew", &["install", "docker/tap/sbx"])?);
        if !is_sbx_installed() {
            return Err(format!(
                "Installation finished but sbx was not found on PATH.\nOutput:\n{}",
                log
            ));
        }
    }

    #[cfg(target_os = "windows")]
    {
        // WinGet installs the per-user MSI and adds its bin directory to the
        // user PATH — but only for processes started afterwards. Sandy keeps
        // its current environment, so report success and let the user
        // restart instead of failing the step.
        log.push_str(&run_via(
            "powershell",
            &["-NoProfile", "-Command", "winget install -h Docker.sbx"],
        )?);
        log.push_str("\nRestart Sandy so the updated PATH is picked up.\n");
    }

    #[cfg(target_os = "linux")]
    {
        let mut push = |line: &str| log.push_str(line);
        install_sbx_linux(&mut push)?;
        if !is_sbx_installed() {
            return Err(format!(
                "Installation finished but sbx was not found on PATH.\nRestart Sandy (so the updated PATH is picked up) and try again.\nOutput:\n{}",
                log
            ));
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        return Err("This platform is not supported yet.".to_string());
    }

    Ok(log)
}

/// Linux (Ubuntu/Debian): add Docker's apt repository and install the
/// docker-sbx package, then add the current user to the kvm group for
/// local sandboxes. Mirrors https://docs.docker.com/ai/sandboxes/install/.
#[cfg(target_os = "linux")]
fn install_sbx_linux(log: &mut dyn FnMut(&str)) -> Result<(), String> {
    if !std::path::Path::new("/usr/bin/apt-get").exists() {
        return Err(
            "Only Ubuntu/Debian systems are supported by the automatic installer.\n\
             See https://docs.docker.com/ai/sandboxes/install/ for manual instructions."
                .to_string(),
        );
    }
    let script = r#"set -e
curl -fsSL https://get.docker.com -o /tmp/sandy-get-docker.sh
sudo REPO_ONLY=1 sh /tmp/sandy-get-docker.sh
rm -f /tmp/sandy-get-docker.sh
sudo apt-get install -y docker-sbx
sudo usermod -aG kvm "$(logname 2>/dev/null || id -un)" || echo "warning: could not add user to the kvm group" >&2
"#;
    log("Running the Docker install script (may ask for your password)...\n");
    let status = std::process::Command::new("bash")
        .arg("-c")
        .arg(script)
        .status()
        .map_err(|e| format!("failed to start installer: {}", e))?;
    if !status.success() {
        return Err(format!(
            "the Linux install script failed (exit {}). If no password prompt appeared, \
             Sandy may need to be started from a terminal once so sudo can ask for it.",
            status.code().unwrap_or(-1)
        ));
    }
    log("done\n");
    Ok(())
}

/// Run an external program (brew/powershell), capturing the tail of the
/// output for the wizard log.
#[cfg(any(target_os = "macos", target_os = "windows"))]
fn run_via(program: &str, args: &[&str]) -> Result<String, String> {
    use std::io::Read;
    use std::process::Stdio;

    let mut child = std::process::Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to run {}: {}", program, e))?;

    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let mut out = String::new();
    let t1 = std::thread::spawn(move || {
        let _ = stdout.read_to_string(&mut out);
        out
    });
    let t2 = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });
    let status = child
        .wait()
        .map_err(|e| format!("failed to wait for {}: {}", program, e))?;
    let stdout_text = t1.join().map_err(|_| "stdout thread panicked".to_string())?;
    let stderr_text = t2.join().map_err(|_| "stderr thread panicked".to_string())?;
    let combined = format!("{}{}", stdout_text, stderr_text);
    if !status.success() {
        return Err(format!(
            "{} failed (exit {}):\n{}",
            program,
            status.code().unwrap_or(-1),
            tail(&combined, 2000)
        ));
    }
    Ok(tail(&combined, 2000))
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn tail(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut start = s.len() - max;
    while !s.is_char_boundary(start) {
        start += 1;
    }
    format!("…{}", &s[start..])
}

/// List local sandboxes (`sbx ls`), parsed into rows. Errors propagate so
/// the UI can show them instead of inventing rows.
pub fn list_sandboxes() -> Result<Vec<SandboxInfo>, String> {
    let out = run_capture(&["ls"])?;
    if out.is_empty() {
        return Ok(vec![]);
    }
    Ok(parse_ls(&out))
}

fn parse_ls(out: &str) -> Vec<SandboxInfo> {
    let mut rows = Vec::new();
    let mut header_cols: Option<Vec<(usize, usize)>> = None;
    // Box-drawing tables carry interior pipes; once the header has them we
    // split data rows on pipes directly instead of header word spans.
    let mut pipe_table = false;

    for line in out.lines() {
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() || is_separator_line(trimmed) {
            continue;
        }
        // Only the first table header is recognized; later lines that happen
        // to contain "name" and "state" stay data rows.
        if header_cols.is_none() && is_header_line(trimmed) {
            pipe_table = trimmed.contains('│') || trimmed.contains('|');
            let header = strip_border_pipes(trimmed).replace(['│', '|'], " ");
            header_cols = Some(header_spans(&header));
            continue;
        }
        let cols = if pipe_table {
            strip_border_pipes(trimmed)
                .split(['│', '|'])
                .map(|c| c.trim().to_string())
                .collect()
        } else {
            match &header_cols {
                Some(spans) => split_by_spans(trimmed, spans),
                None => vec![trimmed.trim().to_string()],
            }
        };
        let get = |i: usize| cols.get(i).map(|s| s.trim().to_string()).unwrap_or_default();
        rows.push(SandboxInfo {
            name: get(0),
            state: get(1),
            agent: get(2),
            workspace: get(3),
            raw: trimmed.to_string(),
        });
    }
    rows
}

fn is_header_line(line: &str) -> bool {
    let lower = line.trim().trim_start_matches(['│', '|']).trim().to_lowercase();
    lower.starts_with("name") && lower.contains("state")
}

/// Remove a leading and trailing border pipe from a box-drawing table row
/// and trim the result.
fn strip_border_pipes(line: &str) -> &str {
    let t = line.trim();
    let stripped = t
        .trim_start_matches(['│', '|'])
        .trim_end_matches(['│', '|']);
    stripped.trim_end()
}

/// True for table decoration lines (┌───┬───┐, ├───┼───┤, +----+----+, ───).
fn is_separator_line(line: &str) -> bool {
    let mut has_separator = false;
    for c in line.trim().chars() {
        if c.is_whitespace() {
            continue;
        }
        if matches!(
            c,
            '─' | '━' | '-' | '=' | '+' | '┌' | '┐' | '└' | '┘' | '├' | '┤' | '┬' | '┴' | '┼' | '│' | '|'
        ) {
            has_separator = true;
        } else {
            return false;
        }
    }
    has_separator
}

/// Find the byte spans of each header word so data rows can be split on the
/// same columns.
fn header_spans(header: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in header.char_indices() {
        if c == ' ' {
            if let Some(s) = start.take() {
                spans.push((s, i));
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        spans.push((s, header.len()));
    }
    spans
}

fn split_by_spans(line: &str, spans: &[(usize, usize)]) -> Vec<String> {
    let mut out = Vec::new();
    for (i, (s, _e)) in spans.iter().enumerate() {
        let end = if i + 1 < spans.len() { spans[i + 1].0 } else { line.len() };
        // Use get() so a value wider than its header column (which shifts
        // byte offsets on multi-byte UTF-8) can never panic on a slice.
        out.push(line.get(*s..end).unwrap_or("").trim().to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typical_ls_table() {
        let sample = "\
NAME      STATE    AGENT    WORKSPACE
demo      running  opencode /home/me/project
build     stopped  claude   /home/me/api
";
        let rows = parse_ls(sample);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "demo");
        assert_eq!(rows[0].state, "running");
        assert_eq!(rows[0].agent, "opencode");
        assert_eq!(rows[1].name, "build");
        assert_eq!(rows[1].state, "stopped");
    }

    #[test]
    fn parses_box_drawing_table() {
        let sample = "\
┌───────┬──────────┬────────┐
│ NAME  │ STATE    │ AGENT  │
├───────┼──────────┼────────┤
│ alpha │ running  │ claude │
│ beta  │ creating │ codex  │
└───────┴──────────┴────────┘
";
        let rows = parse_ls(sample);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "alpha");
        assert_eq!(rows[0].state, "running");
        assert_eq!(rows[1].name, "beta");
        assert_eq!(rows[1].state, "creating");
    }

    #[test]
    fn utf8_values_do_not_panic() {
        let sample = "\
NAME     STATE    AGENT  WORKSPACE
demo     running  claude /home/ünïcødé/プロジェクト
";
        let rows = parse_ls(sample);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "demo");
    }

    #[test]
    fn data_row_with_name_and_state_words_is_not_a_header() {
        let sample = "\
NAME   STATE   AGENT
demo   running claude
weird  paused  /home/name/state
";
        let rows = parse_ls(sample);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].name, "weird");
    }

    #[test]
    fn single_row_no_header() {
        let rows = parse_ls("only-one-line");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "only-one-line");
    }

    #[test]
    fn empty_input() {
        assert!(parse_ls("").is_empty());
        assert!(parse_ls("  \n \n").is_empty());
    }
}
