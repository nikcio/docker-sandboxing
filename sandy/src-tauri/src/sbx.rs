//! Integration with the `sbx` CLI (Docker Sandboxes).
//!
//! Sandy shells out to `sbx` for everything: installation checks, settings,
//! SSH setup, login, and listing local sandboxes.

use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;
#[cfg(target_os = "linux")]
use std::process::Stdio;

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

fn sbx_executable() -> Result<PathBuf, String> {
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

fn first_executable(candidates: &[PathBuf]) -> Option<PathBuf> {
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

#[cfg(target_os = "macos")]
fn homebrew_executable() -> Result<PathBuf, String> {
    which::which("brew").ok().or_else(|| {
        first_executable(&["/opt/homebrew/bin/brew".into(), "/usr/local/bin/brew".into()])
    }).ok_or_else(|| {
        "Homebrew is required to install sbx on macOS.\nInstall it from https://brew.sh and try again.".to_string()
    })
}

/// Construct a direct CLI invocation; callers can also set `current_dir`.
pub(crate) fn sbx_command(args: &[&str]) -> Result<Command, String> {
    let mut cmd = Command::new(sbx_executable()?);
    cmd.args(args);
    Ok(cmd)
}

pub fn is_sbx_installed() -> bool {
    sbx_executable().is_ok()
}

/// Run `sbx` with args, capturing stdout (stderr is merged into the error
/// path only, so progress noise never pollutes parsed output).
pub fn run_capture(args: &[&str]) -> Result<String, String> {
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
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
        if let Some(status) = value.get("status").and_then(|v| v.as_str()) {
            return Ok(out.status.success() && status.eq_ignore_ascii_case("running"));
        }
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
            if let Some(actual_key) = value.get("key").and_then(|v| v.as_str()) {
                if actual_key != key {
                    return Err(format!(
                        "sbx returned setting {actual_key} instead of {key}"
                    ));
                }
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

#[cfg(any(test, target_os = "macos"))]
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(any(test, target_os = "macos"))]
fn applescript_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(any(test, target_os = "windows"))]
fn powershell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// Wait briefly for launch failures, then transfer ownership to a waiter.
/// Terminal sessions may last hours; dropping Child without waiting leaks
/// zombies on Unix. No pipes are held open by the long-lived terminal.
#[cfg(any(target_os = "linux", target_os = "windows"))]
fn spawn_terminal(mut command: Command) -> Result<(), String> {
    // A Windows CREATE_NEW_CONSOLE must get real console handles, rather
    // than explicitly redirecting its interactive PowerShell to NUL.
    #[cfg(not(windows))]
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = command
        .spawn()
        .map_err(|e| format!("could not launch terminal: {e}"))?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    match child.try_wait() {
        Ok(Some(status)) if status.success() => Ok(()),
        Ok(Some(status)) => Err(format!(
            "terminal launcher failed (exit {})",
            status.code().unwrap_or(-1)
        )),
        result => {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
            result
                .map(|_| ())
                .map_err(|e| format!("could not check terminal launcher: {e}"))
        }
    }
}

/// Open a native interactive terminal, using the absolute CLI path so GUI
/// applications on macOS/Windows need not share Sandy's PATH. `sbx exec`
/// resumes stopped sandboxes and does not require SSH setup.
pub fn open_terminal(name: &str) -> Result<(), String> {
    validate_sandbox_name(name)?;
    let executable = sbx_executable()?;
    #[cfg(target_os = "linux")]
    {
        let mut failures = Vec::new();
        // Each of these accepts a program and argv, rather than a shell
        // command string. Avoid terminal flags such as `--command` that
        // concatenate/reparse arguments.
        for (terminal, separator) in [
            ("x-terminal-emulator", "-e"),
            ("gnome-terminal", "--"),
            ("konsole", "-e"),
            ("xfce4-terminal", "-x"),
            ("xterm", "-e"),
        ] {
            if let Ok(program) = which::which(terminal) {
                let mut command = Command::new(program);
                command
                    .arg(separator)
                    .arg(&executable)
                    .args(["exec", "-it", name, "/bin/bash"]);
                match spawn_terminal(command) {
                    Ok(()) => return Ok(()),
                    Err(e) => failures.push(format!("{terminal}: {e}")),
                }
            }
        }
        Err(format!("Could not open a terminal. Install a supported terminal (x-terminal-emulator, GNOME Terminal, Konsole, XFCE Terminal, or xterm).\n{}", failures.join("\n")))
    }
    #[cfg(target_os = "macos")]
    {
        let executable = executable.to_str().ok_or("sbx path is not valid UTF-8")?;
        let shell_command = [executable, "exec", "-it", name, "/bin/bash"]
            .map(shell_quote)
            .join(" ");
        let script = format!(
            "tell application \"Terminal\"\nactivate\ndo script {}\nend tell",
            applescript_quote(&format!("exec {shell_command}"))
        );
        let mut command = Command::new("/usr/bin/osascript");
        command.args(["-e", &script]);
        // osascript exits after the Apple event; report automation permission
        // failures synchronously rather than silently swallowing them.
        let output = command
            .output()
            .map_err(|e| format!("could not open Terminal: {e}"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "could not open Terminal: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(program) = which::which("wt.exe") {
            let mut command = Command::new(program);
            command.args(["new-tab", "--"]).arg(&executable).args([
                "exec",
                "-it",
                name,
                "/bin/bash",
            ]);
            if spawn_terminal(command).is_ok() {
                return Ok(());
            }
        }
        use std::os::windows::process::CommandExt;
        let executable = executable.to_str().ok_or("sbx path is not valid UTF-8")?;
        let script = format!(
            "& {}",
            [executable, "exec", "-it", name, "/bin/bash"]
                .map(powershell_quote)
                .join(" ")
        );
        let mut command = Command::new("powershell.exe");
        command
            .args(["-NoProfile", "-NoExit", "-Command", &script])
            .creation_flags(0x00000010); // CREATE_NEW_CONSOLE
        spawn_terminal(command)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = executable;
        Err("Opening a terminal is not supported on this platform".to_string())
    }
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
        let brew = homebrew_executable()?;
        let brew = brew.to_str().ok_or("Homebrew path is not valid UTF-8")?;
        log.push_str(&run_via(brew, &["install", "docker/tap/sbx"])?);
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
    let stdout_text = t1
        .join()
        .map_err(|_| "stdout thread panicked".to_string())?;
    let stderr_text = t2
        .join()
        .map_err(|_| "stderr thread panicked".to_string())?;
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

/// List local sandboxes. Prefers `sbx ls --json` (stable, machine-readable)
/// and falls back to parsing the human table if JSON is unavailable or
/// entirely unparseable (older CLI versions).
pub fn list_sandboxes() -> Result<Vec<SandboxInfo>, String> {
    let json_result = run_capture(&["ls", "--json"]).and_then(|out| {
        // Tolerate individual bad entries instead of discarding the whole
        // payload: parse as raw values, keep the ones that deserialize.
        let values: Vec<serde_json::Value> = serde_json::from_str(&out)
            .map_err(|e| format!("could not parse sbx ls --json output: {}", e))?;
        Ok(values
            .into_iter()
            .filter_map(|v| serde_json::from_value::<SandboxJson>(v).ok())
            .map(|s| SandboxInfo {
                name: s.name,
                state: s.status.unwrap_or_default(),
                agent: s.agent.unwrap_or_default(),
                workspace: s.workspaces.unwrap_or_default().join(", "),
                raw: s.id.unwrap_or_default(),
            })
            .collect())
    });
    match json_result {
        Ok(rows) => Ok(rows),
        Err(json_err) => match run_capture(&["ls"]) {
            Ok(out) if !out.is_empty() => Ok(parse_ls(&out)),
            Ok(_) => Err(json_err),
            Err(_) => Err(json_err),
        },
    }
}

/// Schema of `sbx ls --json` entries. Unknown fields are ignored; optional
/// fields degrade to empty so a schema addition never breaks the UI.
#[derive(Debug, serde::Deserialize)]
struct SandboxJson {
    name: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    workspaces: Option<Vec<String>>,
}

fn parse_ls(out: &str) -> Vec<SandboxInfo> {
    let mut rows = Vec::new();
    let mut header_cols: Option<Vec<(usize, usize)>> = None;
    // Maps logical columns (name/state/agent/workspace) to their index in
    // the header row, since `sbx ls` column names and order vary.
    let mut column_map: Option<std::collections::HashMap<&'static str, usize>> = None;
    // Box-drawing tables carry interior pipes; once the header has them we
    // split data rows on pipes directly instead of header word spans.
    let mut pipe_table = false;

    for line in out.lines() {
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() || is_separator_line(trimmed) {
            continue;
        }
        // Only the first table header is recognized; later lines that happen
        // to look like headers stay data rows.
        if header_cols.is_none() && is_header_line(trimmed) {
            pipe_table = trimmed.contains('│') || trimmed.contains('|');
            let header = strip_border_pipes(trimmed).replace(['│', '|'], " ");
            let spans = header_spans(&header);
            let names = header_spans(&header)
                .iter()
                .map(|(s, e)| header[*s..*e].to_uppercase())
                .collect::<Vec<_>>();
            column_map = Some(map_columns(&names));
            header_cols = Some(spans);
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
        let col = |name: &str| -> String {
            match column_map.as_ref().and_then(|m| m.get(name).copied()) {
                Some(i) => cols
                    .get(i)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default(),
                // No recognized header: treat the whole line as the name so
                // single-line output still shows something sensible.
                None if name == "name" => cols
                    .first()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default(),
                None => String::new(),
            }
        };
        rows.push(SandboxInfo {
            name: col("name"),
            state: col("state"),
            agent: col("agent"),
            workspace: col("workspace"),
            raw: trimmed.to_string(),
        });
    }
    rows
}

fn is_header_line(line: &str) -> bool {
    let lower = line
        .trim()
        .trim_start_matches(['│', '|'])
        .trim()
        .to_lowercase();
    // Known header shapes: the original NAME/STATE/AGENT/WORKSPACE layout
    // and the real v0.48 SANDBOX/AGENT/STATUS/PORTS/WORKSPACE/CREATED one.
    (lower.starts_with("name") && lower.contains("state"))
        || (lower.starts_with("sandbox") && lower.contains("status"))
}

/// Map header column names to the logical fields the UI shows. Recognizes
/// both the SANDBOX/STATUS and NAME/STATE vocabularies, plus aliases.
fn map_columns(names: &[String]) -> std::collections::HashMap<&'static str, usize> {
    let mut map = std::collections::HashMap::new();
    for (i, raw) in names.iter().enumerate() {
        let name = raw.trim();
        match name {
            "NAME" | "SANDBOX" => {
                map.entry("name").or_insert(i);
            }
            "STATE" | "STATUS" => {
                map.entry("state").or_insert(i);
            }
            "AGENT" => {
                map.entry("agent").or_insert(i);
            }
            "WORKSPACE" | "WORKSPACES" => {
                map.entry("workspace").or_insert(i);
            }
            _ => {}
        }
    }
    map
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
            '─' | '━'
                | '-'
                | '='
                | '+'
                | '┌'
                | '┐'
                | '└'
                | '┘'
                | '├'
                | '┤'
                | '┬'
                | '┴'
                | '┼'
                | '│'
                | '|'
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
    // Column boundaries are the header words' start offsets. A cell's value
    // is often wider than its header, so a cell ends at the last
    // non-whitespace before the next column's start — not at the next
    // header word's start itself.
    let mut out = Vec::new();
    for (i, (s, _e)) in spans.iter().enumerate() {
        let next_start = if i + 1 < spans.len() {
            spans[i + 1].0
        } else {
            line.len()
        };
        let mut end = next_start.min(line.len());
        while end > *s && end > 0 && line[..end].ends_with(' ') {
            end -= 1;
        }
        out.push(line.get(*s..end).unwrap_or("").trim().to_string());
    }
    out
}

// Shared by every test module that mutates the CLI environment.
#[cfg(test)]
pub(crate) static CLI_TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
    fn parses_real_v048_ls_table() {
        // Actual `sbx ls` (v0.48) layout: SANDBOX/AGENT/STATUS/PORTS/
        // WORKSPACE/CREATED, values wider than headers, bullet markers.
        let sample = "\
SANDBOX           AGENT            STATUS     PORTS                       WORKSPACE                           CREATED
my-app            my-agent         • running  127.0.0.1:60000->9418/tcp4  C:\\Dev\\MyProject                   1h
second            my-agent         • running  -                           -                                   4h
another-sandbox   my-agent         ○ stopped  -                           C:\\Dev\\OtherProject                1w
latest            shell            ○ stopped  -                           C:\\Dev\\SandboxTester               43m
";
        let rows = parse_ls(sample);
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0].name, "my-app");
        assert_eq!(rows[0].state, "• running");
        assert_eq!(rows[0].agent, "my-agent");
        assert_eq!(rows[0].workspace, "C:\\Dev\\MyProject");
        assert_eq!(rows[2].name, "another-sandbox");
        assert_eq!(rows[2].state, "○ stopped");
        assert_eq!(rows[3].name, "latest");
        assert_eq!(rows[3].workspace, "C:\\Dev\\SandboxTester");
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

    // ---- Integration tests against a mock `sbx` executable ----
    //
    // Each fixture owns its HOME, state, executable and entire PATH. Nothing
    // falls back to a real sbx, terminal emulator or user configuration.

    #[cfg(unix)]
    struct MockCli {
        root: PathBuf,
        environment: Vec<(&'static str, Option<std::ffi::OsString>)>,
    }

    #[cfg(unix)]
    impl MockCli {
        fn new() -> Self {
            use std::os::unix::fs::PermissionsExt;
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let root = std::env::temp_dir().join(format!(
                "sandy cli fixture-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            std::fs::create_dir(&root).expect("create isolated CLI fixture");
            let path = root.join("sbx");
            std::fs::write(&path, MOCK_CLI).expect("write fake CLI");
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
            let keys = [
                "PATH",
                "HOME",
                "USERPROFILE",
                "XDG_STATE_HOME",
                "SBX_TEST_ROOT",
                "SBX_MOCK_FAIL",
                "SBX_MOCK_SIGNED_IN",
                "SBX_MOCK_DAEMON",
                "SBX_MOCK_PASTE",
                "SBX_MOCK_FAIL_JSON",
                "SBX_MOCK_LS_EMPTY",
                "SBX_MOCK_JSON",
                "SBX_MOCK_DIAGNOSE",
                "SBX_MOCK_DIAGNOSE_EXIT",
                "SBX_MOCK_AUTH_MESSAGE",
                "SBX_MOCK_AUTH_STATUS",
                "SBX_MOCK_SSH",
                "SBX_MOCK_STOP_FAIL",
                "SBX_MOCK_RUN_FAIL",
                "SBX_MOCK_START_FAIL",
            ];
            let environment = keys
                .into_iter()
                .map(|key| {
                    let original = std::env::var_os(key);
                    std::env::remove_var(key);
                    (key, original)
                })
                .collect();
            for key in [
                "PATH",
                "HOME",
                "USERPROFILE",
                "XDG_STATE_HOME",
                "SBX_TEST_ROOT",
            ] {
                std::env::set_var(key, &root);
            }
            Self { root, environment }
        }

        fn commands(&self) -> String {
            std::fs::read_to_string(self.root.join("commands")).unwrap_or_default()
        }

        fn clear_commands(&self) {
            std::fs::write(self.root.join("commands"), "").unwrap();
        }
    }

    #[cfg(unix)]
    impl Drop for MockCli {
        fn drop(&mut self) {
            for (key, value) in &self.environment {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[cfg(unix)]
    const MOCK_CLI: &str = r##"#!/bin/sh
printf '%s\n' "$*" >> "$SBX_TEST_ROOT/commands"
if [ "$SBX_MOCK_FAIL" = 1 ]; then
    printf '%s\n' 'sbx mock: simulated failure' >&2
    exit 1
fi
case "$1" in
daemon)
    if [ "$2" = start ]; then
        [ "$SBX_MOCK_START_FAIL" = 1 ] && exit 1
        printf ready > "$SBX_TEST_ROOT/daemon-ready"
    else
        if [ "$SBX_MOCK_DAEMON" = running ] || [ -f "$SBX_TEST_ROOT/daemon-ready" ]; then
            printf '%s\n' '{"status":"running"}'
        else
            printf '%s\n' '{"status":"not running"}'
        fi
    fi
    ;;
diagnose)
    case "$SBX_MOCK_DIAGNOSE" in
    invalid) printf '%s\n' 'not a report'; exit 1 ;;
    missing) printf '%s\n' '{"checks":[]}'; exit 0 ;;
    esac
    if [ "$SBX_MOCK_SIGNED_IN" = 1 ]; then auth=authenticated; else auth='not signed in'; fi
    printf '{"checks":[{"name":"Virtualization","status":"fail","message":"no hypervisor present"},{"name":"Authentication","status":"%s","message":"%s"},{"name":"SSH client config","status":"pass","message":"%s"}]}\n' "${SBX_MOCK_AUTH_STATUS:-pass}" "${SBX_MOCK_AUTH_MESSAGE:-$auth}" "${SBX_MOCK_SSH:-not configured}"
    exit "${SBX_MOCK_DIAGNOSE_EXIT:-0}"
    ;;
settings)
    if [ "$2" = get ]; then printf '%s\n' "${SBX_MOCK_PASTE:-false}"; fi
    ;;
ls)
    if [ "$SBX_MOCK_LS_EMPTY" = 1 ]; then printf '%s\n' '[]'; exit 0; fi
    if [ "$2" = --json ]; then
        if [ "$SBX_MOCK_FAIL_JSON" = 1 ]; then printf '%s\n' 'bad json'; exit 0; fi
        case "$SBX_MOCK_JSON" in
        sparse) printf '%s\n' '[{"name":"bare"}]' ;;
        extra) printf '%s\n' '[{"name":"with-unknown","status":"running","future_field":123}]' ;;
        unicode) printf '%s\n' '[{"name":"ünïcødé-サンドボックス","workspaces":["/home/ünïcødé/プロジェクト"]}]' ;;
        mixed) printf '%s\n' '[{"name":"ok","status":"running"},{"status":"running"}]' ;;
        multi) printf '%s\n' '[{"name":"multi-ws","workspaces":["/home/me/first","/home/me/second"]}]' ;;
        *) printf '%s\n' '[{"name":"test","status":"stopped","agent":"shell","workspaces":["C:\\Dev\\MyProject"],"id":"11111111-2222-3333-4444-555555555555"},{"name":"demo","status":"running","agent":"my-agent"}]' ;;
        esac
    else
        printf '%s\n' 'NAME      STATE     AGENT     WORKSPACE' 'demo      running   opencode  /workspace/project' 'build     stopped   claude    /workspace/api'
    fi
    ;;
