//! `SandboxInfo` and parsing of `sbx ls --json` output (JSON only).

use serde::Serialize;

use super::cli::run_capture;

#[derive(Debug, Serialize, Clone)]
pub struct SandboxInfo {
    pub name: String,
    pub state: String,
    pub agent: String,
    pub workspace: String,
    pub raw: String,
}

/// List local sandboxes from `sbx ls --json` (stable, machine-readable).
/// JSON-only: a human-table fallback would guess at CLI table layouts that
/// can change between versions, so an unreadable report is an explicit
/// error pointing at an upgrade instead of a silent misclassification.
pub fn list_sandboxes() -> Result<Vec<SandboxInfo>, String> {
    let out = run_capture(&["ls", "--json"])?;
    // Tolerate individual bad entries instead of discarding the whole
    // payload: parse as raw values, keep the ones that deserialize.
    let values: Vec<serde_json::Value> = serde_json::from_str(&out)
        .map_err(|e| format!("could not parse sbx ls --json output (the CLI may be too old — please upgrade sbx): {e}"))?;
    let total = values.len();
    let rows: Vec<SandboxInfo> = values
        .into_iter()
        .filter_map(|v| serde_json::from_value::<SandboxJson>(v).ok())
        .map(|s| SandboxInfo {
            name: s.name,
            state: s.status.unwrap_or_default(),
            agent: s.agent.unwrap_or_default(),
            workspace: s.workspaces.unwrap_or_default().join(", "),
            raw: s.id.unwrap_or_default(),
        })
        .collect();
    // A non-empty payload where nothing deserializes means the CLI renamed
    // its fields, not that there are no sandboxes: reporting an empty list
    // would silently break existence checks downstream (start_sandbox would
    // claim existing sandboxes do not exist), so fail loudly.
    if total > 0 && rows.is_empty() {
        return Err(format!(
            "sbx ls --json returned {total} entries but none matched the expected schema — the CLI may be too old — please upgrade sbx"
        ));
    }
    Ok(rows)
}

/// Schema of `sbx ls --json` entries. Unknown fields are ignored; optional
/// fields degrade to empty so a schema addition never breaks the UI.
#[derive(Debug, serde::Deserialize)]
struct SandboxJson {
    name: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    workspaces: Option<Vec<String>>,
}
