//! Opening a native interactive terminal attached to a sandbox.

use std::process::Command;
#[cfg(target_os = "linux")]
use std::process::Stdio;

#[cfg(target_os = "windows")]
use super::cli::HideConsole;
use super::cli::sbx_executable;
use super::lifecycle::validate_sandbox_name;

#[cfg(any(test, target_os = "macos"))]
pub(crate) fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(any(test, target_os = "macos"))]
pub(crate) fn applescript_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(any(test, target_os = "windows"))]
pub(crate) fn powershell_quote(value: &str) -> String {
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
        Err(format!(
            "Could not open a terminal. Install a supported terminal (x-terminal-emulator, GNOME Terminal, Konsole, XFCE Terminal, or xterm).\n{}",
            failures.join("\n")
        ))
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
            // `wt.exe` is a console-subsystem launcher stub: without this
            // flag the release build briefly flashes a console for the stub
            // itself. The real Windows Terminal window is opened by the
            // stub and stays visible.
            command.hide_console();
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
