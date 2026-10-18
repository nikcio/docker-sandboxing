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
/// <https://docs.docker.com/ai/sandboxes/agents/>. Last-resort fallback for
/// when the installed CLI reports nothing usable.
const DOCUMENTED_AGENTS: &[&str] = &[
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

/// Candidate built-in agent names plus whether they came from the CLI or from
/// the documented fallback list.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BuiltinAgents {
    pub names: Vec<String>,
    pub from_fallback: bool,
}

/// First candidate name from the CLI, the documented fallback list otherwise.
/// Callers push a warning when the fallback is used so a silent CLI failure
/// never masquerades as an authoritative answer.
fn builtin_agents(completion: Option<&str>, help: Option<&str>) -> BuiltinAgents {
    if let Some(output) = completion {
        let names = valid_agent_names(output.lines());
        if !names.is_empty() {
            return BuiltinAgents {
                names,
                from_fallback: false,
            };
        }
    }
    if let Some(help) = help {
        let names = parse_available_agents(help);
        if !names.is_empty() {
            return BuiltinAgents {
                names,
                from_fallback: false,
            };
        }
    }
    BuiltinAgents {
        names: DOCUMENTED_AGENTS
            .iter()
            .map(|name| (*name).into())
            .collect(),
        from_fallback: true,
    }
}

/// Filter raw CLI output down to names that could actually be a built-in
/// agent. The completion protocol mixes agents with directory completions and
/// protocol trailers, so anything that [`validate_name`] would reject (flags,
/// paths, `:4`, prose) is discarded here rather than becoming a phantom
/// `builtin:` kit.
fn valid_agent_names<'a>(candidates: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut names: Vec<String> = candidates
        .map(str::trim)
        .filter(|name| !name.is_empty() && name.len() <= 128)
        .filter(|name| {
            name.as_bytes()[0].is_ascii_alphanumeric()
                && name
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
        })
        .map(String::from)
        .collect();
    names.sort();
    names.dedup();
    names
}

/// Parse "Available agents: ..." out of the human-readable `sbx run --help`
/// text. Case-insensitive and decoration-tolerant (ANSI color codes around the
/// label and values, wrapped values on later lines) so a reworded help still
/// parses; returns an empty list when no agent line is present.
fn parse_available_agents(help: &str) -> Vec<String> {
    // Everything from the first ANSI escape to the last one goes, but the text
    // between separate escape sequences is kept — colors wrap individual
    // values, not the whole line.
    fn strip_ansi(line: &str) -> String {
        let mut plain = String::with_capacity(line.len());
        let mut rest = line;
        while let Some((before, after)) = rest.split_once('\x1b') {
            plain.push_str(before);
            // ESC [ ... letter is CSI; ESC ] ... BEL/ESC is OSC. Keep it simple:
            // drop to the next ASCII letter, which ends both forms.
            rest = after
                .split(|c: char| c.is_ascii_alphabetic())
                .nth(1)
                .map(|_| &after[after.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(0) + 1..])
                .unwrap_or("");
        }
        plain.push_str(rest);
        plain
    }

    let mut names = Vec::new();
    let mut in_agents = false;
    for line in help.lines() {
        let plain = strip_ansi(line);
        let trimmed = plain.trim();
        // Trim decoration that would otherwise end up inside values.
        let trimmed = trimmed.trim_end_matches(['\\', ';']);
        let trimmed = trimmed.trim();
        if let Some((label, value)) = trimmed.split_once(':') {
            if label.trim().eq_ignore_ascii_case("Available agents") {
                in_agents = true;
                names = valid_agent_names(value.split(','));
                continue;
            }
        }
        if in_agents {
            if trimmed.is_empty() {
                break; // The agent list ended; this is the next paragraph.
            }
            names.extend(valid_agent_names(trimmed.split(',')));
        }
    }
    names.sort();
    names.dedup();
    names
}

/// Discover the built-in agents by asking the installed CLI, with the
/// documented list as a last resort. The completion protocol output is only
/// trustworthy from an empty directory (cobra completes relative paths
/// otherwise), so the probe runs in a scratch dir under the OS temp dir.
fn builtin_agents_from_cli() -> (BuiltinAgents, Vec<String>) {
    let mut warnings = Vec::new();
    let scratch = std::env::temp_dir().join(format!("sandy-agents-{}", unique_id()));
    let completion = match std::fs::create_dir(&scratch)
        .map_err(|e| e.to_string())
        .and_then(|_| crate::sbx::run_capture_in(&scratch, &["__complete", "run", ""]))
    {
        Ok(output) => Some(output),
        Err(e) => {
            warnings.push(format!(
                "The installed sbx did not report its available built-in agents: {e}"
            ));
            None
        }
    };
    let _ = std::fs::remove_dir_all(&scratch);

    let help = match crate::sbx::run_capture(&["run", "--help"]) {
        Ok(help) => Some(help),
        Err(e) => {
            warnings.push(format!(
                "The installed sbx did not report its available built-in agents: {e}"
            ));
            None
        }
    };

    let agents = builtin_agents(completion.as_deref(), help.as_deref());
    if agents.from_fallback {
        warnings.push(
            "The installed sbx did not report its available built-in agents; using the documented list".into(),
        );
    }
    (agents, warnings)
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
    let (agents, warnings) = builtin_agents_from_cli();
    catalog.warnings.extend(warnings);
    for name in agents.names {
        let description = format!("Built-in {name} agent (v2)");
        catalog.kits.push(KitOption {
            id: format!("builtin:{name}"),
            name: name.clone(),
            description,
            source: name,
            schema_version: 2,
            kind: "agent".into(),
            provider_id: "builtin".into(),
            required_agent: None,
            arguments: Vec::new(),
        });
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
