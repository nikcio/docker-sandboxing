// Tests may unwrap/expect freely — a panic is exactly the failure mode we want.
#[allow(unused_imports)]
use super::{
    MAX_LOG_BYTES, civil_from_days, log_path, record_with_detail, rename_over, rfc3339_now,
};
use std::path::PathBuf;

/// The writer is a process-global; every test that points it at a fixture
/// must hold this lock for its whole body.
fn lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "sandy diaglog-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        super::init(log_path(&root));
        Self { root }
    }

    fn log(&self) -> String {
        std::fs::read_to_string(log_path(&self.root)).unwrap_or_default()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // Reset the writer so later tests never append into a deleted fixture.
        super::reset();
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn timestamp_is_rfc3339_utc() {
    let stamp = rfc3339_now();
    // 2026-10-18T12:34:56.789Z — fixed-width, UTC suffix.
    assert_eq!(stamp.len(), 24, "{stamp}");
    assert_eq!(&stamp[4..5], "-");
    assert_eq!(&stamp[7..8], "-");
    assert_eq!(&stamp[10..11], "T");
    assert_eq!(&stamp[13..14], ":");
    assert_eq!(&stamp[16..17], ":");
    assert_eq!(&stamp[19..20], ".");
    assert!(stamp.ends_with('Z'), "{stamp}");
}

#[test]
fn civil_dates_match_known_days() {
    // Day 0 is 1970-01-01.
    assert_eq!(civil_from_days(0), (1970, 1, 1));
    assert_eq!(civil_from_days(1), (1970, 1, 2));
    assert_eq!(civil_from_days(31), (1970, 2, 1));
    // 2024-02-29 (leap day) is 19782 days after the epoch.
    assert_eq!(civil_from_days(19_782), (2024, 2, 29));
    // 2000-02-29, the century leap rule, is 11016 days after the epoch.
    assert_eq!(civil_from_days(11_016), (2000, 2, 29));
}

#[test]
fn records_are_json_lines_with_context() {
    let _guard = lock();
    let fixture = Fixture::new();
    super::record_with_detail("cli", "sbx ls --json", "output line", Some("ok"));
    super::error("Something failed", "the detailed error text");
    super::info("Sandy started");
    let log = fixture.log();
    let lines: Vec<serde_json::Value> = log
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0]["timestamp"].as_str().unwrap().ends_with('Z'));
    assert_eq!(lines[0]["kind"], "cli");
    assert_eq!(lines[0]["summary"], "sbx ls --json");
    assert_eq!(lines[0]["detail"], "output line");
    assert_eq!(lines[0]["outcome"], "ok");
    assert_eq!(lines[1]["kind"], "error");
    assert_eq!(lines[1]["outcome"], "error");
    assert_eq!(lines[1]["detail"], "the detailed error text");
    assert_eq!(lines[2]["kind"], "info");
    // Informational records carry no outcome field.
    assert!(lines[2].get("outcome").is_none());
}

#[test]
fn details_are_sanitized_and_bounded() {
    let _guard = lock();
    let fixture = Fixture::new();
    let noisy = format!("start\u{7}mid\u{0}{}", "x".repeat(10_000));
    super::record_with_detail("cli", "sbx huge", &noisy, Some("error"));
    let log = fixture.log();
    assert!(!log.contains('\u{7}'));
    assert!(!log.contains('\u{0}'));
    let value: serde_json::Value = serde_json::from_str(log.trim()).unwrap();
    let detail = value["detail"].as_str().unwrap();
    assert!(detail.starts_with('…'), "kept the tail: {detail}");
    assert!(detail.len() < 4_200, "detail bounded, got {}", detail.len());
    let whole_file = std::fs::metadata(log_path(&fixture.root)).unwrap().len();
    assert!(whole_file < 8_000, "file stays small, got {whole_file}");
}

#[test]
fn oversize_log_is_truncated_but_keeps_newest_entries() {
    let _guard = lock();
    let fixture = Fixture::new();
    let filler = "y".repeat(2_000);
    // A bit over 5 MiB of records: enough that the cap is crossed well
    // before the final entries.
    for _ in 0..2_600 {
        super::record_with_detail("cli", "sbx fill", &filler, Some("ok"));
    }
    let size = std::fs::metadata(log_path(&fixture.root)).unwrap().len();
    assert!(
        size <= MAX_LOG_BYTES,
        "file must never exceed the cap, got {size}"
    );
    super::record_with_detail("cli", "sbx final", "final entry", Some("ok"));
    let log = fixture.log();
    assert!(
        log.contains("sbx final"),
        "newest entries survive truncation"
    );
    let size = std::fs::metadata(log_path(&fixture.root)).unwrap().len();
    assert!(size <= MAX_LOG_BYTES, "still under the cap, got {size}");
    // Lines stay parseable JSON even after truncation rewrites.
    for line in log.lines() {
        serde_json::from_str::<serde_json::Value>(line).unwrap_or_else(|e| {
            panic!("truncated file keeps whole JSON lines: {e} in {line:.100}")
        });
    }
}

#[test]
fn rename_over_replaces_the_destination() {
    // The fixture re-points the process-global writer; serialize against the
    // other diaglog tests like every other fixture user.
    let _guard = lock();
    let fixture = Fixture::new();
    let from = fixture.root.join("from.tmp");
    let to = log_path(&fixture.root);
    std::fs::write(&from, b"replacement").unwrap();
    std::fs::write(&to, b"original").unwrap();
    super::rename_over(&from, &to).unwrap();
    assert_eq!(std::fs::read_to_string(&to).unwrap(), "replacement");
    assert!(!from.exists(), "the source is gone after the rename");
}
