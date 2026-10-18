//! Public provider discovery. GitHub uses HTTPS APIs, never a local git/curl install.
//! Hub discovery uses its typed catalog and small OCI manifests, never image layers.
//! Blocking entry points belong on Tauri's blocking worker pool.
//!
//! Verified public interfaces:
//! - GitHub /repos/{owner}/{repo}/tags and /git/trees/{commit}?recursive=1.
//! - Hub /api/search/v3/catalog/search?type=sbx_kit&from=0&size=24;
//!   the older /v2/search/repositories endpoint ignores the type filter.
//! - Hub /v2/repositories/{namespace}/{repo}/tags exposes content_type,
//!   media_type, and version tag names. The search provider has no global version.
//! - V2 OCI artifactType is application/vnd.docker.sandbox.kit.v2, with a
//!   application/vnd.docker.sandbox.kit.v2.spec+yaml config. V3 uses ordinary
//!   image manifests with vnd.docker.sandbox.kit.descriptor annotations.
//! - sbx kit inspect REFERENCE --json currently normalizes v3 workload to
//!   kind=sandbox; raw authored v3 descriptors have kind=workload and no name.

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DESCRIPTOR: &str = "vnd.docker.sandbox.kit.descriptor";
const MAX_BODY: u64 = 12 * 1024 * 1024;
const METADATA_BATCH: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Provider {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub repository: String,
    #[serde(default)]
    pub reference: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KitArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
    pub choices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KitOption {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: String,
    pub schema_version: u8,
    pub kind: String,
    pub provider_id: String,
    pub required_agent: Option<String>,
    #[serde(default)]
    pub arguments: Vec<KitArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Catalog {
    pub kits: Vec<KitOption>,
    pub warnings: Vec<String>,
}

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
                enabled: true,
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

fn validate_provider(provider: &Provider) -> Result<(), String> {
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

fn hub_repository(repository: &str) -> Result<Option<String>, String> {
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

#[derive(Clone)]
struct Api {
    client: Client,
    github: String,
    raw: String,
    hub: String,
    registry: String,
    auth: String,
}

impl Api {
    fn public() -> Result<Self, String> {
        Ok(Self {
            client: Client::builder()
                .user_agent("Sandy/0.1 (public sandbox-kit discovery)")
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|e| e.to_string())?,
            github: "https://api.github.com".into(),
            raw: "https://raw.githubusercontent.com".into(),
            hub: "https://hub.docker.com".into(),
            registry: "https://registry-1.docker.io".into(),
            auth: "https://auth.docker.io/token".into(),
        })
    }

    fn text(&self, url: &str) -> Result<String, String> {
        self.request(url, None).map(|(text, _)| text)
    }

    fn json(&self, url: &str) -> Result<Value, String> {
        serde_json::from_str(&self.text(url)?).map_err(|e| format!("Invalid JSON from {url}: {e}"))
    }

    fn request(&self, url: &str, token: Option<&str>) -> Result<(String, Option<String>), String> {
        // Cache successes only; a rate limit or missing repository must remain visible.
        type HttpCache = Mutex<HashMap<String, (Instant, String, Option<String>)>>;
        static CACHE: OnceLock<HttpCache> = OnceLock::new();
        let cache = CACHE.get_or_init(Default::default);
        if let Some((at, body, digest)) = cache.lock().map_err(|e| e.to_string())?.get(url) {
            if at.elapsed() < Duration::from_secs(300) {
                return Ok((body.clone(), digest.clone()));
            }
        }
        let mut request = self.client.get(url);
        if let Some(token) = token {
            request = request.bearer_auth(token).header(
                "Accept",
                concat!(
                    "application/vnd.oci.image.index.v1+json, ",
                    "application/vnd.oci.image.manifest.v1+json, ",
                    "application/vnd.docker.distribution.manifest.list.v2+json, ",
                    "application/vnd.docker.distribution.manifest.v2+json"
                ),
            );
        }
        let response = request.send().map_err(|e| format!("{url}: {e}"))?;
        let status = response.status();
        if !status.is_success() {
            let explanation = if status.as_u16() == 403 || status.as_u16() == 429 {
                " (public API rate limit or access policy; retry later)"
            } else {
                ""
            };
            return Err(format!("{url}: HTTP {status}{explanation}"));
        }
        let digest = response
            .headers()
            .get("docker-content-digest")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let mut body = String::new();
        response
            .take(MAX_BODY + 1)
            .read_to_string(&mut body)
            .map_err(|e| e.to_string())?;
        if body.len() as u64 > MAX_BODY {
            return Err(format!("Metadata from {url} exceeds 12 MiB"));
        }
        let mut entries = cache.lock().map_err(|e| e.to_string())?;
        if entries.len() > 1024 {
            entries.retain(|_, (at, _, _)| at.elapsed() < Duration::from_secs(300));
        }
        if entries.len() > 1024 {
            entries.clear();
        }
        entries.insert(url.into(), (Instant::now(), body.clone(), digest.clone()));
        Ok((body, digest))
    }

    fn github_tags(&self, repository: &str) -> Result<Vec<Value>, String> {
        let mut tags = Vec::new();
        for page in 1..=100 {
            let value = self.json(&format!(
                "{}/repos/{repository}/tags?per_page=100&page={page}",
                self.github
            ))?;
            let entries = value
                .as_array()
                .ok_or("GitHub tags response is not an array")?;
            tags.extend(entries.iter().cloned());
            if entries.len() < 100 {
                return Ok(tags);
            }
        }
        Err("GitHub repository has over 10,000 tags; choose a reference directly".into())
    }

    fn hub_tags(&self, repository: &str) -> Result<Vec<Value>, String> {
        let mut tags = Vec::new();
        for page in 1..=100 {
            let value = self.json(&format!(
                "{}/v2/repositories/{repository}/tags?page_size=100&page={page}",
                self.hub
            ))?;
            let entries = value["results"]
                .as_array()
                .ok_or("Hub tags response has no results")?;
            tags.extend(entries.iter().filter(|v| is_kit_tag(v)).cloned());
            if value["next"].as_str().is_none_or(str::is_empty) {
                return Ok(tags);
            }
        }
        Err("Docker Hub repository has over 10,000 tags; choose a tag directly".into())
    }
}

fn encoded(value: &str) -> String {
    // Form encoding is also the documented encoding of the Git fragment's ref/dir pairs.
    reqwest::Url::parse_with_params("https://sandy.invalid", &[("v", value)])
        .unwrap()
        .query()
        .unwrap()
        .trim_start_matches("v=")
        .to_string()
}

pub fn provider_versions(provider: Provider) -> Result<Vec<String>, String> {
    versions_with(&Api::public()?, &provider)
}

fn versions_with(api: &Api, provider: &Provider) -> Result<Vec<String>, String> {
    validate_provider(provider)?;
    let values = if provider.kind == "github" {
        api.github_tags(&github_repository(&provider.repository)?)?
    } else if let Some(repository) = hub_repository(&provider.repository)? {
        api.hub_tags(&repository)?
    } else {
        return Ok(Vec::new()); // The search provider has per-repository tags, not a global version.
    };
    Ok(values
        .iter()
        .filter_map(|v| v["name"].as_str().map(String::from))
        .collect())
}

/// `query` narrows Docker Hub's remote search. GitHub repositories and installed
/// built-ins remain available; the frontend can filter their complete metadata.
pub fn discover_kits(providers: Vec<Provider>, query: Option<String>) -> Result<Catalog, String> {
    let mut catalog = discover_with(&Api::public()?, &providers, query.as_deref().unwrap_or(""))?;
    // The CLI is authoritative about the built-ins available in this installation.
    // They are documented v2 shortcuts, not guessed registry repositories.
    match crate::sbx::run_capture(&["run", "--help"]) {
        Ok(help) => {
            if let Some(line) = help
                .lines()
                .find_map(|line| line.trim().strip_prefix("Available agents: "))
            {
                for name in line
                    .split(',')
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                {
                    catalog.kits.push(KitOption {
                        id: format!("builtin:{name}"),
                        name: name.into(),
                        description: format!("Built-in {name} agent (v2)"),
                        source: name.into(),
                        schema_version: 2,
                        kind: "agent".into(),
                        provider_id: "builtin".into(),
                        required_agent: None,
                        arguments: Vec::new(),
                    });
                }
            } else {
                catalog
                    .warnings
                    .push("The installed sbx did not report its available built-in agents".into());
            }
        }
        Err(e) => catalog
            .warnings
            .push(format!("Built-in agents unavailable: {e}")),
    }
    Ok(catalog)
}

fn discover_with(api: &Api, providers: &[Provider], query: &str) -> Result<Catalog, String> {
    let mut catalog = Catalog::default();
    for provider in providers.iter().filter(|p| p.enabled) {
        let result = validate_provider(provider).and_then(|_| match provider.kind.as_str() {
            "github" => discover_github(api, provider),
            "dockerhub" => discover_hub(api, provider, query),
            _ => unreachable!(),
        });
        match result {
            Ok(mut found) => {
                catalog.kits.append(&mut found.kits);
                catalog.warnings.append(&mut found.warnings);
            }
            Err(error) => catalog
                .warnings
                .push(format!("{}: {error}", provider.label)),
        }
    }
    catalog
        .kits
        .sort_by(|a, b| (&a.kind, &a.name, &a.source).cmp(&(&b.kind, &b.name, &b.source)));
    catalog.kits.dedup_by(|a, b| a.id == b.id);
    Ok(catalog)
}

fn discover_github(api: &Api, provider: &Provider) -> Result<Catalog, String> {
    let repository = github_repository(&provider.repository)?;
    let commit = if provider.reference.is_empty() {
        let tags = api.github_tags(&repository)?;
        if let Some(sha) = tags.first().and_then(|tag| tag["commit"]["sha"].as_str()) {
            sha.to_string()
        } else {
            let metadata = api.json(&format!("{}/repos/{repository}", api.github))?;
            let branch = metadata["default_branch"]
                .as_str()
                .ok_or("GitHub repository has no default branch")?;
            github_commit(api, &repository, branch)?
        }
    } else {
        // A manual branch, tag, or commit is independent of tag enumeration.
        // Resolve it directly even when listing tags is unavailable or rate-limited.
        github_commit(api, &repository, &provider.reference)?
    };
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
                .unwrap_or(repository.rsplit('/').next().unwrap());
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

fn encoded_path(value: &str) -> String {
    encoded(value).replace('+', "%20")
}

fn comment_descriptor(text: &str) -> Option<String> {
    let mut in_block = false;
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.trim() == "# kit:" {
            in_block = true;
            continue;
        }
        if in_block {
            if let Some(line) = line.strip_prefix("#   ") {
                lines.push(line);
            } else if line.trim() == "#" {
                lines.push("");
            } else {
                break;
            }
        }
    }
    in_block.then(|| lines.join("\n"))
}

fn schema_version(value: &Value) -> Option<u8> {
    value["schemaVersion"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .or_else(|| {
            value["schemaVersion"]
                .as_u64()
                .and_then(|n| n.try_into().ok())
        })
}

fn option_from_descriptor(
    value: &Value,
    source: &str,
    provider_id: &str,
    fallback: &str,
) -> Result<KitOption, String> {
    let schema_version = schema_version(value)
        .filter(|v| *v == 2 || *v == 3)
        .ok_or("Only kit schemas 2 and 3 are supported")?;
    let kind = match (schema_version, value["kind"].as_str()) {
        (2, Some("sandbox")) => "agent",
        (3, Some("sandbox" | "workload")) => "workload",
        (_, Some("mixin")) => "mixin",
        (_, Some("set")) => return Err("Authored kit sets must be published before selection (their derived workload/mixin kind is not yet known)".into()),
        _ => return Err("Unsupported kit kind".into()),
    };
    let name = if schema_version == 2 {
        value["name"]
            .as_str()
            .ok_or("V2 kit descriptor must declare its name")?
    } else {
        value["name"].as_str().unwrap_or(fallback)
    }
    .to_string();
    if name.is_empty() {
        return Err("Kit has no identity".into());
    }
    let args: BTreeMap<String, Value> = value
        .get("args")
        .and_then(Value::as_object)
        .map(|args| {
            args.iter()
                .map(|(name, arg)| (name.clone(), arg.clone()))
                .collect()
        })
        .unwrap_or_default();
    let arguments = args
        .into_iter()
        .map(|(name, arg)| KitArgument {
            name,
            description: arg["description"].as_str().unwrap_or_default().into(),
            required: arg["required"].as_bool().unwrap_or(false),
            default: arg["default"].as_str().map(String::from),
            choices: arg["enum"]
                .as_array()
                .map(|v| {
                    v.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect();
    Ok(KitOption {
        id: format!("{provider_id}:{source}"),
        name,
        source: source.into(),
        schema_version,
        kind: kind.into(),
        provider_id: provider_id.into(),
        description: value["description"]
            .as_str()
            .or_else(|| value["displayName"].as_str())
            .unwrap_or_default()
            .into(),
        required_agent: value["requires"]["agent"].as_str().map(String::from),
        arguments,
    })
}

fn is_kit_tag(value: &Value) -> bool {
    value["content_type"] == "sbx_kit"
        || value["media_type"]
            .as_str()
            .is_some_and(|m| m.starts_with("application/vnd.docker.sandbox.kit.v"))
}

fn is_kit_repository(value: &Value) -> bool {
    value["content_types"]
        .as_array()
        .is_some_and(|v| v.iter().any(|v| v == "sbx_kit"))
        || value["media_types"].as_array().is_some_and(|v| {
            v.iter().any(|m| {
                m.as_str()
                    .is_some_and(|m| m.starts_with("application/vnd.docker.sandbox.kit.v"))
            })
        })
}

fn discover_hub(api: &Api, provider: &Provider, query: &str) -> Result<Catalog, String> {
    let mut catalog = Catalog::default();
    let repositories = if let Some(repository) = hub_repository(&provider.repository)? {
        vec![repository]
    } else {
        // The Hub catalog API uses from/size (not page_size). Keep browsing bounded and ask for a narrower search.
        let value = api.json(&format!(
            "{}/api/search/v3/catalog/search?type=sbx_kit&query={}&from=0&size=24",
            api.hub,
            encoded(query)
        ))?;
        let entries = value["results"]
            .as_array()
            .ok_or("Hub catalog response has no results")?;
        if value["total"].as_u64().unwrap_or(0) > entries.len() as u64 {
            catalog.warnings.push(format!("{}: showing the first {} matching repositories; search to narrow results or add a repository provider to choose its tags", provider.label, entries.len()));
        }
        entries
            .iter()
            .filter(|v| is_kit_repository(v))
            .filter_map(|v| v["slug"].as_str().or_else(|| v["id"].as_str()))
            .map(String::from)
            .collect()
    };
    for chunk in repositories.chunks(METADATA_BATCH) {
        let jobs = std::thread::scope(|scope| {
            chunk
                .iter()
                .map(|repository| {
                    scope.spawn(move || (repository, selected_hub_kit(api, provider, repository)))
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|job| {
                    job.join()
                        .map_err(|_| "Hub metadata worker failed".to_string())
                })
                .collect::<Vec<_>>()
        });
        for job in jobs {
            let (repository, result) = job?;
            match result {
                Ok(kit) => catalog.kits.push(kit),
                Err(e) => catalog
                    .warnings
                    .push(format!("{} / {repository}: {e}", provider.label)),
            }
        }
    }
    Ok(catalog)
}

fn selected_hub_kit(api: &Api, provider: &Provider, repository: &str) -> Result<KitOption, String> {
    if let Some(digest) = provider.reference.strip_prefix("sha256:") {
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(
                "An OCI sha256 digest must contain exactly 64 hexadecimal characters".into(),
            );
        }
        let source = format!("docker.io/{repository}@{}", provider.reference);
        let descriptor = hub_descriptor(api, repository, &provider.reference)?;
        return option_from_descriptor(
            &descriptor,
            &source,
            &provider.id,
            repository.rsplit('/').next().unwrap(),
        );
    }
    let response = if provider.reference.is_empty() {
        api.json(&format!(
            "{}/v2/repositories/{repository}/tags?page_size=100&page=1",
            api.hub
        ))?
    } else {
        let tag = api.json(&format!(
            "{}/v2/repositories/{repository}/tags/{}",
            api.hub,
            encoded_path(&provider.reference)
        ))?;
        serde_json::json!({"results":[tag]})
    };
    let tag = response["results"]
        .as_array()
        .and_then(|tags| tags.iter().find(|tag| is_kit_tag(tag)))
        .ok_or_else(|| format!("No sandbox-kit tag {} in the returned tags for {repository}; add this repository as a provider to select a version", provider.reference))?;
    let name = tag["name"].as_str().ok_or("Hub tag has no name")?;
    // Retain the chosen tag in the reference for usability. Metadata reads are cached.
    let source = format!("docker.io/{repository}:{name}");
    let descriptor = hub_descriptor(api, repository, name)?;
    option_from_descriptor(
        &descriptor,
        &source,
        &provider.id,
        repository.rsplit('/').next().unwrap(),
    )
}

fn hub_descriptor(api: &Api, repository: &str, reference: &str) -> Result<Value, String> {
    let token = api.json(&format!(
        "{}?service=registry.docker.io&scope=repository:{}:pull",
        api.auth,
        encoded_path(repository)
    ))?["token"]
        .as_str()
        .ok_or("Docker registry returned no public pull token")?
        .to_string();
    let mut reference = reference.to_string();
    for _ in 0..4 {
        let (body, _) = api.request(
            &format!(
                "{}/v2/{repository}/manifests/{}",
                api.registry,
                encoded_path(&reference)
            ),
            Some(&token),
        )?;
        let manifest: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        if let Some(descriptor) = manifest["annotations"][DESCRIPTOR].as_str() {
            return serde_yaml_ng::from_str(descriptor)
                .map_err(|e| format!("Invalid OCI kit descriptor: {e}"));
        }
        if let Some(manifests) = manifest["manifests"].as_array() {
            reference = manifests
                .iter()
                .find(|m| m["platform"]["os"] == "linux")
                .or_else(|| manifests.first())
                .and_then(|m| m["digest"].as_str())
                .ok_or("OCI index has no image manifest")?
                .into();
            continue;
        }
        // V2's artifactType is application/vnd.docker.sandbox.kit.v2, and its
        // config is the YAML spec (not an image config). Some older publishers
        // used a spec layer instead. Neither requires pulling image content.
        if manifest["config"]["mediaType"]
            .as_str()
            .is_some_and(|m| m.ends_with(".spec+yaml"))
        {
            let digest = manifest["config"]["digest"]
                .as_str()
                .ok_or("Kit spec config has no digest")?;
            let (text, _) = api.request(
                &format!("{}/v2/{repository}/blobs/{digest}", api.registry),
                Some(&token),
            )?;
            return serde_yaml_ng::from_str(&text).map_err(|e| e.to_string());
        }
        // Read only the small spec layer, never filesystem/image layers.
        if let Some(layers) = manifest["layers"].as_array() {
            if let Some(layer) = layers.iter().find(|l| {
                l["mediaType"]
                    .as_str()
                    .is_some_and(|m| m.ends_with(".spec+yaml"))
            }) {
                let digest = layer["digest"]
                    .as_str()
                    .ok_or("Kit spec layer has no digest")?;
                let (text, _) = api.request(
                    &format!("{}/v2/{repository}/blobs/{digest}", api.registry),
                    Some(&token),
                )?;
                return serde_yaml_ng::from_str(&text).map_err(|e| e.to_string());
            }
        }
        // Older published formats are understood by the installed CLI. Only bounded selected tags reach this fallback.
        let separator = if reference.starts_with("sha256:") {
            "@"
        } else {
            ":"
        };
        let output = crate::sbx::run_capture(&[
            "kit",
            "inspect",
            &format!("docker.io/{repository}{separator}{reference}"),
            "--json",
        ])?;
        return serde_json::from_str(&output)
            .map_err(|e| format!("Invalid sbx kit inspect JSON: {e}"));
    }
    Err("OCI image index nesting exceeds four levels".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    fn fixture(routes: Vec<(&str, &str)>) -> (Api, std::thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let routes: Vec<_> = routes
            .into_iter()
            .map(|(p, b)| (p.to_string(), b.to_string()))
            .collect();
        let worker = std::thread::spawn(move || {
            for (path, body) in routes {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0; 4096];
                let n = stream.read(&mut request).unwrap();
                assert!(String::from_utf8_lossy(&request[..n]).starts_with(&format!("GET {path} ")));
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                )
                .unwrap();
            }
        });
        let mut api = Api::public().unwrap();
        api.github = base.clone();
        api.raw = base.clone();
        api.hub = base.clone();
        api.registry = base.clone();
        api.auth = format!("{base}/token");
        (api, worker)
    }

    #[test]
    fn github_api_resolves_selected_tag_and_discovers_descriptor() {
        let (api, worker) = fixture(vec![
            ("/repos/team/kits/commits/v1.0", r#"{"sha":"abc123"}"#),
            ("/repos/team/kits/git/trees/abc123?recursive=1", r#"{"tree":[{"type":"blob","path":"tool/spec.yaml"}]}"#),
            ("/team/kits/abc123/tool/spec.yaml", "schemaVersion: '2'\nkind: mixin\nname: tool\nrequires: {agent: claude}\nargs:\n  channel:\n    required: true\n    enum: [stable, beta]\n"),
        ]);
        let provider = Provider {
            id: "custom".into(),
            label: "Test".into(),
            kind: "github".into(),
            repository: "team/kits".into(),
            reference: "v1.0".into(),
            enabled: true,
        };
        let result = discover_with(&api, &[provider], "").unwrap();
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
        assert_eq!(result.kits.len(), 1);
        assert_eq!(
            result.kits[0].source,
            "git+https://github.com/team/kits.git#ref=abc123&dir=tool"
        );
        assert_eq!(result.kits[0].required_agent.as_deref(), Some("claude"));
        assert_eq!(result.kits[0].arguments[0].choices, ["stable", "beta"]);
        worker.join().unwrap();
    }

    #[test]
    fn manual_github_branch_and_commit_do_not_list_tags() {
        for reference in ["feature/next", "abc123"] {
            // The fixture has no tag endpoint. An attempt to list tags fails
            // the expected-request assertion instead of masking the bug.
            let commit_path = format!("/repos/team/kits/commits/{}", encoded_path(reference));
            let (api, worker) = fixture(vec![
                (&commit_path, r#"{"sha":"abc123"}"#),
                (
                    "/repos/team/kits/git/trees/abc123?recursive=1",
                    r#"{"tree":[{"type":"blob","path":"tool/spec.yaml"}]}"#,
                ),
                (
                    "/team/kits/abc123/tool/spec.yaml",
                    "schemaVersion: '2'\nkind: mixin\nname: tool\n",
                ),
            ]);
            let provider = Provider {
                id: "manual".into(),
                label: "Manual reference".into(),
                kind: "github".into(),
                repository: "team/kits".into(),
                reference: reference.into(),
                enabled: true,
            };
            let catalog = discover_with(&api, &[provider], "").unwrap();
            assert!(catalog.warnings.is_empty(), "{:?}", catalog.warnings);
            assert_eq!(catalog.kits.len(), 1);
            assert!(catalog.kits[0].source.contains("#ref=abc123&dir=tool"));
            worker.join().unwrap();
        }
    }

    #[test]
    fn normalized_v3_and_typed_filtering() {
        let value =
            serde_json::json!({"schemaVersion":"3","kind":"sandbox","name":"sbx-kit-claude"});
        assert_eq!(
            option_from_descriptor(
                &value,
                "docker.io/docker/sbx-kit-claude:1",
                "hub",
                "fallback"
            )
            .unwrap()
            .kind,
            "workload"
        );
        let value = serde_json::json!({"schemaVersion":"3","kind":"workload"});
        assert_eq!(
            option_from_descriptor(&value, "source", "id", "alpine")
                .unwrap()
                .name,
            "alpine"
        );
        assert!(!is_kit_repository(
            &serde_json::json!({"content_types":["image"]})
        ));
        assert!(is_kit_repository(
            &serde_json::json!({"content_types":["image","sbx_kit"]})
        ));
        assert!(!is_kit_tag(
            &serde_json::json!({"name":"latest","content_type":"image"})
        ));
        assert!(is_kit_tag(
            &serde_json::json!({"media_type":"application/vnd.docker.sandbox.kit.v2"})
        ));
        assert!(option_from_descriptor(
            &serde_json::json!({"schemaVersion":"1","kind":"mixin"}),
            "s",
            "p",
            "n"
        )
        .is_err());
    }

    #[test]
    fn hub_versions_filter_plain_images_and_keep_tag_selection() {
        let (api, worker) = fixture(vec![(
            "/v2/repositories/team/kit/tags?page_size=100&page=1",
            r#"{"next":null,"results":[{"name":"plain","content_type":"image"},{"name":"v3","content_type":"sbx_kit"},{"name":"v2","media_type":"application/vnd.docker.sandbox.kit.v2"}]}"#,
        )]);
        let provider = Provider {
            id: "hub".into(),
            label: "Test".into(),
            kind: "dockerhub".into(),
            repository: "team/kit".into(),
            reference: String::new(),
            enabled: true,
        };
        assert_eq!(versions_with(&api, &provider).unwrap(), ["v3", "v2"]);
        worker.join().unwrap();
    }

    #[test]
    fn oci_v2_reads_spec_config_without_cli_or_image_layers() {
        let (api, worker) = fixture(vec![
            (
                "/token?service=registry.docker.io&scope=repository:team%2Fkit:pull",
                r#"{"token":"public-fixture-token"}"#,
            ),
            (
                "/v2/team/kit/manifests/1.0",
                r#"{"artifactType":"application/vnd.docker.sandbox.kit.v2","config":{"mediaType":"application/vnd.docker.sandbox.kit.v2.spec+yaml","digest":"sha256:abc"},"layers":[{"mediaType":"application/vnd.oci.empty.v1+json"}]}"#,
            ),
            (
                "/v2/team/kit/blobs/sha256:abc",
                "schemaVersion: '2'\nkind: sandbox\nname: custom\n",
            ),
        ]);
        let descriptor = hub_descriptor(&api, "team/kit", "1.0").unwrap();
        assert_eq!(
            option_from_descriptor(&descriptor, "source", "hub", "kit")
                .unwrap()
                .kind,
            "agent"
        );
        worker.join().unwrap();
    }

    #[test]
    fn immutable_hub_reference_skips_tag_lookup_and_preserves_digest() {
        let digest = format!("sha256:{}", "a".repeat(64));
        let path = format!("/v2/team/kit/manifests/{}", encoded_path(&digest));
        let descriptor = serde_json::json!({"schemaVersion":"3","kind":"workload"});
        let manifest =
            serde_json::json!({"annotations": {DESCRIPTOR: descriptor.to_string()}}).to_string();
        let (api, worker) = fixture(vec![
            (
                "/token?service=registry.docker.io&scope=repository:team%2Fkit:pull",
                r#"{"token":"public-fixture-token"}"#,
            ),
            (&path, &manifest),
        ]);
        let mut provider = Provider {
            id: "hub".into(),
            label: "Test".into(),
            kind: "dockerhub".into(),
            repository: "team/kit".into(),
            reference: digest.clone(),
            enabled: true,
        };
        let kit = selected_hub_kit(&api, &provider, "team/kit").unwrap();
        assert_eq!(kit.source, format!("docker.io/team/kit@{digest}"));
        assert_eq!(kit.kind, "workload");
        worker.join().unwrap();
        provider.reference = "sha256:invalid".into();
        assert!(selected_hub_kit(&api, &provider, "team/kit")
            .unwrap_err()
            .contains("64 hexadecimal"));
    }

    #[test]
    fn oci_v3_index_reads_annotated_descriptor_without_layers() {
        let descriptor = serde_json::json!({"schemaVersion":"3","kind":"workload","args":{"team":{"required":true,"enum":["alpha","beta"]}}});
        let manifest = serde_json::json!({"annotations": {DESCRIPTOR: descriptor.to_string()}, "layers":[{"digest":"never-download"}]}).to_string();
        let (api, worker) = fixture(vec![
            (
                "/token?service=registry.docker.io&scope=repository:team%2Fkit:pull",
                r#"{"token":"public-fixture-token"}"#,
            ),
            (
                "/v2/team/kit/manifests/3.0",
                r#"{"manifests":[{"digest":"sha256:child","platform":{"os":"linux"}}]}"#,
            ),
            ("/v2/team/kit/manifests/sha256%3Achild", &manifest),
        ]);
        let descriptor = hub_descriptor(&api, "team/kit", "3.0").unwrap();
        let kit = option_from_descriptor(&descriptor, "source", "hub", "kit").unwrap();
        assert_eq!(kit.kind, "workload");
        assert!(kit.arguments[0].required);
        assert_eq!(kit.arguments[0].choices, ["alpha", "beta"]);
        worker.join().unwrap();
    }

    #[test]
    fn provider_failure_preserves_other_provider_results() {
        let (api, worker) = fixture(vec![
            (
                "/repos/broken/kits/tags?per_page=100&page=1",
                "invalid-json",
            ),
            (
                "/repos/team/kits/tags?per_page=100&page=1",
                r#"[{"name":"v3","commit":{"sha":"abc123"}}]"#,
            ),
            (
                "/repos/team/kits/git/trees/abc123?recursive=1",
                r#"{"tree":[{"type":"blob","path":"workload/workload.yaml"}]}"#,
            ),
            (
                "/team/kits/abc123/workload/workload.yaml",
                "schemaVersion: '3'\nkind: workload\n",
            ),
        ]);
        let providers = ["broken/kits", "team/kits"].map(|repository| Provider {
            id: repository.into(),
            label: repository.into(),
            kind: "github".into(),
            repository: repository.into(),
            reference: String::new(),
            enabled: true,
        });
        let catalog = discover_with(&api, &providers, "").unwrap();
        assert_eq!(catalog.kits.len(), 1);
        assert_eq!(catalog.warnings.len(), 1);
        assert!(catalog.warnings[0].contains("broken/kits"));
        worker.join().unwrap();
    }

    #[test]
    fn providers_round_trip_and_report_corruption() {
        let root = std::env::temp_dir().join(format!("sandy-catalog-{}", unique_id()));
        let providers = get_providers(&root).unwrap();
        save_providers(&root, providers.clone()).unwrap();
        assert_eq!(get_providers(&root).unwrap(), providers);
        std::fs::write(root.join("providers.json"), "not json").unwrap();
        assert!(get_providers(&root).unwrap_err().contains("providers.json"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
