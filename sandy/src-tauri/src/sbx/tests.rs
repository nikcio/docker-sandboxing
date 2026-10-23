// Tests may unwrap/expect freely — a panic is exactly the failure mode we want.
use std::path::PathBuf;

use crate::sbx::cli::{SSH_SETUP_COMMAND, first_executable, is_sbx_installed, run_capture};
use crate::sbx::install::ensure_sbx_installed;
use crate::sbx::lifecycle::{
    connection_info, remove_sandbox, restart_sandbox, start_sandbox, stop_sandbox,
};
use crate::sbx::listing::list_sandboxes;
use crate::sbx::status::{
    diagnose_setup_checks, get_setting, run_login, set_clipboard_image_paste,
};

use crate::sbx::terminal::{applescript_quote, open_terminal, powershell_quote, shell_quote};

/// Sign-in verdict of a shared diagnose run.
fn is_signed_in() -> Result<bool, String> {
    diagnose_setup_checks().signed_in
}
use crate::sbx::test_env::CLI_TEST_ENV_LOCK;
use crate::setup::check_setup;

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
            "SBX_MOCK_DAEMON_TEXT",
            "SBX_MOCK_PASTE",
            "SBX_MOCK_FAIL_JSON",
            "SBX_MOCK_LS_EMPTY",
            "SBX_MOCK_JSON",
            "SBX_MOCK_DIAGNOSE",
            "SBX_MOCK_DIAGNOSE_EXIT",
            "SBX_MOCK_AUTH_MESSAGE",
            "SBX_MOCK_AUTH_STATUS",
            "SBX_MOCK_AUTH_HINT",
            "SBX_MOCK_SSH",
            "SBX_MOCK_SSH_STATUS",
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
        // The daemon-confirmation cache and its TTL are process-global; a
        // previous test's confirmed daemon (or pinned TTL) must not change
        // this fixture's daemon probe behavior.
        crate::sbx::status::reset_daemon_cache();
        #[cfg(unix)]
        crate::sbx::status::set_daemon_confirmation_ttl(None);
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
    if [ "$SBX_MOCK_DAEMON_TEXT" = 1 ]; then
        printf '%s\n' 'Status:  ● running'
        exit 0
    fi
    if [ "$SBX_MOCK_DAEMON_TEXT" = 2 ]; then
        printf '%s\n' '{}'
        exit 0
    fi
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
if [ "$SBX_MOCK_SIGNED_IN" = 1 ]; then
    printf '{"checks":[{"name":"Virtualization","status":"fail","message":"no hypervisor present","detail":"/dev/kvm does not exist","hint":""},{"name":"Authentication","status":"%s","message":"%s","detail":"","hint":"%s"},{"name":"SSH client config","status":"%s","message":"%s","detail":"","hint":"Run: sbx setup ssh"}]}\n' "${SBX_MOCK_AUTH_STATUS:-pass}" "${SBX_MOCK_AUTH_MESSAGE:-authenticated}" "${SBX_MOCK_AUTH_HINT:-Run: sbx login}" "${SBX_MOCK_SSH_STATUS:-pass}" "${SBX_MOCK_SSH:-configured}"
else
    printf '{"checks":[{"name":"Virtualization","status":"fail","message":"no hypervisor present","detail":"/dev/kvm does not exist","hint":""},{"name":"Authentication","status":"%s","message":"not signed in","detail":"","hint":"Run: sbx login"},{"name":"SSH client config","status":"%s","message":"%s","detail":"","hint":"Run: sbx setup ssh"}]}\n' "${SBX_MOCK_AUTH_STATUS:-fail}" "${SBX_MOCK_SSH_STATUS:-fail}" "${SBX_MOCK_SSH:-not configured}"
fi
exit "${SBX_MOCK_DIAGNOSE_EXIT:-0}"
;;
settings)
if [ "$2" = get ]; then
    printf '%s\n' "${SBX_MOCK_PASTE:-false}"
fi
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
    renamed) printf '%s\n' '[{"sandbox_name":"x","status":"running"}]' ;;
    multi) printf '%s\n' '[{"name":"multi-ws","workspaces":["/home/me/first","/home/me/second"]}]' ;;
    *) printf '%s\n' '[{"name":"test","status":"stopped","agent":"shell","workspaces":["C:\\Dev\\MyProject"],"id":"11111111-2222-3333-4444-555555555555"},{"name":"demo","status":"running","agent":"my-agent"}]' ;;
    esac
