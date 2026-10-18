//! Persistent cache for immutable GitHub data: repository tag lists and
//! per-commit discovery results. Tag names and the commits they point to
//! never change once created, and a repository tree is fixed by its commit
//! SHA, so entries stay valid until a refresh bypasses and overwrites them.
//! The cache is best effort: read and write failures never fail discovery.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::store::write_json;
use super::types::Catalog;

const CACHE_FILE: &str = "github-cache.json";
/// Discovery snapshots kept per repository; older ones are pruned on write.
const KEEP_SNAPSHOTS: usize = 8;

/// One cached payload. `fetched_at` is informational; correctness comes from
/// immutability, not expiry.
#[derive(Debug, Serialize, Deserialize)]
struct Entry {
    fetched_at: u64,
    value: serde_json::Value,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CacheFile {
    entries: BTreeMap<String, Entry>,
    /// Whether the on-disk file has been read into this mirror. Skipped in
    /// the serialized form; an unreadable file simply stays empty.
    #[serde(skip)]
    loaded: bool,
}

/// In-memory mirror keyed by data directory, so parallel tests and the single
/// app instance never observe another root's entries.
fn mirrors() -> &'static Mutex<BTreeMap<PathBuf, CacheFile>> {
    static MIRRORS: OnceLock<Mutex<BTreeMap<PathBuf, CacheFile>>> = OnceLock::new();
    MIRRORS.get_or_init(Default::default)
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Run `work` against the mirror for `root`, loading the on-disk file on
/// first access. A corrupt or unreadable file is treated as empty; the next
/// store overwrites it.
fn with_file<T>(root: &Path, work: impl FnOnce(&mut CacheFile) -> T) -> Result<T, String> {
    let mut mirrors = mirrors()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let file = mirrors.entry(root.to_path_buf()).or_default();
    if !file.loaded {
        file.loaded = true;
        file.entries = match std::fs::read(root.join(CACHE_FILE)) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => BTreeMap::new(),
        };
    }
    Ok(work(file))
}

fn persist(root: &Path, file: &CacheFile) {
    // Best effort: a full disk or read-only directory must not fail the
    // discovery that produced the entry.
    let _ = write_json(&root.join(CACHE_FILE), file);
}

/// The cached tag array for a repository, or `None` on a miss.
pub(super) fn tags(
    root: &Path,
    repository: &str,
) -> Result<Option<Vec<serde_json::Value>>, String> {
    with_file(root, |file| {
        file.entries
            .get(&format!("tags:{repository}"))
            .and_then(|entry| entry.value.as_array().cloned())
    })
}

pub(super) fn store_tags(root: &Path, repository: &str, tags: &[serde_json::Value]) {
    let value = serde_json::Value::Array(tags.to_vec());
    let _ = with_file(root, |file| {
        file.entries.insert(
            format!("tags:{repository}"),
            Entry {
                fetched_at: now(),
                value,
            },
        );
        persist(root, file);
    });
}

/// The cached discovery result for one repository at one commit.
pub(super) fn discovery(
    root: &Path,
    repository: &str,
    commit: &str,
) -> Result<Option<Catalog>, String> {
    with_file(root, |file| {
        file.entries
            .get(&format!("discover:{repository}@{commit}"))
            .and_then(|entry| serde_json::from_value(entry.value.clone()).ok())
    })
}

/// The newest cached discovery result for a repository, whatever reference
/// produced it. Used when the provider follows the latest tag.
pub(super) fn latest_discovery(root: &Path, repository: &str) -> Result<Option<Catalog>, String> {
    let prefix = format!("discover:{repository}@");
    with_file(root, |file| {
        file.entries
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix))
            .max_by_key(|(_, entry)| entry.fetched_at)
            .and_then(|(_, entry)| serde_json::from_value(entry.value.clone()).ok())
    })
}

/// The commit SHA a cached tag points at. Tag names are immutable, so a hit
/// never needs a network round trip.
pub(super) fn tag_commit(
    root: &Path,
    repository: &str,
    tag: &str,
) -> Result<Option<String>, String> {
    with_file(root, |file| {
        file.entries
            .get(&format!("tags:{repository}"))
            .and_then(|entry| entry.value.as_array())
            .and_then(|tags| {
                tags.iter()
                    .find(|tag_value| tag_value["name"].as_str() == Some(tag))
            })
            .and_then(|tag_value| tag_value["commit"]["sha"].as_str().map(String::from))
    })
}

pub(super) fn store_discovery(root: &Path, repository: &str, commit: &str, catalog: &Catalog) {
    let value = match serde_json::to_value(catalog) {
        Ok(value) => value,
        Err(_) => return,
    };
    let _ = with_file(root, |file| {
        let prefix = format!("discover:{repository}@");
        file.entries.insert(
            format!("{prefix}{commit}"),
            Entry {
                fetched_at: now(),
                value,
            },
        );
        prune_snapshots(file, &prefix);
        persist(root, file);
    });
}

/// Keep only the newest `KEEP_SNAPSHOTS` discovery snapshots of a repository.
fn prune_snapshots(file: &mut CacheFile, prefix: &str) {
    let mut snapshots: Vec<(u64, String)> = file
        .entries
        .iter()
        .filter(|(key, _)| key.starts_with(prefix))
        .map(|(key, entry)| (entry.fetched_at, key.clone()))
        .collect();
    if snapshots.len() <= KEEP_SNAPSHOTS {
        return;
    }
    snapshots.sort_unstable();
    let stale = snapshots.len() - KEEP_SNAPSHOTS;
    for (_, key) in snapshots.into_iter().take(stale) {
        file.entries.remove(&key);
    }
}
