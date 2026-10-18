//! GitHub provider discovery: tags, repository trees, and raw kit
//! descriptors fetched over the public HTTPS APIs.

use std::collections::HashSet;

use serde_json::Value;

use super::cache;
use super::descriptor::{comment_descriptor, option_from_descriptor, schema_version};
use super::http::Api;
use super::store::github_repository;
use super::types::{Catalog, Provider};
use super::{METADATA_BATCH, encoded, encoded_path, repository_name};

pub(super) fn discover_github(
    api: &Api,
    provider: &Provider,
    root: &std::path::Path,
) -> Result<Catalog, String> {
    let repository = github_repository(&provider.repository)?;
    // Reference resolution needs the network only for names the caches have
    // never seen; tags, the commits they name, and whole discovery results
    // are immutable once fetched.
    let reference = provider.reference.as_str();
    if !api.refresh
        && reference.is_empty()
        && let Some(cached) = cache::latest_discovery(root, &repository)?
    {
        return Ok(cached);
    }
    let commit = if reference.is_empty() {
        // Tag lists are immutable, so the persistent cache answers without
        // touching the API; a refresh re-fetches and overwrites the entry.
        let tags = match cache::tags(root, &repository)? {
            Some(tags) if !api.refresh => tags,
            _ => {
                let tags = api.github_tags(&repository)?;
                cache::store_tags(root, &repository, &tags);
                tags
            }
        };
        if let Some(sha) = tags.first().and_then(|tag| tag["commit"]["sha"].as_str()) {
            sha.to_string()
        } else {
            let metadata = api.json(&format!("{}/repos/{repository}", api.github))?;
            let branch = metadata["default_branch"]
                .as_str()
                .ok_or("GitHub repository has no default branch")?;
            github_commit(api, &repository, branch)?
        }
    } else if reference.len() == 40 && reference.bytes().all(|b| b.is_ascii_hexdigit()) {
        // A commit SHA is its own resolution; the tree fetch validates it.
        reference.to_string()
    } else if let Some(sha) = cache::tag_commit(root, &repository, reference)?
        && !api.refresh
    {
        sha
    } else {
        // A manual branch, tag, or commit is independent of tag enumeration.
        // Resolve it directly even when listing tags is unavailable or rate-limited.
        github_commit(api, &repository, reference)?
    };
    // A tree is fixed by its commit SHA, so the whole discovery result for a
    // commit is cacheable without expiry.
    if !api.refresh
        && let Some(cached) = cache::discovery(root, &repository, &commit)?
    {
        return Ok(cached);
    }
    let tree = api.json(&format!(
        "{}/repos/{repository}/git/trees/{commit}?recursive=1",
        api.github
    ))?;
    let entries = tree["tree"]
        .as_array()
        .ok_or("GitHub tree has no entries")?;
    let mut candidates: Vec<_> = entries
        .iter()
        .filter(|entry| entry["type"] == "blob")
        .filter_map(|entry| entry["path"].as_str())
        .filter(|path| {
            path.ends_with(".yaml")
                || path.ends_with(".yml")
                || path.ends_with(".dockerfile")
                || path.rsplit('/').next() == Some("Dockerfile")
        })
        .map(String::from)
        .collect();
    let mut catalog = Catalog::default();
    if tree["truncated"] == true {
        catalog.warnings.push(format!(
            "{}: GitHub truncated the repository tree; some kits may be missing",
            provider.label
        ));
    }
    candidates.sort_by_key(|path| (!path.ends_with("spec.yaml"), path.clone()));
    if candidates.len() > 512 {
        catalog.warnings.push(format!("{}: repository contains {} possible descriptors; only the first 512 are read (spec.yaml first). Use a focused kit repository for complete discovery", provider.label, candidates.len()));
        candidates.truncate(512);
    }
    // Fetch raw descriptors in small batches. Ignore ordinary YAML; malformed kit descriptors remain visible.
    let mut directories = HashSet::new();
    for chunk in candidates.chunks(METADATA_BATCH) {
        let repository = &repository;
        let commit = &commit;
        let results = std::thread::scope(|scope| {
            chunk
                .iter()
                .map(|path| {
                    scope.spawn(move || {
                        let encoded = path
                            .split('/')
                            .map(encoded_path)
                            .collect::<Vec<_>>()
                            .join("/");
                        let url = format!("{}/{repository}/{commit}/{encoded}", api.raw);
                        (path, api.text(&url))
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|job| {
                    job.join()
                        .map_err(|_| "Descriptor worker failed".to_string())
                })
                .collect::<Vec<_>>()
        });
        for result in results {
            let (path, text) = result?;
            let text = match text {
                Ok(text) => text,
                Err(e) => {
                    catalog
                        .warnings
                        .push(format!("{} / {path}: {e}", provider.label));
                    continue;
                }
            };
            let text = comment_descriptor(&text).unwrap_or(text);
            let value: Value = match serde_yaml_ng::from_str(&text) {
                Ok(value) => value,
                Err(e) => {
                    if text.contains("schemaVersion:") && text.contains("kind:") {
                        catalog
                            .warnings
                            .push(format!("{} / {path}: {e}", provider.label));
                    }
                    continue;
                }
            };
            if !matches!(schema_version(&value), Some(2 | 3)) || value["kind"].as_str().is_none() {
                continue;
            }
            // V2's loader only accepts spec.yaml. Other YAML files may be
            // documentation examples rather than loadable directory artifacts.
            if schema_version(&value) == Some(2) && path.rsplit('/').next() != Some("spec.yaml") {
                continue;
            }
            let directory = path.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("");
            let source = format!(
                "git+https://github.com/{repository}.git#ref={}&dir={}",
                encoded(commit),
                encoded(directory)
            );
            let fallback = directory
                .rsplit('/')
                .next()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| repository_name(repository).unwrap_or(repository));
            match option_from_descriptor(&value, &source, &provider.id, fallback) {
                Ok(kit) if directories.insert(directory.to_string()) => catalog.kits.push(kit),
                Ok(kit) => {
                    catalog
                        .kits
                        .retain(|existing| existing.source != kit.source);
                    catalog.warnings.push(format!("{} / {path}: multiple kit descriptors in one directory; sbx requires an unambiguous directory", provider.label));
                }
                Err(e) => catalog
                    .warnings
                    .push(format!("{} / {path}: {e}", provider.label)),
            }
        }
    }
    cache::store_discovery(root, &repository, &commit, &catalog);
    Ok(catalog)
}

fn github_commit(api: &Api, repository: &str, reference: &str) -> Result<String, String> {
    api.json(&format!(
        "{}/repos/{repository}/commits/{}",
        api.github,
        encoded_path(reference)
    ))?["sha"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| "GitHub could not resolve the selected reference to a commit".into())
}
