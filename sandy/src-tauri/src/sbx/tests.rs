use std::path::PathBuf;

use crate::read_status;
use crate::sbx::cli::{SSH_SETUP_COMMAND, first_executable, is_sbx_installed, run_capture};
use crate::sbx::install::ensure_sbx_installed;
use crate::sbx::lifecycle::{
    connection_info, remove_sandbox, restart_sandbox, start_sandbox, stop_sandbox,
};
use crate::sbx::listing::{list_sandboxes, parse_ls};
use crate::sbx::status::{
    get_setting, is_signed_in, is_ssh_set_up, run_login, set_clipboard_image_paste,
};
use crate::sbx::terminal::{applescript_quote, open_terminal, powershell_quote, shell_quote};
use crate::sbx::test_env::CLI_TEST_ENV_LOCK;

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
                crate::sbx::test_env::remove_var(key);
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
            crate::sbx::test_env::set_var(key, &root);
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
                Some(value) => crate::sbx::test_env::set_var(key, value),
                None => crate::sbx::test_env::remove_var(key),
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
    crate::sbx::test_env::set_var("SBX_MOCK_SIGNED_IN", "1");
    crate::sbx::test_env::set_var("SBX_MOCK_DAEMON", "running");

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
    crate::sbx::test_env::set_var("SBX_MOCK_PASTE", "true");
    assert_eq!(get_setting("clipboard.imagePaste"), Ok("true".to_string()));
    crate::sbx::test_env::remove_var("SBX_MOCK_PASTE");

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
    crate::sbx::test_env::set_var("SBX_MOCK_FAIL_JSON", "1");
    let rows = list_sandboxes().expect("table fallback should parse");
    crate::sbx::test_env::remove_var("SBX_MOCK_FAIL_JSON");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name, "demo");
    assert_eq!(rows[0].state, "running");
    assert_eq!(rows[1].name, "build");

    // list_sandboxes: empty output -> empty vec, no error.
    crate::sbx::test_env::set_var("SBX_MOCK_LS_EMPTY", "1");
    let rows = list_sandboxes().expect("empty ls should succeed");
    crate::sbx::test_env::remove_var("SBX_MOCK_LS_EMPTY");
    assert!(rows.is_empty());

    // JSON with only "name": optional fields degrade to empty strings.
    crate::sbx::test_env::set_var("SBX_MOCK_JSON", "sparse");
    let rows = list_sandboxes().expect("sparse JSON should parse");
    crate::sbx::test_env::remove_var("SBX_MOCK_JSON");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "bare");
    assert_eq!(rows[0].state, "");
    assert_eq!(rows[0].agent, "");
    assert_eq!(rows[0].workspace, "");
    assert_eq!(rows[0].raw, "");

    // Unknown JSON fields are ignored by serde.
    crate::sbx::test_env::set_var("SBX_MOCK_JSON", "extra");
    let rows = list_sandboxes().expect("unknown fields should be ignored");
    crate::sbx::test_env::remove_var("SBX_MOCK_JSON");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "with-unknown");
    assert_eq!(rows[0].state, "running");

    // Unicode in names/workspaces survives the JSON round-trip.
    crate::sbx::test_env::set_var("SBX_MOCK_JSON", "unicode");
    let rows = list_sandboxes().expect("unicode JSON should parse");
    crate::sbx::test_env::remove_var("SBX_MOCK_JSON");
    assert_eq!(rows[0].name, "ünïcødé-サンドボックス");
    assert_eq!(rows[0].workspace, "/home/ünïcødé/プロジェクト");

    // Multiple workspaces are joined with ", ".
    crate::sbx::test_env::set_var("SBX_MOCK_JSON", "multi");
    let rows = list_sandboxes().expect("multi-workspace JSON should parse");
    crate::sbx::test_env::remove_var("SBX_MOCK_JSON");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].workspace, "/home/me/first, /home/me/second");

    // login after explicit sign-out flow: not signed in and login
    // output claims success — still Ok (login itself is authoritative).
    crate::sbx::test_env::remove_var("SBX_MOCK_SIGNED_IN");
    crate::sbx::test_env::remove_var("SBX_MOCK_DAEMON");
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
    crate::sbx::test_env::set_var("SBX_MOCK_JSON", "mixed");
    let rows = list_sandboxes().expect("malformed entry should be tolerated");
    crate::sbx::test_env::remove_var("SBX_MOCK_JSON");
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

    crate::sbx::test_env::set_var("SBX_MOCK_FAIL", "1");

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

    crate::sbx::test_env::remove_var("SBX_MOCK_FAIL");
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
    crate::sbx::test_env::set_var("SBX_MOCK_SSH", "configured");
    crate::sbx::test_env::set_var("SBX_MOCK_DIAGNOSE_EXIT", "1");
    crate::sbx::test_env::remove_var("HOME");
    crate::sbx::test_env::remove_var("USERPROFILE");
    assert_eq!(is_ssh_set_up(), Ok(true));
    crate::sbx::test_env::set_var("SBX_MOCK_SSH", "unavailable");
    assert!(is_ssh_set_up().is_err());
}

