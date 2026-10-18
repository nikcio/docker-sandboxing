//! Health, authentication, daemon, and settings checks via `sbx diagnose`
//! and `sbx settings`.

use super::cli::{run_capture, sbx_command};

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
///
/// The check's structured `status` field is the primary signal (it gates
/// which message phrases may decide, see `check_configured`); the human
/// `message` is matched tolerantly as the secondary signal, so a reworded
/// CLI message cannot make Sandy report a bogus setup issue.
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

/// Resolve one diagnose check to a boolean from its structured `status`
/// (primary signal) and human `message` (secondary signal).
///
/// Neither signal alone is sufficient:
///
/// - `status` alone lies in both directions. A *negative* fact can be
///   reported as `pass` — today's real CLI emits SSH client config
///   `pass` / "not configured" — and positive fact can be `fail` if the
///   check partially fails.
/// - `message` alone is fragile: the CLI may reword it ("OK — Docker
///   account is authenticated" vs "authenticated"), and exact matching
///   would then surface a bogus setup issue in the UI.
///
/// So `status` gates which phrases may decide at all, and the message is
/// matched tolerantly (see [`message_matches_phrase`]). A genuinely
/// unknown message still errors: an unrecognized check must never be
/// interpreted as an explicit sign-out or as "not set up".
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
    let status = check.status.to_ascii_lowercase();
    // Status gates (primary signal), message decides (secondary signal):
    // a negative fact can arrive with any real status ("pass" + "not
    // configured" in today's CLI; "warn" auth is not a sign-out), while a
    // positive fact is only trusted from an outright "pass".
    //
    // Polarity comes from the message itself: an unnegated phrase decides
    // its own list's fact, and a phrase that appears only in negated form
    // ("no longer signed in", "not yet configured") decides the opposite
    // fact. Anything else is unknown and errors — never a guess.
    let gated = matches!(status.as_str(), "pass" | "warn" | "fail");
    if gated
        && (negative
            .iter()
            .any(|p| message_matches_phrase(&check.message, p))
            || positive
                .iter()
                .any(|p| message_negates_phrase(&check.message, p)))
    {
        return Ok(false);
    }
    if status == "pass"
        && (positive
            .iter()
            .any(|p| message_matches_phrase(&check.message, p))
            || negative
                .iter()
                .any(|p| message_negates_phrase(&check.message, p)))
    {
        return Ok(true);
    }
    Err(format!(
        "Could not determine {name}: {} ({})",
        check.message, check.status
    ))
}

/// Tokenize a check message into lowercase word runs. Contractions are
/// expanded first ("isn't" → "is not", ASCII or typographic apostrophe)
/// so the negator scan sees them — the replacement consumes the stem's
/// final "n" ("won't" → "wo not"), which is cosmetically mangled but
/// never part of a phrase, and the inserted "not" is what matters.
/// Clause separators (",", ".", ";", ":", "!", "?", dashes) mark where a
/// clause ends, so the negator look-behind in [`negated`] never reads a
/// negator from a previous clause into the next one. Every other
/// non-alphanumeric character (punctuation, "OK —" decoration) is
/// dropped, so comparison depends only on the words.
fn message_words(text: &str) -> Vec<String> {
    const CLAUSE_END: &str = "\u{0}";
    text.to_lowercase()
        .replace("n't", " not")
        .replace("n’t", " not")
        .split([',', '.', ';', ':', '!', '?', '—', '–'])
        .flat_map(|clause| {
            clause
                .split(|ch: char| !ch.is_alphanumeric())
                .filter(|w| !w.is_empty())
                .map(str::to_owned)
                .chain(std::iter::once(CLAUSE_END.to_owned()))
        })
        .collect()
}

/// Words that flip a fact's polarity ("not configured" is the opposite of
/// "configured").
fn negator(word: &str) -> bool {
    matches!(word, "not" | "no" | "never" | "without")
}

/// Whether the word run starting at `start` is preceded by a negator.
/// English negation precedes what it negates ("no longer signed in",
/// "not yet configured"), so only a two-word look-behind inside the same
/// clause is checked (clause boundaries are marked in [`message_words`]).
/// Within a clause a stray negator-like word ("no doubt") still flips the
/// fact — the residual risk of any prose heuristic — but a negator can
/// never reach across a "signed in. No errors." style boundary. A
/// trailing "not" starts a new clause rather than negating backwards
/// ("signed in, not signed out"), so it is deliberately ignored.
fn negated(words: &[String], start: usize) -> bool {
    words[..start]
        .iter()
        .rev()
        .take_while(|w| w.as_str() != "\u{0}")
        .take(2)
        .any(|w| negator(w))
}

/// Whether `phrase` appears in `words` as a contiguous word run with the
/// wanted polarity: an occurrence NOT preceded by a negator, or — when
/// `require_negated` — one that is. Every occurrence is considered, so a
/// negated first mention ("not configured, then configured") does not
/// hide a later clean one. Messages and phrases are compared as word
/// runs, never raw substrings, so "unauthenticated" does not match
/// "authenticated". An empty message or phrase matches nothing: an
/// unparseable message stays unknown.
fn run_with_polarity(words: &[String], phrase: &str, require_negated: bool) -> bool {
    // Phrases are single-clause by construction; drop the trailing clause
    // marker [`message_words`] appends so the phrase is a plain word run.
    let mut phrase = message_words(phrase);
    phrase.pop();
    if phrase.is_empty() || phrase.len() > words.len() {
        return false;
    }
    (0..=words.len() - phrase.len())
        .filter(|&start| words[start..start + phrase.len()] == *phrase)
        .any(|start| negated(words, start) == require_negated)
}

/// Tolerant match between a check's human `message` and a known phrase:
/// case- and punctuation-insensitive word-run containment (see
/// [`message_words`] / [`run_with_polarity`]). The `status` gate in
/// [`check_configured`] remains the primary signal; this only widens how
/// the secondary message signal is read.
fn message_matches_phrase(message: &str, phrase: &str) -> bool {
    run_with_polarity(&message_words(message), phrase, false)
}

/// Whether a known phrase appears in the check's message only in negated
/// form ("no longer signed in" for "signed in"). A negated phrase implies
/// the opposite list's fact, which lets rewordings like "not yet
/// configured" resolve instead of erroring.
fn message_negates_phrase(message: &str, phrase: &str) -> bool {
    run_with_polarity(&message_words(message), phrase, true)
}

fn daemon_running() -> Result<bool, String> {
    let out = sbx_command(&["daemon", "status", "--format", "json"])?
        .output()
        .map_err(|e| format!("could not check sbx daemon: {e}"))?;
    if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&out.stdout)
        && let Some(status) = value.get("status").and_then(|v| v.as_str())
    {
        return Ok(out.status.success() && status.eq_ignore_ascii_case("running"));
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
            if let Some(actual_key) = value.get("key").and_then(|v| v.as_str())
                && actual_key != key
            {
                return Err(format!(
                    "sbx returned setting {actual_key} instead of {key}"
                ));
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