else
    printf '%s\n' 'sbx mock: only --json is supported' >&2
    exit 1
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
    let mock = MockCli::new();
    crate::sbx::test_env::set_var("SBX_MOCK_SIGNED_IN", "1");
    crate::sbx::test_env::set_var("SBX_MOCK_DAEMON", "running");
    // Command-count assertions must not race the real 5-second
    // confirmation window on a loaded machine.
    crate::sbx::status::set_daemon_confirmation_ttl(Some(std::time::Duration::from_secs(3600)));

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

    // Back-to-back checks inside the daemon-confirmation window reuse the
    // first `daemon status` probe: login and get_setting run no new probe.
    let commands = mock.commands();
    assert_eq!(commands.matches("daemon status --format json\n").count(), 1);

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

    // list_sandboxes: unreadable JSON (mock returns garbage for --json)
    // is a hard error, never a fallback to a human table.
    crate::sbx::test_env::set_var("SBX_MOCK_FAIL_JSON", "1");
    let err = list_sandboxes().unwrap_err();
    crate::sbx::test_env::remove_var("SBX_MOCK_FAIL_JSON");
    assert!(
        err.contains("could not parse sbx ls --json output"),
        "got: {err}"
    );
    assert!(err.contains("upgrade sbx"), "got: {err}");
    assert_eq!(mock.commands().matches("ls --json\n").count(), 2);

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

/// Per-entry tolerance for `sbx ls --json`: a malformed entry (missing
/// "name") is dropped while the valid ones survive. Sequential + env-locked.
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