#[test]
#[cfg(unix)]
fn authentication_starts_daemon_before_check_and_parses_failed_report() {
    let _guard = env_lock();
    let mock = MockCli::new();
    crate::sbx::test_env::set_var("SBX_MOCK_SIGNED_IN", "1");
    crate::sbx::test_env::set_var("SBX_MOCK_DIAGNOSE_EXIT", "1");
    assert_eq!(is_signed_in(), Ok(true));
    assert_eq!(
        mock.commands(),
        "daemon status --format json\ndaemon start --detach\ndaemon status --format json\ndiagnose --format json\n"
    );
    // Neither login nor a previous pass is cached. Logout is visible on
    // the very next refresh, even if diagnose still exits nonzero.
    crate::sbx::test_env::remove_var("SBX_MOCK_SIGNED_IN");
    assert_eq!(is_signed_in(), Ok(false));
    crate::sbx::test_env::set_var("SBX_MOCK_SIGNED_IN", "1");
    assert_eq!(is_signed_in(), Ok(true));
}

#[test]
#[cfg(unix)]
fn whole_status_cold_start_keeps_successful_checks_from_nonzero_diagnose() {
    let _guard = env_lock();
    let mock = MockCli::new();
    crate::sbx::test_env::set_var("SBX_MOCK_SIGNED_IN", "1");
    crate::sbx::test_env::set_var("SBX_MOCK_DIAGNOSE_EXIT", "1");
    crate::sbx::test_env::set_var(
        "SBX_MOCK_PASTE",
        "{\"key\":\"clipboard.imagePaste\",\"value\":true}",
    );
    crate::sbx::test_env::set_var("SBX_MOCK_SSH", "configured");
    let status = read_status().unwrap();
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
    crate::sbx::test_env::set_var("SBX_MOCK_SIGNED_IN", "1");
    crate::sbx::test_env::set_var("SBX_MOCK_PASTE", "true");
    crate::sbx::test_env::set_var("SBX_MOCK_SSH", "configured");
    assert_eq!(read_status().unwrap().signed_in, Some(true));
    crate::sbx::test_env::remove_var("SBX_MOCK_SIGNED_IN");
    assert_eq!(read_status().unwrap().signed_in, Some(false));
    crate::sbx::test_env::set_var("SBX_MOCK_DIAGNOSE", "invalid");
    let status = read_status().unwrap();
    assert_eq!(status.signed_in, None);
    assert_eq!(status.ssh_set_up, None);
    assert_eq!(status.image_paste_on, Some(true));
    assert_eq!(status.issues.len(), 2);
    let serialized = serde_json::to_value(&status).unwrap();
    assert!(serialized["signed_in"].is_null());
    assert!(serialized["ssh_set_up"].is_null());
    crate::sbx::test_env::set_var("SBX_MOCK_FAIL", "1");
    let status = read_status().unwrap();
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
        crate::sbx::test_env::set_var("SBX_MOCK_DIAGNOSE", mode);
        assert!(is_signed_in().is_err(), "mode: {mode}");
    }
    crate::sbx::test_env::remove_var("SBX_MOCK_DIAGNOSE");
    crate::sbx::test_env::set_var("SBX_MOCK_AUTH_STATUS", "skip");
    crate::sbx::test_env::set_var("SBX_MOCK_AUTH_MESSAGE", "daemon unavailable");
    assert!(is_signed_in().is_err());
    crate::sbx::test_env::set_var("SBX_MOCK_AUTH_MESSAGE", "not signed in");
    assert!(is_signed_in().is_err(), "a skipped check is not a sign-out");
    crate::sbx::test_env::set_var("SBX_MOCK_AUTH_STATUS", "warn");
    assert_eq!(is_signed_in(), Ok(false));
}

#[test]
#[cfg(unix)]
fn failed_daemon_start_is_unknown_and_does_not_run_authentication() {
    let _guard = env_lock();
    let mock = MockCli::new();
    crate::sbx::test_env::set_var("SBX_MOCK_START_FAIL", "1");
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
        crate::sbx::test_env::set_var("SBX_MOCK_PASTE", output);
        assert_eq!(get_setting("clipboard.imagePaste"), Ok("true".to_string()));
    }
    crate::sbx::test_env::set_var(
        "SBX_MOCK_PASTE",
        "{\"key\":\"another.setting\",\"value\":true}",
    );
    assert!(get_setting("clipboard.imagePaste").is_err());
    crate::sbx::test_env::set_var("SBX_MOCK_PASTE", "{\"default\":true}");
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
    assert!(
        start_sandbox("missing")
            .unwrap_err()
            .contains("does not exist")
    );
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
    crate::sbx::test_env::set_var("SBX_MOCK_STOP_FAIL", "1");
    assert!(restart_sandbox("demo").is_err());
    assert_eq!(mock.commands(), "stop demo\n");
    crate::sbx::test_env::remove_var("SBX_MOCK_STOP_FAIL");
    crate::sbx::test_env::set_var("SBX_MOCK_RUN_FAIL", "1");
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
    assert!(
        open_terminal("demo")
            .unwrap_err()
            .contains("Install a supported terminal")
    );
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
