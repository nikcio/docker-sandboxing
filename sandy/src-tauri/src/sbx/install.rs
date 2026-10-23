//! Installing the `sbx` CLI per platform (Homebrew, WinGet, apt).

#[cfg(target_os = "macos")]
use std::path::PathBuf;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use super::cli::HideConsole;
#[cfg(target_os = "macos")]
use super::cli::first_executable;
use super::cli::is_sbx_installed;

#[cfg(target_os = "macos")]
fn homebrew_executable() -> Result<PathBuf, String> {
    which::which("brew").ok().or_else(|| {
        first_executable(&["/opt/homebrew/bin/brew".into(), "/usr/local/bin/brew".into()])
    }).ok_or_else(|| {
        "Homebrew is required to install sbx on macOS.\nInstall it from https://brew.sh and try again.".to_string()
    })
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

    // `hide_console` returns `&mut Command` (like std's builder methods), so
    // the chain must end in an owning call: either `.spawn()` below or a
    // separate statement on a binding. Assigning the chain directly would
    // borrow a temporary that dies at the end of the statement (E0716).
    let mut child = std::process::Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .hide_console()
        .spawn()
        .map_err(|e| format!("failed to run {}: {}", program, e))?;

    // Pipes were requested above, so these handles are always present.
    let (Some(mut stdout), Some(mut stderr)) = (child.stdout.take(), child.stderr.take()) else {
        return Err(format!("{} produced no pipe handles", program));
    };
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