/// A future CLI schema change (e.g. renaming "name") must not look like
/// "no sandboxes": zero-of-N deserialized entries is a loud error instead
/// of a silently empty list. Sequential + env-locked.
#[test]
#[cfg(unix)]
fn cli_mock_json_renamed_field_is_an_error() {
    let _guard = env_lock();
    let mock = MockCli::new();

    // Every entry lacks the required "name" field, so the JSON parse
    // yields zero rows from a non-empty payload -> explicit error.
    crate::sbx::test_env::set_var("SBX_MOCK_JSON", "renamed");
    let err = list_sandboxes().unwrap_err();
    crate::sbx::test_env::remove_var("SBX_MOCK_JSON");
    assert!(
        err.contains("returned 1 entries but none matched the expected schema"),
        "got: {err}"
    );
    assert!(err.contains("upgrade sbx"), "got: {err}");

    // The JSON read stays the only invocation: no retry in any other mode.
    crate::sbx::test_env::set_var("SBX_MOCK_JSON", "renamed");
    mock.clear_commands();
    let _ = list_sandboxes();
    crate::sbx::test_env::remove_var("SBX_MOCK_JSON");
    assert_eq!(mock.commands(), "ls --json\n");
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

    // list_sandboxes propagates the error instead of inventing rows.
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
fn ssh_fail_is_downgraded_to_warning() {
    let _guard = env_lock();
    let _mock = MockCli::new();
    // A `fail` from the CLI's SSH check counts as set up, with a warning:
    // the check has a known false alarm (any <name>.sbx host entry trips
    // it), so a fail must not gate the setup wizard.
    let checks = diagnose_setup_checks();
    assert_eq!(checks.ssh_set_up, Ok(true));
    assert!(checks.ssh_warning.is_some());
    assert!(checks.ssh_warning.unwrap().contains("sbx setup ssh"));
}

#[test]
#[cfg(unix)]
fn authentication_starts_daemon_before_check_and_parses_failed_report() {
    let _guard = env_lock();
    let mock = MockCli::new();
    crate::sbx::test_env::set_var("SBX_MOCK_SIGNED_IN", "1");
    crate::sbx::test_env::set_var("SBX_MOCK_DIAGNOSE_EXIT", "1");
    crate::sbx::status::set_daemon_confirmation_ttl(Some(std::time::Duration::from_secs(3600)));
    assert_eq!(is_signed_in(), Ok(true));
    assert_eq!(
        mock.commands(),
        "daemon status --format json\ndaemon start --detach\ndaemon status --format json\ndiagnose --format json\n"
    );
    // Diagnose is not cached (logout must be visible immediately), but the
    // daemon confirmation is: the follow-up is_signed_in runs diagnose only,
    // with no second daemon probe or start.
    crate::sbx::test_env::remove_var("SBX_MOCK_SIGNED_IN");
    assert_eq!(is_signed_in(), Ok(false));
    crate::sbx::test_env::set_var("SBX_MOCK_SIGNED_IN", "1");
    assert_eq!(is_signed_in(), Ok(true));
    assert_eq!(
        mock.commands()
            .matches("daemon status --format json\n")
            .count(),
        2
    );
    assert_eq!(
        mock.commands().matches("daemon start --detach\n").count(),
        1
    );
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
    crate::sbx::status::set_daemon_confirmation_ttl(Some(std::time::Duration::from_secs(3600)));
    let status = check_setup().unwrap();
    assert!(status.sbx_installed);
    assert_eq!(status.signed_in, Some(true));
    assert_eq!(status.image_paste_on, Some(true));
    assert_eq!(status.ssh_set_up, Some(true));
    assert!(status.warnings.is_empty(), "{:?}", status.warnings);
    assert!(status.issues.is_empty(), "{:?}", status.issues);
    let commands = mock.commands();
    // One status read: a single cold daemon probe/start pair, one shared
    // diagnose run, and one settings read afterwards.
    assert!(commands.starts_with("daemon status --format json\ndaemon start --detach\ndaemon status --format json\ndiagnose --format json\n"));
    assert_eq!(commands.matches("daemon status --format json\n").count(), 2);
    assert_eq!(commands.matches("daemon start --detach\n").count(), 1);
    assert_eq!(commands.matches("diagnose --format json\n").count(), 1);
    assert!(commands.find("diagnose").unwrap() < commands.find("settings get").unwrap());
}

#[test]
#[cfg(unix)]
fn whole_status_refresh_distinguishes_sign_out_from_unknown_checks() {
    let _guard = env_lock();
    let _mock = MockCli::new();
    crate::sbx::test_env::set_var("SBX_MOCK_SIGNED_IN", "1");
    crate::sbx::test_env::set_var("SBX_MOCK_PASTE", "true");
    assert_eq!(check_setup().unwrap().signed_in, Some(true));
    crate::sbx::test_env::remove_var("SBX_MOCK_SIGNED_IN");
    let status = check_setup().unwrap();
    assert_eq!(status.signed_in, Some(false));
    // The SSH check's fail is downgraded to a warning, not an issue.
    assert_eq!(status.ssh_set_up, Some(true));
    assert_eq!(status.warnings.len(), 1);
    crate::sbx::test_env::set_var("SBX_MOCK_DIAGNOSE", "invalid");
    let status = check_setup().unwrap();
    assert_eq!(status.signed_in, None);
    assert_eq!(status.ssh_set_up, None);
    assert_eq!(status.image_paste_on, Some(true));
    assert!(status.warnings.is_empty());
    // One diagnose run serves both checks, so its shared failure is one
    // issue, not two.
    assert_eq!(status.issues.len(), 1);
    let serialized = serde_json::to_value(&status).unwrap();
    assert!(serialized["signed_in"].is_null());
    assert!(serialized["ssh_set_up"].is_null());
    crate::sbx::test_env::set_var("SBX_MOCK_FAIL", "1");
    let status = check_setup().unwrap();
    assert!(status.sbx_installed);
    assert_eq!(status.signed_in, None);
    assert_eq!(status.image_paste_on, None);
    assert_eq!(status.ssh_set_up, None);
    // The shared diagnose error and the settings error: two issues, with
    // the diagnose failure reported once for both checks.
    assert_eq!(status.issues.len(), 2);
}

/// A `warn` verdict on one check and a missing other check produce two
/// distinct unknown-check errors, and both must reach the issues list.
/// Sequential + env-locked.
#[test]
#[cfg(unix)]
fn differing_check_errors_are_both_reported() {
    let _guard = env_lock();
    let _mock = MockCli::new();
    crate::sbx::test_env::set_var("SBX_MOCK_SIGNED_IN", "1");
    crate::sbx::test_env::set_var("SBX_MOCK_AUTH_STATUS", "warn");
    crate::sbx::test_env::set_var("SBX_MOCK_AUTH_MESSAGE", "daemon unavailable");
    crate::sbx::test_env::set_var("SBX_MOCK_DIAGNOSE", "missing");
    let status = check_setup().unwrap();
    assert_eq!(status.signed_in, None);
    assert_eq!(status.ssh_set_up, None);
    // "Could not determine Authentication …" and "did not include the SSH
    // client config check": two distinct problems, two issues.
    assert_eq!(status.issues.len(), 2, "{:?}", status.issues);
}

/// A `fail` verdict from the SSH check is downgraded to "set up with a
/// warning" (the check's known false alarm), while an unreadable diagnose
/// report still leaves the check unknown. Sequential + env-locked.
#[test]
#[cfg(unix)]
fn ssh_fail_is_a_warning_and_unknown_report_is_not() {
    let _guard = env_lock();
    let _mock = MockCli::new();
    crate::sbx::test_env::set_var("SBX_MOCK_SIGNED_IN", "1");
    crate::sbx::test_env::set_var("SBX_MOCK_SSH_STATUS", "fail");
    let checks = diagnose_setup_checks();
    assert_eq!(checks.ssh_set_up, Ok(true));
    let warning = checks.ssh_warning.unwrap();
    assert!(warning.contains("most likely working"), "{warning}");

    // An unknown status (neither pass nor fail) stays an error with no
    // warning, so the check is reported as temporarily unavailable.
    crate::sbx::test_env::set_var("SBX_MOCK_SSH_STATUS", "skip");
    let checks = diagnose_setup_checks();
    assert!(checks.ssh_set_up.is_err());
    assert!(checks.ssh_warning.is_none());
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

/// A failing check's message, detail, and hint all survive into the
/// explanations, and blank fields collapse to None. Sequential + env-locked.
#[test]
#[cfg(unix)]
fn failing_checks_expose_message_detail_and_hint() {
    let _guard = env_lock();
    let _mock = MockCli::new();
    crate::sbx::test_env::set_var("SBX_MOCK_SIGNED_IN", "1");
    let checks = diagnose_setup_checks();

    // Signed in: pass verdict, but the explanation still carries the CLI's
    // fields.
    assert_eq!(checks.signed_in, Ok(true));
    let auth = checks.auth_explanation.expect("auth explanation present");
    assert_eq!(auth.message, "authenticated");
    assert_eq!(auth.detail, None);
    assert_eq!(auth.hint.as_deref(), Some("Run: sbx login"));

    // A fail with a non-empty detail and hint keeps all three.
    crate::sbx::test_env::set_var("SBX_MOCK_AUTH_STATUS", "fail");
    crate::sbx::test_env::set_var("SBX_MOCK_AUTH_MESSAGE", "docker not reachable");
    crate::sbx::test_env::set_var("SBX_MOCK_AUTH_HINT", "Start Docker Desktop");
    let checks = diagnose_setup_checks();
    assert_eq!(checks.signed_in, Ok(false));
    let auth = checks.auth_explanation.expect("auth explanation present");
    assert_eq!(auth.message, "docker not reachable");
    assert_eq!(auth.hint.as_deref(), Some("Start Docker Desktop"));

    // The SSH fail keeps its explanation; the message reflects the mock's
    // signed-in branch default.
    assert_eq!(checks.ssh_set_up, Ok(true), "the SSH fail is downgraded");
    let ssh = checks.ssh_explanation.expect("ssh explanation present");
    assert_eq!(ssh.message, "configured");
    assert_eq!(ssh.hint.as_deref(), Some("Run: sbx setup ssh"));
}

/// A missing or malformed check keeps the explanation unset instead of
/// inventing one. Sequential + env-locked.
#[test]
#[cfg(unix)]
fn missing_checks_have_no_explanation() {
    let _guard = env_lock();
    let _mock = MockCli::new();
    crate::sbx::test_env::set_var("SBX_MOCK_DIAGNOSE", "missing");
    let checks = diagnose_setup_checks();
    assert!(checks.signed_in.is_err());
    assert!(checks.auth_explanation.is_none());
    assert!(checks.ssh_explanation.is_none());
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
    // Only the `status` field decides: a "warn" is neither pass nor fail, so
    // it stays unknown.
    crate::sbx::test_env::set_var("SBX_MOCK_AUTH_STATUS", "warn");
    assert!(is_signed_in().is_err(), "a warn is not a verdict");
    crate::sbx::test_env::set_var("SBX_MOCK_AUTH_MESSAGE", "not signed in");
    crate::sbx::test_env::set_var("SBX_MOCK_AUTH_STATUS", "fail");
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

/// A daemon status that is not usable JSON (e.g. an old CLI that ignores
/// --format) is an explicit upgrade hint, not a trigger to start a daemon
/// we cannot observe. Sequential + env-locked.
#[test]
#[cfg(unix)]
fn unusable_daemon_status_json_is_an_error_and_does_not_start_daemon() {
    let _guard = env_lock();
    let mock = MockCli::new();
    crate::sbx::test_env::set_var("SBX_MOCK_DAEMON_TEXT", "1");
    let err = is_signed_in().unwrap_err();
    crate::sbx::test_env::remove_var("SBX_MOCK_DAEMON_TEXT");
    assert!(
        err.contains("did not return a usable JSON report"),
        "got: {err}"
    );
    assert!(err.contains("upgrade sbx"), "got: {err}");
    assert_eq!(mock.commands(), "daemon status --format json\n");

    // Valid JSON that lacks a "status" string field is equally unusable.
    mock.clear_commands();
    crate::sbx::test_env::set_var("SBX_MOCK_DAEMON_TEXT", "2");
    let err = is_signed_in().unwrap_err();
    crate::sbx::test_env::remove_var("SBX_MOCK_DAEMON_TEXT");
    assert!(
        err.contains("without a \"status\" string field"),
        "got: {err}"
    );
    assert!(err.contains("upgrade sbx"), "got: {err}");
    assert_eq!(mock.commands(), "daemon status --format json\n");
}

#[test]
#[cfg(unix)]
fn image_paste_normalizes_scalar_and_setting_record_outputs() {
    let _guard = env_lock();
    let mock = MockCli::new();
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
    // A mismatching record is a real error, not retried in any other mode:
    // the failed --json read must stay the only settings invocation.
    mock.clear_commands();
    crate::sbx::test_env::set_var(
        "SBX_MOCK_PASTE",
        "{\"key\":\"another.setting\",\"value\":true}",
    );
    assert!(get_setting("clipboard.imagePaste").is_err());
    assert!(
        mock.commands()
            .ends_with("settings get clipboard.imagePaste --json\n")
    );
    assert!(
        !mock
            .commands()
            .contains("settings get clipboard.imagePaste\n")
    );
}

/// get_setting must request the structured record and parse it through the
/// same normalization as plain output. Sequential + env-locked.
#[test]
#[cfg(unix)]
fn get_setting_requests_json_records_from_the_cli() {
    let _guard = env_lock();
    let mock = MockCli::new();
    crate::sbx::test_env::set_var("SBX_MOCK_DAEMON", "running");
    crate::sbx::test_env::set_var(
        "SBX_MOCK_PASTE",
        "{\"key\":\"clipboard.imagePaste\",\"value\":true,\"source\":\"user\"}",
    );
    assert_eq!(get_setting("clipboard.imagePaste"), Ok("true".to_string()));
    assert_eq!(
        mock.commands(),
        "daemon status --format json\nsettings get clipboard.imagePaste --json\n"
    );
}

/// Plain-text output (a decorated or legacy non-JSON response) must never
/// be parsed: get_setting reports an error instead of guessing. Sequential
/// + env-locked.
#[test]
#[cfg(unix)]
fn get_setting_never_parses_plain_text_output() {
    let _guard = env_lock();
    let mock = MockCli::new();
    crate::sbx::test_env::set_var("SBX_MOCK_DAEMON", "running");
    crate::sbx::test_env::set_var("SBX_MOCK_PASTE", "enabled");
    assert!(get_setting("clipboard.imagePaste").is_err());
    // No plain-text retry: the --json read stays the only invocation.
    assert_eq!(
        mock.commands(),
        "daemon status --format json\nsettings get clipboard.imagePaste --json\n"
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