run)
    [ "$SBX_MOCK_RUN_FAIL" = 1 ] && exit 1
    printf '%s\n' sandbox-id
    ;;
stop) [ "$SBX_MOCK_STOP_FAIL" = 1 ] && exit 1; exit 0 ;;
rm|setup) exit 0 ;;
login) printf '%s\n' 'You are signed in [username: fixture]' ;;
*) printf '%s\n' 'sbx mock: unknown command' >&2; exit 1 ;;
esac
"##;

    /// Env vars are process-wide but cargo runs tests in parallel threads,
    /// so every test that mutates (or whose children read) mock-behavior env
    /// vars must hold this lock for its whole body.
    #[cfg(unix)]
    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        CLI_TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// All success-path CLI scenarios, run sequentially so the mock's
    /// behavior env vars never race with each other.
    #[test]
    #[cfg(unix)]
    fn cli_mock_success_paths() {
        let _guard = env_lock();
        let _mock = MockCli::new();
        std::env::set_var("SBX_MOCK_SIGNED_IN", "1");
        std::env::set_var("SBX_MOCK_DAEMON", "running");

        // Unknown subcommand exits 1 in the mock -> Err (the real CLI also
        // has no whoami; sign-in detection uses diagnose instead).
        let err = run_capture(&["definitely-not-a-command"]).unwrap_err();
        assert!(
            err.contains("sbx definitely-not-a-command failed (exit 1)"),
            "got: {err}"
        );
        assert!(
            err.contains("unknown command"),
            "stderr should be included, got: {err}"
        );

        // is_signed_in: diagnose's Authentication check says authenticated.
        assert_eq!(is_signed_in(), Ok(true));

        // login: already-signed-in reports are success, not failure.
        assert!(run_login().is_ok());

        // get_setting default: clipboard.imagePaste prints "false".
        assert_eq!(get_setting("clipboard.imagePaste"), Ok("false".to_string()));

        // get_setting with a mock-controlled value.
        std::env::set_var("SBX_MOCK_PASTE", "true");
        assert_eq!(get_setting("clipboard.imagePaste"), Ok("true".to_string()));
        std::env::remove_var("SBX_MOCK_PASTE");

        // settings set: exit 0.
        assert_eq!(set_clipboard_image_paste(), Ok(()));

        // setup ssh: exit 0, no output.
        assert_eq!(
            run_capture(&["setup", SSH_SETUP_COMMAND]),
            Ok(String::new())
        );

        // login with the mock (whoami exits 0, so "already signed in"
        // reports from the real CLI are treated as success, not failure).
        assert!(run_login().is_ok());

        // list_sandboxes: JSON path (primary). Two mock sandboxes.
        let rows = list_sandboxes().expect("json ls should parse");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "test");
        assert_eq!(rows[0].state, "stopped");
        assert_eq!(rows[0].agent, "shell");
        assert_eq!(rows[0].workspace, "C:\\Dev\\MyProject");
        assert_eq!(rows[0].raw, "11111111-2222-3333-4444-555555555555");
        assert_eq!(rows[1].name, "demo");
        assert_eq!(rows[1].state, "running");
        assert_eq!(rows[1].agent, "my-agent");

        // list_sandboxes: table fallback (mock returns garbage for --json,
        // so Sandy falls back to the human table and parses it).
        std::env::set_var("SBX_MOCK_FAIL_JSON", "1");
        let rows = list_sandboxes().expect("table fallback should parse");
        std::env::remove_var("SBX_MOCK_FAIL_JSON");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "demo");
        assert_eq!(rows[0].state, "running");
        assert_eq!(rows[1].name, "build");

        // list_sandboxes: empty output -> empty vec, no error.
        std::env::set_var("SBX_MOCK_LS_EMPTY", "1");
        let rows = list_sandboxes().expect("empty ls should succeed");
        std::env::remove_var("SBX_MOCK_LS_EMPTY");
        assert!(rows.is_empty());

        // JSON with only "name": optional fields degrade to empty strings.
        std::env::set_var("SBX_MOCK_JSON", "sparse");
        let rows = list_sandboxes().expect("sparse JSON should parse");
        std::env::remove_var("SBX_MOCK_JSON");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "bare");
        assert_eq!(rows[0].state, "");
        assert_eq!(rows[0].agent, "");
        assert_eq!(rows[0].workspace, "");
        assert_eq!(rows[0].raw, "");

        // Unknown JSON fields are ignored by serde.
        std::env::set_var("SBX_MOCK_JSON", "extra");
        let rows = list_sandboxes().expect("unknown fields should be ignored");
        std::env::remove_var("SBX_MOCK_JSON");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "with-unknown");
        assert_eq!(rows[0].state, "running");

        // Unicode in names/workspaces survives the JSON round-trip.
        std::env::set_var("SBX_MOCK_JSON", "unicode");
        let rows = list_sandboxes().expect("unicode JSON should parse");
        std::env::remove_var("SBX_MOCK_JSON");
        assert_eq!(rows[0].name, "ünïcødé-サンドボックス");
        assert_eq!(rows[0].workspace, "/home/ünïcødé/プロジェクト");

        // Multiple workspaces are joined with ", ".
        std::env::set_var("SBX_MOCK_JSON", "multi");
        let rows = list_sandboxes().expect("multi-workspace JSON should parse");
        std::env::remove_var("SBX_MOCK_JSON");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].workspace, "/home/me/first, /home/me/second");

        // login after explicit sign-out flow: not signed in and login
        // output claims success — still Ok (login itself is authoritative).
        std::env::remove_var("SBX_MOCK_SIGNED_IN");
        std::env::remove_var("SBX_MOCK_DAEMON");
        assert!(run_login().is_ok());
    }

    /// JSON-schema edge cases that fall back to the table: a malformed entry
    /// (missing "name") makes the whole --json parse fail, so list_sandboxes
    /// must recover via the human table. Sequential + env-locked.
    #[test]
    #[cfg(unix)]
    fn cli_mock_json_malformed_falls_back_to_table() {
        let _guard = env_lock();
        let _mock = MockCli::new();

        // Per-entry tolerance: the valid entry survives, the invalid one
        // (missing "name") is dropped — no silent fallback to other data.
        std::env::set_var("SBX_MOCK_JSON", "mixed");
        let rows = list_sandboxes().expect("malformed entry should be tolerated");
        std::env::remove_var("SBX_MOCK_JSON");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "ok");
        assert_eq!(rows[0].state, "running");
    }

    /// Failure-path CLI scenarios (SBX_MOCK_FAIL=1), sequential and isolated
    /// from the success test so the env flag never leaks across threads.
    #[test]
    #[cfg(unix)]
    fn cli_mock_failure_paths() {
        let _guard = env_lock();
        let _mock = MockCli::new();

        std::env::set_var("SBX_MOCK_FAIL", "1");

        // run_capture: exit 1 -> Err with command, exit code and stderr.
        let err = run_capture(&["ls"]).unwrap_err();
        assert!(err.contains("sbx ls failed (exit 1)"), "got: {err}");
        assert!(
            err.contains("simulated failure"),
            "stderr body expected, got: {err}"
        );

        // list_sandboxes propagates the error instead of inventing rows:
        // both the --json attempt and the table fallback fail.
        let err = list_sandboxes().unwrap_err();
        assert!(err.contains("failed (exit 1)"), "got: {err}");
        assert!(err.contains("simulated failure"), "got: {err}");

        // set_clipboard_image_paste wraps the failure.
        let err = set_clipboard_image_paste().unwrap_err();
        assert!(
            err.starts_with("could not enable image paste:"),
            "got: {err}"
        );
        assert!(err.contains("simulated failure"), "got: {err}");

        // get_setting propagates the failure.
        assert!(get_setting("clipboard.imagePaste").is_err());

        // An unavailable check is unknown, not an explicit sign-out.
        assert!(is_signed_in().is_err());

        std::env::remove_var("SBX_MOCK_FAIL");
    }

    /// Installation checks against the mock on PATH.
    #[test]
    #[cfg(unix)]
    fn sbx_installed_checks_with_mock() {
        let _guard = env_lock();
        let _mock = MockCli::new();
        assert!(is_sbx_installed());
        assert_eq!(
            ensure_sbx_installed(),
            Ok("sbx is already installed".to_string())
        );
    }

    #[test]
    #[cfg(unix)]
    fn ssh_config_detection_scenarios() {
        let _guard = env_lock();
        let mock = MockCli::new();
        assert_eq!(is_ssh_set_up(), Ok(false));
        std::fs::create_dir(mock.root.join(".ssh")).unwrap();
        // Merely mentioning Docker Sandboxes in a comment is not setup.
        std::fs::write(
            mock.root.join(".ssh/config"),
            "# Docker Sandboxes setup was removed\n",
        )
        .unwrap();
        assert_eq!(is_ssh_set_up(), Ok(false));
        // CLI-managed Includes and OS-specific paths are understood by sbx,
        // without Sandy needing to read HOME or USERPROFILE itself.
        std::fs::write(
            mock.root.join(".ssh/config"),
            "Include generated/sbx.conf\n",
        )
        .unwrap();
        std::env::set_var("SBX_MOCK_SSH", "configured");
        std::env::set_var("SBX_MOCK_DIAGNOSE_EXIT", "1");
        std::env::remove_var("HOME");
        std::env::remove_var("USERPROFILE");
        assert_eq!(is_ssh_set_up(), Ok(true));
        std::env::set_var("SBX_MOCK_SSH", "unavailable");
        assert!(is_ssh_set_up().is_err());
    }

    #[test]
    #[cfg(unix)]
    fn authentication_starts_daemon_before_check_and_parses_failed_report() {
        let _guard = env_lock();
        let mock = MockCli::new();
        std::env::set_var("SBX_MOCK_SIGNED_IN", "1");
        std::env::set_var("SBX_MOCK_DIAGNOSE_EXIT", "1");
        assert_eq!(is_signed_in(), Ok(true));
        assert_eq!(mock.commands(), "daemon status --format json\ndaemon start --detach\ndaemon status --format json\ndiagnose --format json\n");
        // Neither login nor a previous pass is cached. Logout is visible on
        // the very next refresh, even if diagnose still exits nonzero.
        std::env::remove_var("SBX_MOCK_SIGNED_IN");
        assert_eq!(is_signed_in(), Ok(false));
        std::env::set_var("SBX_MOCK_SIGNED_IN", "1");
        assert_eq!(is_signed_in(), Ok(true));
    }

    #[test]
    #[cfg(unix)]
    fn whole_status_cold_start_keeps_successful_checks_from_nonzero_diagnose() {
        let _guard = env_lock();
        let mock = MockCli::new();
        std::env::set_var("SBX_MOCK_SIGNED_IN", "1");
        std::env::set_var("SBX_MOCK_DIAGNOSE_EXIT", "1");
        std::env::set_var(
            "SBX_MOCK_PASTE",
            "{\"key\":\"clipboard.imagePaste\",\"value\":true}",
        );
        std::env::set_var("SBX_MOCK_SSH", "configured");
        let status = crate::read_status().unwrap();
        assert!(status.sbx_installed);
        assert_eq!(status.signed_in, Some(true));
        assert_eq!(status.image_paste_on, Some(true));
        assert_eq!(status.ssh_set_up, Some(true));
        assert!(status.issues.is_empty(), "{:?}", status.issues);
        let commands = mock.commands();
        assert!(commands.starts_with("daemon status --format json\ndaemon start --detach\ndaemon status --format json\ndiagnose --format json\n"));
        assert_eq!(commands.matches("daemon start --detach\n").count(), 1);
        assert!(commands.find("diagnose").unwrap() < commands.find("settings get").unwrap());
    }

    #[test]
    #[cfg(unix)]
    fn whole_status_refresh_distinguishes_sign_out_from_unknown_checks() {
        let _guard = env_lock();
        let _mock = MockCli::new();
        std::env::set_var("SBX_MOCK_SIGNED_IN", "1");
        std::env::set_var("SBX_MOCK_PASTE", "true");
        std::env::set_var("SBX_MOCK_SSH", "configured");
        assert_eq!(crate::read_status().unwrap().signed_in, Some(true));
        std::env::remove_var("SBX_MOCK_SIGNED_IN");
        assert_eq!(crate::read_status().unwrap().signed_in, Some(false));
        std::env::set_var("SBX_MOCK_DIAGNOSE", "invalid");
        let status = crate::read_status().unwrap();
        assert_eq!(status.signed_in, None);
        assert_eq!(status.ssh_set_up, None);
        assert_eq!(status.image_paste_on, Some(true));
        assert_eq!(status.issues.len(), 2);
        let serialized = serde_json::to_value(&status).unwrap();
        assert!(serialized["signed_in"].is_null());
        assert!(serialized["ssh_set_up"].is_null());
        std::env::set_var("SBX_MOCK_FAIL", "1");
        let status = crate::read_status().unwrap();
        assert!(status.sbx_installed);
        assert_eq!(status.signed_in, None);
        assert_eq!(status.image_paste_on, None);
        assert_eq!(status.ssh_set_up, None);
        assert_eq!(status.issues.len(), 3);
    }

    #[test]
    #[cfg(unix)]
    fn install_location_fallback_checks_executable_files_without_a_shell() {
        let _guard = env_lock();
        let mock = MockCli::new();
        let not_executable = mock.root.join("not-executable");
        std::fs::write(&not_executable, "not an executable").unwrap();
        assert!(first_executable(&[mock.root.join("missing"), not_executable.clone()]).is_none());
        assert_eq!(
            first_executable(&[not_executable, mock.root.join("sbx")]),
            Some(mock.root.join("sbx"))
        );
    }

    #[test]
    #[cfg(unix)]
    fn authentication_unknown_is_never_explicit_sign_out() {
        let _guard = env_lock();
        let _mock = MockCli::new();
        for mode in ["invalid", "missing"] {
            std::env::set_var("SBX_MOCK_DIAGNOSE", mode);
            assert!(is_signed_in().is_err(), "mode: {mode}");
        }
        std::env::remove_var("SBX_MOCK_DIAGNOSE");
        std::env::set_var("SBX_MOCK_AUTH_STATUS", "skip");
        std::env::set_var("SBX_MOCK_AUTH_MESSAGE", "daemon unavailable");
        assert!(is_signed_in().is_err());
        std::env::set_var("SBX_MOCK_AUTH_MESSAGE", "not signed in");
        assert!(is_signed_in().is_err(), "a skipped check is not a sign-out");
        std::env::set_var("SBX_MOCK_AUTH_STATUS", "warn");
        assert_eq!(is_signed_in(), Ok(false));
    }

    #[test]
    #[cfg(unix)]
    fn failed_daemon_start_is_unknown_and_does_not_run_authentication() {
        let _guard = env_lock();
        let mock = MockCli::new();
        std::env::set_var("SBX_MOCK_START_FAIL", "1");
        assert!(is_signed_in().is_err());
        assert!(!mock.commands().contains("diagnose"));
        assert!(mock.commands().contains("daemon start --detach"));
    }

    #[test]
    #[cfg(unix)]
    fn image_paste_normalizes_scalar_and_setting_record_outputs() {
        let _guard = env_lock();
        let _mock = MockCli::new();
        for output in [
            "true",
            "\"true\"",
            "{\"key\":\"clipboard.imagePaste\",\"value\":true,\"source\":\"user\"}",
            "{\"value\":\"true\"}",
            "{\"clipboard.imagePaste\":true}",
        ] {
            std::env::set_var("SBX_MOCK_PASTE", output);
            assert_eq!(get_setting("clipboard.imagePaste"), Ok("true".to_string()));
        }
        std::env::set_var(
            "SBX_MOCK_PASTE",
            "{\"key\":\"another.setting\",\"value\":true}",
        );
        assert!(get_setting("clipboard.imagePaste").is_err());
        std::env::set_var("SBX_MOCK_PASTE", "{\"default\":true}");
        assert!(
            get_setting("clipboard.imagePaste").is_err(),
            "never substitute the default for the evaluated value"
        );
    }

    #[test]
    #[cfg(unix)]
    fn lifecycle_reuses_existing_sandbox_and_stops_before_restart() {
        let _guard = env_lock();
        let mock = MockCli::new();
        assert_eq!(start_sandbox("demo"), Ok("sandbox-id".to_string()));
        assert_eq!(mock.commands(), "ls --json\nrun --detach --name demo\n");
        mock.clear_commands();
        assert_eq!(restart_sandbox("demo"), Ok("sandbox-id".to_string()));
        assert_eq!(
            mock.commands(),
            "stop demo\nls --json\nrun --detach --name demo\n"
        );
        mock.clear_commands();
        assert!(start_sandbox("missing")
            .unwrap_err()
            .contains("does not exist"));
        assert_eq!(mock.commands(), "ls --json\n");
        mock.clear_commands();
        assert!(remove_sandbox("demo").is_ok());
        assert_eq!(mock.commands(), "rm --force demo\n");
    }

    #[test]
    #[cfg(unix)]
    fn restart_does_not_start_after_failed_stop_and_reports_failed_resume() {
        let _guard = env_lock();
        let mock = MockCli::new();
        std::env::set_var("SBX_MOCK_STOP_FAIL", "1");
        assert!(restart_sandbox("demo").is_err());
        assert_eq!(mock.commands(), "stop demo\n");
        std::env::remove_var("SBX_MOCK_STOP_FAIL");
        std::env::set_var("SBX_MOCK_RUN_FAIL", "1");
        let error = restart_sandbox("demo").unwrap_err();
        assert!(error.contains("was stopped but could not be started"));
    }

    #[test]
    #[cfg(unix)]
    fn untrusted_names_never_reach_cli_or_terminal() {
        let _guard = env_lock();
        let mock = MockCli::new();
        for name in [
            "",
            "--all",
            "-demo",
            "demo\nother",
            "demo\r",
            "demo\0",
            "demo;touch",
            "demo&calc",
            "demo|cmd",
            "demo%PATH%",
            "demo$(id)",
            "demo`id`",
            "demo\"",
            "demo'",
            "demo/name",
            "demo name",
        ] {
            assert!(start_sandbox(name).is_err(), "{name:?}");
            assert!(stop_sandbox(name).is_err(), "{name:?}");
            assert!(restart_sandbox(name).is_err(), "{name:?}");
            assert!(remove_sandbox(name).is_err(), "{name:?}");
            assert!(open_terminal(name).is_err(), "{name:?}");
            assert!(connection_info(name).is_err(), "{name:?}");
        }
        assert!(mock.commands().is_empty());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn opens_terminal_with_direct_arguments_and_absolute_cli_path() {
        use std::os::unix::fs::PermissionsExt;
        let _guard = env_lock();
        let mock = MockCli::new();
        let terminal = mock.root.join("x-terminal-emulator");
        std::fs::write(
            &terminal,
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$SBX_TEST_ROOT/terminal-args\"\n",
        )
        .unwrap();
        std::fs::set_permissions(&terminal, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(open_terminal("demo"), Ok(()));
        let args = std::fs::read_to_string(mock.root.join("terminal-args")).unwrap();
        assert_eq!(
            args,
            format!(
                "-e\n{}\nexec\n-it\ndemo\n/bin/bash\n",
                mock.root.join("sbx").display()
            )
        );
        assert!(
            mock.commands().is_empty(),
            "the fake terminal must not run a real exec"
        );
        std::fs::remove_file(&terminal).unwrap();
        assert!(open_terminal("demo")
            .unwrap_err()
            .contains("Install a supported terminal"));
    }

    #[test]
    fn connection_info_uses_managed_ssh_alias_and_default_image_user() {
        let info = connection_info("my-project_1").unwrap();
        assert_eq!(info.host, "my-project_1.sbx");
        assert_eq!(info.user, "_default_user_");
        assert_eq!(info.port, None);
        assert_eq!(info.command, "ssh my-project_1.sbx");
    }

    #[test]
    fn platform_terminal_string_escaping_covers_path_metacharacters() {
        assert_eq!(
            shell_quote("/Applications/Agent's tools/sbx"),
            "'/Applications/Agent'\\''s tools/sbx'"
        );
        assert_eq!(
            applescript_quote("exec '/path/with\\slash' \"quoted\""),
            "\"exec '/path/with\\\\slash' \\\"quoted\\\"\""
        );
        assert_eq!(
            powershell_quote("C:\\Tools & SDK's\\sbx.exe"),
            "'C:\\Tools & SDK''s\\sbx.exe'"
        );
    }
}
