//! Validation of names, workspace paths, kit selections, and arguments.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use crate::catalog::KitOption;

pub(super) fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 128
        || !name.as_bytes()[0].is_ascii_alphanumeric()
        || !name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
    {
        return Err("Environment name must start with a letter or number and contain only letters, numbers, dots, underscores, or hyphens (maximum 128 characters)".into());
    }
    Ok(())
}

pub(super) fn workspace_path(path: &str) -> Result<PathBuf, String> {
    if !Path::new(path).is_absolute() {
        return Err("Choose an absolute workspace directory".into());
    }
    let canonical = canonicalize(path).map_err(|e| format!("Workspace {path}: {e}"))?;
    if !canonical.is_dir() {
        return Err(format!("Workspace {path} is not a directory"));
    }
    Ok(canonical)
}

/// `std::fs::canonicalize` returns Windows verbatim paths (`\\?\C:\...`).
/// The `sbx` CLI and ordinary users cannot use those, so simplify them back to
/// regular paths (a no-op on other platforms).
pub(super) fn canonicalize(path: impl AsRef<Path>) -> Result<PathBuf, String> {
    dunce::canonicalize(path).map_err(|e| e.to_string())
}

pub(super) fn path_string(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(String::from)
        .ok_or_else(|| format!("Path is not valid Unicode: {}", path.display()))
}

/// Strip the Windows verbatim prefix (`\\?\C:\...`) that some sources return.
/// The `sbx` CLI and ordinary users cannot use verbatim paths.
pub(super) fn strip_verbatim(path: &Path) -> PathBuf {
    const PREFIX: &str = r"\\?\";
    path.to_string_lossy()
        .strip_prefix(PREFIX)
        .map(PathBuf::from)
        .unwrap_or_else(|| path.to_path_buf())
}

pub(super) fn ensure_outside(path: &Path, workspace: &Path) -> Result<(), String> {
    if path.starts_with(workspace) {
        return Err(format!(
            "Save the environment file outside the workspace: {} is inside {}",
            path.display(),
            workspace.display()
        ));
    }
    Ok(())
}

pub(super) fn builtin(agent: &KitOption) -> bool {
    agent.schema_version == 2
        && agent.kind == "agent"
        && agent.provider_id == "builtin"
        && agent.source == agent.name
        && !agent.name.contains('/')
        && !agent.name.is_empty()
}

pub(super) fn selected<'a>(agent: &'a KitOption, mixins: &'a [KitOption]) -> Vec<&'a KitOption> {
    let mut kits = Vec::new();
    if !builtin(agent) {
        kits.push(agent);
    }
    kits.extend(mixins);
    kits
}

pub(super) fn validate_selection(agent: &KitOption, mixins: &[KitOption]) -> Result<(), String> {
    if !matches!(
        (agent.schema_version, agent.kind.as_str()),
        (2, "agent") | (3, "workload")
    ) {
        return Err("Select a v2 agent or v3 workload as the base environment".into());
    }
    let mut ids = HashSet::new();
    let mut names = HashSet::new();
    for kit in std::iter::once(agent).chain(mixins) {
        if kit.schema_version != agent.schema_version {
            return Err(format!(
                "{} is v{}, but {} is v{}; v2 and v3 kits cannot be combined",
                kit.name, kit.schema_version, agent.name, agent.schema_version
            ));
        }
        if kit.id.trim().is_empty() || kit.source.trim().is_empty() || kit.name.trim().is_empty() {
            return Err("Selected kits must have an ID, name, and source".into());
        }
        if !ids.insert(&kit.id) || !names.insert(&kit.name) {
            return Err(format!(
                "{} is selected more than once; kit names must be unique",
                kit.name
            ));
        }
        if let Some(required) = &kit.required_agent
            && required != &agent.name
        {
            return Err(format!(
                "{} requires agent {required}, but {} was selected",
                kit.name, agent.name
            ));
        }
    }
    if let Some(kit) = mixins.iter().find(|k| k.kind != "mixin") {
        return Err(format!("{} is not a mixin", kit.name));
    }
    Ok(())
}

pub(super) fn validate_arguments(
    kit: &KitOption,
    supplied: &BTreeMap<String, String>,
) -> Result<(), String> {
    for name in supplied.keys() {
        if !kit.arguments.iter().any(|a| &a.name == name) {
            return Err(format!("{} has no argument named {name}", kit.name));
        }
    }
    for arg in &kit.arguments {
        let value = supplied.get(&arg.name).or(arg.default.as_ref());
        if arg.required && value.is_none() {
            return Err(format!("{} requires argument {}", kit.name, arg.name));
        }
        if let Some(value) = value {
            if !arg.choices.is_empty() && !arg.choices.contains(value) {
                return Err(format!(
                    "{}.{} must be one of: {}",
                    kit.name,
                    arg.name,
                    arg.choices.join(", ")
                ));
            }
            if value.contains('\0') {
                return Err(format!("{}.{} contains a NUL byte", kit.name, arg.name));
            }
        }
    }
    Ok(())
}
