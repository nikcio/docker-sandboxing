//! Provider persistence: reading, validating, and saving providers.json.

use std::collections::HashSet;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use super::types::Provider;

pub fn get_providers(root: &Path) -> Result<Vec<Provider>, String> {
    let path = root.join("providers.json");
    match std::fs::read(&path) {
        Ok(bytes) => {
            let providers: Vec<Provider> = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
            validate_providers(&providers)?;
            Ok(providers)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(vec![
            Provider {
                id: "nikcio-docker-sandboxing".into(),
                label: "nikcio/docker-sandboxing".into(),
                kind: "github".into(),
                repository: "nikcio/docker-sandboxing".into(),
                reference: String::new(),
                enabled: true,
            },
            Provider {
                id: "dockerhub".into(),
                label: "Docker Hub sandbox kits".into(),
                kind: "dockerhub".into(),
                repository: "https://hub.docker.com/search?type=sbx_kit".into(),
                reference: String::new(),
                enabled: false,
            },
        ]),
        Err(e) => Err(format!("Cannot read {}: {e}", path.display())),
    }
}

pub fn save_providers(root: &Path, providers: Vec<Provider>) -> Result<Vec<Provider>, String> {
    validate_providers(&providers)?;
    write_json(&root.join("providers.json"), &providers)?;
    Ok(providers)
}

fn validate_providers(providers: &[Provider]) -> Result<(), String> {
    let mut ids = HashSet::new();
    for provider in providers {
        if provider.id.trim().is_empty() || !ids.insert(&provider.id) {
            return Err("Provider IDs must be nonempty and unique".into());
        }
        validate_provider(provider)?;
    }
    Ok(())
}

pub(super) fn validate_provider(provider: &Provider) -> Result<(), String> {
    if provider.reference.chars().any(char::is_control) {
        return Err("Provider reference contains control characters".into());
    }
    match provider.kind.as_str() {
        "github" => github_repository(&provider.repository).map(|_| ()),
        "dockerhub" => hub_repository(&provider.repository).map(|_| ()),
        _ => Err(format!("Unsupported provider kind: {}", provider.kind)),
    }
}

pub(crate) fn github_repository(repository: &str) -> Result<String, String> {
    let repository = repository.trim().trim_end_matches('/');
    let repository = repository
        .strip_prefix("https://github.com/")
        .unwrap_or(repository)
        .trim_end_matches(".git");
    let parts: Vec<_> = repository.split('/').collect();
    if parts.len() != 2 || parts.iter().any(|p| !valid_component(p)) {
        return Err("GitHub repository must be owner/repo or https://github.com/owner/repo".into());
    }
    Ok(repository.into())
}

fn valid_component(component: &str) -> bool {
    !component.is_empty()
        && component != "."
        && component != ".."
        && component
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
}

pub(super) fn hub_repository(repository: &str) -> Result<Option<String>, String> {
    let repository = repository.trim().trim_end_matches('/');
    if repository.is_empty()
        || repository == "docker.io"
        || repository == "https://hub.docker.com/search?type=sbx_kit"
        || repository == "https://hub.docker.com/search"
    {
        return Ok(None);
    }
    let repository = repository
        .strip_prefix("https://hub.docker.com/r/")
        .or_else(|| repository.strip_prefix("docker.io/"))
        .unwrap_or(repository);
    let parts: Vec<_> = repository.split('/').collect();
    if parts.len() == 2 && parts.iter().all(|p| valid_component(p)) {
        Ok(Some(repository.into()))
    } else {
        Err("Docker Hub provider must be the kit search URL or namespace/repository".into())
    }
}

pub(crate) fn unique_id() -> String {
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{:x}-{:x}-{:x}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        std::process::id(),
        SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

pub(crate) fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let parent = path.parent().ok_or("Missing data directory")?;
    std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    let temporary = parent.join(format!(".{}.tmp", unique_id()));
    let result = (|| {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| e.to_string())?;
        serde_json::to_writer_pretty(&mut file, value).map_err(|e| e.to_string())?;
        file.write_all(b"\n").map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        std::fs::rename(&temporary, path).map_err(|e| format!("{}: {e}", path.display()))
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}
