//! The Sandy diagnostic log: a single JSON Lines file in the app data
//! directory recording every CLI invocation, HTTP API call, Tauri command
//! result, and error, for debugging from Settings → Advanced.
//!
//! The file is capped at 10 MiB on disk. Every write measures the file and
//! truncates the oldest half when the cap is reached, so the cap is enforced
//! even across restarts without a separate rotation job.

// Tests may unwrap/expect freely — a panic is exactly the failure mode we want.
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests;

use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Hard on-disk ceiling for the log file.
pub(crate) const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024;

/// When the ceiling is hit, keep roughly this fraction (newest entries live
/// at the end of the file).
const TRIM_KEEP_FRACTION: u64 = 2;

const FILE_NAME: &str = "sandy-debug.log";

#[derive(Debug, Clone, serde::Serialize)]
struct LogRecord<'a> {
    /// RFC 3339 UTC timestamp, e.g. `2026-10-18T12:34:56.789Z`.
    timestamp: String,
    /// One of `cli`, `http`, `command`, `error`, `info`.
    kind: &'a str,
    /// Short summary line (command line, URL, message).
    summary: String,
    /// Optional detail: CLI output, error text, HTTP status.
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    /// `ok` or `error` for operations; absent for informational entries.
    #[serde(skip_serializing_if = "Option::is_none")]
    outcome: Option<&'a str>,
}

/// Milliseconds since the Unix epoch, formatted as RFC 3339 UTC. The `time`
/// crate is already in the dependency tree via Tauri, but its formatting
/// features are not enabled, so the timestamp is composed from the epoch
/// duration directly (no timezone math beyond the UTC offset of zero).
fn rfc3339_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();
    let days = secs / 86_400;
    let seconds_of_day = secs % 86_400;
    let (year, month, day) = civil_from_days(days as i64);
    format!(
        "{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}.{millis:03}Z",
        h = seconds_of_day / 3600,
        m = (seconds_of_day % 3600) / 60,
        s = seconds_of_day % 60,
    )
}

/// Howard Hinnant's days-to-civil algorithm: convert days since the Unix
/// epoch into a (year, month, day) UTC date.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m as u32, d as u32)
}

pub(crate) fn log_path(root: &Path) -> PathBuf {
    root.join(FILE_NAME)
}

struct LogWriter {
    path: PathBuf,
    /// Serializes appends and truncation so concurrent writers cannot
    /// interleave partial lines.
    lock: Mutex<()>,
}

static WRITER: Mutex<Option<LogWriter>> = Mutex::new(None);

/// Point the log at the app data directory. Called once at startup with the
/// resolved path; until then, records are dropped.
pub(crate) fn init(path: PathBuf) {
    if let Ok(mut slot) = WRITER.lock() {
        *slot = Some(LogWriter {
            path,
            lock: Mutex::new(()),
        });
    }
}

/// Reset the writer to uninitialized. Only used by tests.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn reset() {
    if let Ok(mut slot) = WRITER.lock() {
        *slot = None;
    }
}

/// Append a pre-formatted JSON line, enforcing the size cap.
fn append_line(line: &str) {
    // The guard keeps the writer slot locked for the whole append so `init`
    // cannot repoint the log mid-write.
    let guard = match WRITER.lock() {
        Ok(slot) => slot,
        Err(poisoned) => poisoned.into_inner(),
    };
    let Some(writer) = guard.as_ref() else {
        return;
    };
    let file_guard = match writer.lock.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let _guard = file_guard;
    if let Ok(meta) = std::fs::metadata(&writer.path)
        && meta.len() + line.len() as u64 > MAX_LOG_BYTES
    {
        truncate_half(&writer.path);
    }
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&writer.path)
    {
        let _ = file.write_all(line.as_bytes());
        let _ = file.write_all(b"\n");
    }
}

/// How many times `rename_over` retries a transiently denied rename, and how
/// long it waits between attempts.
const RENAME_RETRIES: u32 = 4;
const RENAME_RETRY_DELAY_MS: u64 = 10;

/// Replace `to` with `from` by rename. On Windows the final rename can fail
/// with a sharing violation (surfaced as `PermissionDenied`) while another
/// process — an editor, a sync client, the Settings log viewer — holds the
/// destination open without `FILE_SHARE_DELETE`. Such failures are transient,
/// so retry briefly before giving up.
fn rename_over(from: &Path, to: &Path) -> std::io::Result<()> {
    let mut last_error = None;
    for attempt in 0..=RENAME_RETRIES {
        match std::fs::rename(from, to) {
            Ok(()) => return Ok(()),
            Err(error)
                if attempt < RENAME_RETRIES
                    && error.kind() == std::io::ErrorKind::PermissionDenied =>
            {
                last_error = Some(error);
                std::thread::sleep(std::time::Duration::from_millis(RENAME_RETRY_DELAY_MS));
            }
            Err(error) => return Err(error),
        }
    }
    Err(last_error.unwrap_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::PermissionDenied, "rename was denied")
    }))
}

