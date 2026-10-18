//! Source authorization: extending the sbx kit.allowedSources policy for
//! enabled GitHub providers.

use std::path::Path;

use crate::catalog::{Provider, get_providers, github_repository};

use super::SOURCE_POLICY;

/// Explicit authorization hook for a provider UI. Only enabled GitHub providers
/// can extend the user allowlist, and existing entries are always preserved.
pub fn allow_source(root: &Path, provider_id: &str) -> Result<String, String> {
    let providers = get_providers(root)?;
    let provider = providers
        .iter()
        .find(|p| p.id == provider_id && p.enabled)
        .ok_or("Enable this provider before authorizing its source")?;
    allow_source_with(provider, &mut |args| crate::sbx::run_capture(args))
}

fn allowed_prefixes(record: &serde_json::Value) -> Result<Vec<String>, String> {
    let value = if let Some(text) = record["value"].as_str() {
        serde_json::from_str(text).map_err(|e| format!("Invalid kit.allowedSources value: {e}"))?
    } else {
        record["value"].clone()
    };
    serde_json::from_value(value)
        .map_err(|e| format!("Cannot read kit.allowedSources as a string array: {e}"))
}

pub(super) fn prefix_matches(prefix: &str, source: &str) -> bool {
    prefix == "*"
        || source == prefix.trim_end_matches('/')
        || source.starts_with(&format!("{}/", prefix.trim_end_matches('/')))
}

pub(super) fn allow_source_with(
    provider: &Provider,
    run: &mut impl FnMut(&[&str]) -> Result<String, String>,
) -> Result<String, String> {
    if provider.kind != "github" {
        return Ok("Docker Hub is governed by the existing sbx source policy".into());
    }
    if !provider.enabled {
        return Err("Enable this GitHub provider before authorizing its source".into());
    }
    let _guard = SOURCE_POLICY.lock().map_err(|e| e.to_string())?;
    let repository = github_repository(&provider.repository)?;
    let prefix = format!("github.com/{repository}");
    // Cover both canonical spellings of this repository, without granting the
    // entire owner's repositories. sbx releases may retain or strip .git.
    let spellings = [prefix.clone(), format!("{prefix}.git")];
    let output = run(&["settings", "get", "kit.allowedSources", "--json"])?;
    let record: serde_json::Value = serde_json::from_str(&output)
        .map_err(|e| format!("Cannot read source policy metadata: {e}"))?;
    let mut prefixes = allowed_prefixes(&record)?;
    if spellings
        .iter()
        .all(|source| prefixes.iter().any(|p| prefix_matches(p, source)))
    {
        return Ok(format!("{prefix} is already allowed"));
    }
    if !matches!(record["source"].as_str(), Some("default" | "override"))
        || record["locked"] == true
        || record["managed"] == true
        || record["read_only"] == true
    {
        return Err(format!(
            "{prefix} is blocked by kit.allowedSources (source: {}). Ask the administrator or update the controlling environment policy; Sandy will not replace it",
            record["source"]
        ));
    }
    for source in &spellings {
        if !prefixes.iter().any(|p| prefix_matches(p, source)) {
            prefixes.push(source.clone());
        }
    }
    let value = serde_json::to_string(&prefixes).map_err(|e| e.to_string())?;
    run(&["settings", "set", "kit.allowedSources", &value]).map_err(|e| {
        format!("Cannot authorize {prefix}; the source policy may be administrator-controlled: {e}")
    })?;
    let output = run(&["settings", "get", "kit.allowedSources", "--json"])?;
    let updated =
        serde_json::from_str(&output).map_err(|e| format!("Cannot verify source policy: {e}"))?;
    let evaluated = allowed_prefixes(&updated)?;
    if !spellings
        .iter()
        .all(|source| evaluated.iter().any(|p| prefix_matches(p, source)))
    {
        return Err(format!(
            "{prefix} remains blocked by the evaluated kit.allowedSources policy; ask the administrator to authorize this repository"
        ));
    }
    Ok(format!(
        "Authorized {prefix}; existing source prefixes preserved"
    ))
}
