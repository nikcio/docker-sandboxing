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

mod cache;
mod descriptor;
mod github;
mod http;
mod hub;
mod store;
mod types;

// Tests may unwrap/expect freely — a panic is exactly the failure mode we want.
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests;

pub use store::{get_providers, save_providers};
pub(crate) use store::{github_repository, unique_id, write_json};
#[cfg(test)]
pub use types::KitArgument;
pub use types::{Catalog, KitOption, Provider};

use self::github::discover_github;
use self::http::Api;
use self::hub::discover_hub;
use self::store::{hub_repository, validate_provider};

/// The built-in agents as documented on
/// <https://docs.docker.com/ai/sandboxes/agents/>.
pub(crate) const BUILTIN_AGENTS: &[&str] = &[
    "claude",
    "codex",
    "copilot",
    "cursor",
    "devin",
    "docker-agent",
    "droid",
    "gemini",
    "kiro",
    "opencode",
    "shell",
];

pub fn provider_versions(
    provider: Provider,
    root: &std::path::Path,
    refresh: bool,
) -> Result<Vec<String>, String> {
    let mut api = Api::public()?;
    api.refresh = refresh;
    versions_with(&api, &provider, root)
}

fn versions_with(
    api: &Api,
    provider: &Provider,
    root: &std::path::Path,
) -> Result<Vec<String>, String> {
    validate_provider(provider)?;
    let values = if provider.kind == "github" {
        let repository = github_repository(&provider.repository)?;
        // Tag lists are immutable: serve the persistent cache unless the
        // caller asked for a refresh.
        match cache::tags(root, &repository)? {
            Some(tags) if !api.refresh => tags,
            _ => {
                let tags = api.github_tags(&repository)?;
                cache::store_tags(root, &repository, &tags);
                tags
            }
        }
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
pub fn discover_kits(
    providers: Vec<Provider>,
    query: Option<String>,
    root: &std::path::Path,
    refresh: bool,
) -> Result<Catalog, String> {
    let mut api = Api::public()?;
    api.refresh = refresh;
    let mut catalog = discover_with(&api, &providers, query.as_deref().unwrap_or(""), root)?;
    // The built-ins are documented v2 shortcuts, not guessed registry repositories.
    for name in BUILTIN_AGENTS {
        let description = format!("Built-in {name} agent (v2)");
        catalog.kits.push(KitOption {
            id: format!("builtin:{name}"),
            name: (*name).into(),
            description,
            source: (*name).into(),
            schema_version: 2,
            kind: "agent".into(),
            provider_id: "builtin".into(),
            required_agent: None,
            arguments: Vec::new(),
        });
    }
    Ok(catalog)
}

fn discover_with(
    api: &Api,
    providers: &[Provider],
    query: &str,
    root: &std::path::Path,
) -> Result<Catalog, String> {
    let mut catalog = Catalog::default();
    for provider in providers.iter().filter(|p| p.enabled) {
        let result = validate_provider(provider).and_then(|_| match provider.kind.as_str() {
            "github" => discover_github(api, provider, root),
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

/// Last `/`-separated segment of a repository path (the whole string when it
/// contains no `/`). `rsplit` always yields at least one item, so this only
/// returns `None` for an empty repository string.
fn repository_name(repository: &str) -> Option<&str> {
    repository.rsplit('/').next().filter(|s| !s.is_empty())
}

fn encoded(value: &str) -> String {
    // Form encoding is also the documented encoding of the Git fragment's ref/dir pairs.
    // The URL is a compile-time constant, so parse/query can't actually fail;
    // fall back to the raw value (safe, just unencoded) if it somehow did.
    reqwest::Url::parse_with_params("https://sandy.invalid", &[("v", value)])
        .map(|url| {
            url.query()
                .unwrap_or_default()
                .trim_start_matches("v=")
                .to_string()
        })
        .unwrap_or_else(|_| value.to_string())
}

fn encoded_path(value: &str) -> String {
    encoded(value).replace('+', "%20")
}

const DESCRIPTOR: &str = "vnd.docker.sandbox.kit.descriptor";

const MAX_BODY: u64 = 12 * 1024 * 1024;
const METADATA_BATCH: usize = 4;