/// Drop the oldest half of the file. Reads the whole file, seeks to the first
/// newline after the midpoint, and rewrites only the newer half. On any
/// failure the file is left as-is: losing log entries is never worth
/// breaking the app.
fn truncate_half(path: &Path) {
    let keep = match std::fs::read(path) {
        Ok(bytes) => {
            let midpoint = bytes.len() / TRIM_KEEP_FRACTION as usize;
            let start = match bytes[midpoint..].iter().position(|&b| b == b'\n') {
                Some(offset) => midpoint + offset + 1,
                // No newline anywhere after the midpoint (one huge entry):
                // drop everything rather than keep an over-cap file.
                None => bytes.len(),
            };
            bytes[start..].to_vec()
        }
        Err(_) => return,
    };
    let temporary = path.with_extension("log.tmp");
    if OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&temporary)
        .and_then(|mut file| {
            file.write_all(&keep)?;
            file.sync_all()
        })
        .and_then(|_| rename_over(&temporary, path))
        .is_err()
    {
        let _ = std::fs::remove_file(&temporary);
    }
}

/// Sanitize a detail string: control characters (except tab and newline)
/// become spaces, and very long details are cut to a bounded tail so a
/// chatty command cannot dominate the log.
fn sanitize(text: &str, max_bytes: usize) -> String {
    let cleaned: String = text
        .chars()
        .map(|c| {
            if c == '\n' || c == '\t' || (!c.is_control() && c != '\u{7f}') {
                c
            } else {
                ' '
            }
        })
        .collect();
    let truncated = if cleaned.len() > max_bytes {
        let mut start = cleaned.len() - max_bytes;
        while !cleaned.is_char_boundary(start) {
            start += 1;
        }
        format!("…{}", &cleaned[start..])
    } else {
        cleaned
    };
    truncated.trim().to_string()
}

/// Core entry point: record an operation with its outcome.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn record_with_detail(
    kind: &'static str,
    summary: &str,
    detail: &str,
    outcome: Option<&'static str>,
) {
    write_record(kind, summary, Some(detail), outcome);
}

/// Record an operation without detail (command names, status transitions).
pub(crate) fn write_record(
    kind: &'static str,
    summary: &str,
    detail: Option<&str>,
    outcome: Option<&'static str>,
) {
    let record = LogRecord {
        timestamp: rfc3339_now(),
        kind,
        summary: sanitize(summary, 400),
        detail: detail.map(|detail| sanitize(detail, 4000)),
        outcome,
    };
    // A record that cannot serialize (impossible in practice) is dropped.
    if let Ok(line) = serde_json::to_string(&record) {
        append_line(&line);
    }
}

/// Log a free-form informational message (startup, daemon transitions).
pub(crate) fn info(message: &str) {
    write_record("info", message, None, None);
}

/// Log an error message with its full text.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn error(summary: &str, detail: &str) {
    write_record("error", summary, Some(detail), Some("error"));
}

/// Record a CLI invocation and its result.
pub(crate) fn cli_call(args: &[&str], result: &Result<String, String>) {
    let summary = format!("sbx {}", args.join(" "));
    match result {
        Ok(output) => write_record("cli", &summary, Some(output), Some("ok")),
        Err(e) => write_record("cli", &summary, Some(e), Some("error")),
    }
}

/// Record a CLI invocation whose result carries a non-string payload: the
/// payload shape is summarized, errors are logged in full.
pub(crate) fn cli_call_with<E>(args: &[&str], result: &Result<impl Sized, E>)
where
    E: std::fmt::Display,
{
    let summary = format!("sbx {}", args.join(" "));
    match result {
        Ok(_) => write_record("cli", &summary, None, Some("ok")),
        Err(e) => write_record("cli", &summary, Some(&e.to_string()), Some("error")),
    }
}

/// Record an HTTP API request and its result (status line on failure).
pub(crate) fn http_call(method: &str, url: &str, result: &Result<impl Sized, String>) {
    let summary = format!("{method} {url}");
    match result {
        Ok(_) => write_record("http", &summary, None, Some("ok")),
        Err(e) => write_record("http", &summary, Some(e), Some("error")),
    }
}
