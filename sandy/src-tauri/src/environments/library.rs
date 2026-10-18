//! The persisted template library: saving, listing, and validating
//! environment templates against their selected kits.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::catalog::{KitOption, unique_id, write_json};

use super::LIBRARY;
use super::schema::{Environment, KitEntry, Workspace, escape};
use super::validation::{
    builtin, canonicalize, ensure_outside, path_string, selected, validate_arguments,
    validate_name, validate_selection, workspace_path,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedTemplate {
    pub id: String,
    pub name: String,
    pub path: String,
    pub agent: String,
    pub workspace: String,
    pub kit_version: u8,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveTemplateRequest {
    pub name: String,
    pub workspace: String,
    pub agent: KitOption,
    #[serde(default)]
    pub mixins: Vec<KitOption>,
    /// Keys are KitOption.id, never an ambiguous display name.
    #[serde(default)]
    pub arguments: BTreeMap<String, BTreeMap<String, String>>,
    #[serde(default)]
    pub save_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct TemplateRecord {
    pub(super) id: String,
    pub(super) path: String,
    pub(super) agent: KitOption,
    pub(super) mixins: Vec<KitOption>,
}

pub(super) fn records(root: &Path) -> Result<Vec<TemplateRecord>, String> {
    let path = root.join("templates.json");
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map_err(|e| format!("Cannot read template library {}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!(
            "Cannot read template library {}: {e}",
            path.display()
        )),
    }
}

pub fn save_template(root: &Path, request: SaveTemplateRequest) -> Result<SavedTemplate, String> {
    let _guard = LIBRARY.lock().map_err(|e| e.to_string())?;
    let workspace = workspace_path(&request.workspace)?;
    let environment = build_environment(&request, &workspace)?;
    let content = serde_yaml_ng::to_string(&environment)
        .map_err(|e| format!("Cannot encode environment YAML: {e}"))?;
    let id = unique_id();
    let mut library = records(root)?;
    let path = if let Some(path) = &request.save_path {
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err("Choose an absolute template file path".into());
        }
        let parent = path
            .parent()
            .ok_or("Template path has no parent directory")?;
        // Custom locations must exist; canonicalize the parent before checking symlink escapes.
        let parent = canonicalize(parent)
            .map_err(|e| format!("Template directory {}: {e}", parent.display()))?;
        parent.join(path.file_name().ok_or("Choose a template filename")?)
    } else {
        let directory = root.join("templates").join(&id);
        // Check the existing ancestor before creating the directory in a workspace.
        let mut ancestor = directory.as_path();
        while !ancestor.exists() {
            ancestor = ancestor
                .parent()
                .ok_or("Template directory has no existing parent")?;
        }
        ensure_outside(&canonicalize(ancestor)?, &workspace)?;
        std::fs::create_dir_all(&directory).map_err(|e| format!("{}: {e}", directory.display()))?;
        canonicalize(directory)?.join("sbxenv.yaml")
    };
    ensure_outside(&path, &workspace)?;
    // Even a library-owned path isn't overwritten: each Save creates a new reusable template.
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .map_err(|e| {
            format!(
                "Cannot save {} (existing files are never overwritten): {e}",
                path.display()
            )
        })?;
    if let Err(error) = file
        .write_all(content.as_bytes())
        .and_then(|_| file.sync_all())
    {
        let _ = std::fs::remove_file(&path);
        return Err(format!("Cannot write {}: {error}", path.display()));
    }
    drop(file);
    let record = TemplateRecord {
        id: id.clone(),
        path: path_string(&path)?,
        agent: request.agent,
        mixins: request.mixins,
    };
    library.push(record.clone());
    if let Err(error) = write_json(&root.join("templates.json"), &library) {
        let _ = std::fs::remove_file(&path);
        return Err(format!("Template library could not be saved: {error}"));
    }
    Ok(summary(&record, &environment, content))
}

fn summary(record: &TemplateRecord, environment: &Environment, content: String) -> SavedTemplate {
    SavedTemplate {
        id: record.id.clone(),
        name: environment.name.clone(),
        path: record.path.clone(),
        agent: environment.agent.clone(),
        workspace: environment.workspace.path(),
        kit_version: record.agent.schema_version,
        content,
    }
}

pub(super) fn read_template(
    record: &TemplateRecord,
) -> Result<(SavedTemplate, Environment), String> {
    let content =
        std::fs::read_to_string(&record.path).map_err(|e| format!("{}: {e}", record.path))?;
    let environment: Environment = serde_yaml_ng::from_str(&content)
        .map_err(|e| format!("{}: invalid environment YAML: {e}", record.path))?;
    validate_selection(&record.agent, &record.mixins)?;
    validate_name(&environment.name)?;
    if environment.schema_version != "1" || environment.agent != record.agent.name {
        return Err(format!(
            "{}: environment schema or agent changed; save a new template with the selected kits",
            record.path
        ));
    }
    let expected = selected(&record.agent, &record.mixins);
    if environment
        .kits
        .iter()
        .map(KitEntry::source)
        .ne(expected.iter().map(|k| k.source.as_str()))
    {
        return Err(format!(
            "{}: kit sources changed on disk; save a new template to validate their metadata and compatibility",
            record.path
        ));
    }
    for (entry, kit) in environment.kits.iter().zip(expected) {
        validate_arguments(kit, &entry.arguments())?;
    }
    Ok((summary(record, &environment, content), environment))
}

pub fn list_templates(root: &Path) -> Result<Vec<SavedTemplate>, String> {
    read_library(root).map(|(templates, _)| templates)
}

/// Removes a saved template: its file and the library record. Sandboxes
/// created from the template keep their own copies of the environment.
pub fn delete_template(root: &Path, id: &str) -> Result<(), String> {
    let _guard = LIBRARY.lock().map_err(|e| e.to_string())?;
    let mut library = records(root)?;
    let index = library
        .iter()
        .position(|record| record.id == id)
        .ok_or("Unknown saved template")?;
    let record = library.remove(index);
    // The template file is deleted wherever it lives. A missing file is
    // already gone: deleting the stale record still helps.
    match std::fs::remove_file(&record.path) {
        Ok(()) => {
            // Library-owned templates live in an empty templates/<id>/ folder;
            // custom locations never have one, and remove_dir keeps files.
            let _ = std::fs::remove_dir(root.join("templates").join(&record.id));
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(format!("Cannot delete {}: {e}", record.path)),
    }
    write_json(&root.join("templates.json"), &library)
        .map_err(|error| format!("Template library could not be saved: {error}"))
}

/// Companion to list_templates: report stale files without hiding readable ones.
pub fn template_warnings(root: &Path) -> Result<Vec<String>, String> {
    read_library(root).map(|(_, warnings)| warnings)
}

fn read_library(root: &Path) -> Result<(Vec<SavedTemplate>, Vec<String>), String> {
    let records = {
        let _guard = LIBRARY.lock().map_err(|e| e.to_string())?;
        records(root)?
    };
    let mut templates = Vec::new();
    let mut warnings = Vec::new();
    for record in records {
        match read_template(&record) {
            Ok((template, _)) => templates.push(template),
            Err(error) if error.starts_with(&record.path) => warnings.push(error),
            Err(error) => warnings.push(format!("{}: {error}", record.path)),
        }
    }
    Ok((templates, warnings))
}

fn build_environment(
    request: &SaveTemplateRequest,
    workspace: &Path,
) -> Result<Environment, String> {
    validate_name(&request.name)?;
    validate_selection(&request.agent, &request.mixins)?;
    let all: Vec<_> = std::iter::once(&request.agent)
        .chain(&request.mixins)
        .collect();
    for id in request.arguments.keys() {
        if !all.iter().any(|kit| &kit.id == id) {
            return Err(format!("Arguments reference an unselected kit: {id}"));
        }
    }
    for kit in &all {
        validate_arguments(
            kit,
            request.arguments.get(&kit.id).unwrap_or(&BTreeMap::new()),
        )?;
    }
    if builtin(&request.agent)
        && request
            .arguments
            .get(&request.agent.id)
            .is_some_and(|a| !a.is_empty())
    {
        return Err("Built-in agent shortcuts cannot pin kit arguments in an environment file; select a custom agent kit instead".into());
    }
    let kits = selected(&request.agent, &request.mixins)
        .iter()
        .map(|kit| {
            let supplied = request.arguments.get(&kit.id).cloned().unwrap_or_default();
            if supplied.is_empty() {
                KitEntry::Source(kit.source.clone())
            } else {
                KitEntry::WithArgs {
                    source: kit.source.clone(),
                    args: supplied.into_iter().map(|(k, v)| (k, escape(&v))).collect(),
                }
            }
        })
        .collect();
    // Custom v2 agents register their descriptor name from kits, just like v3 workloads.
    Ok(Environment {
        schema_version: "1".into(),
        name: request.name.clone(),
        agent: request.agent.name.clone(),
        workspace: Workspace::Path(escape(&path_string(workspace)?)),
        kits,
    })
}
