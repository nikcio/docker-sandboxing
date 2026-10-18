//! Persisted environment library. All paths are supplied by the caller (Tauri's
//! AppHandle.path().app_data_dir()); tests use isolated temporary directories.
//! `launch_template` creates when needed, then starts the sandbox without attaching.
//!
//! The environment schema remains "1" for both kit generations. Custom v2
//! agents and v3 workloads register their name via kits; built-in v2 shortcuts
//! need only agent=name. See Docker's configuration/environment-files reference.
//! `sbx env create --help` verifies --force, --name, and file PATH support;
//! it explicitly directs callers to env run to attach afterward.

use crate::catalog::{
    get_providers, github_repository, unique_id, write_json, KitOption, Provider,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static LIBRARY: Mutex<()> = Mutex::new(());
static SOURCE_POLICY: Mutex<()> = Mutex::new(());

pub fn pick_workspace() -> Result<Option<String>, String> {
    rfd::FileDialog::new()
        .set_title("Choose the sandbox workspace")
        .pick_folder()
        .map(|path| path_string(&path))
        .transpose()
}

pub fn pick_template_path() -> Result<Option<String>, String> {
    rfd::FileDialog::new()
        .set_title("Save the environment template outside your workspace")
        .set_file_name("sbxenv.yaml")
        .add_filter("Environment YAML", &["yaml", "yml"])
        .save_file()
        .map(|path| path_string(&path))
        .transpose()
}

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
struct TemplateRecord {
    id: String,
    path: String,
    agent: KitOption,
    mixins: Vec<KitOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Environment {
    schema_version: String,
    name: String,
    agent: String,
    workspace: Workspace,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    kits: Vec<KitEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum Workspace {
    Path(String),
    Object {
        path: String,
        #[serde(default)]
        clone: bool,
    },
}

impl Workspace {
    fn path(&self) -> String {
        unescape(match self {
            Self::Path(path) | Self::Object { path, .. } => path,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum KitEntry {
    Source(String),
    WithArgs {
        source: String,
        args: BTreeMap<String, String>,
    },
}

impl KitEntry {
    fn source(&self) -> &str {
        match self {
            Self::Source(source) | Self::WithArgs { source, .. } => source,
        }
    }

    fn arguments(&self) -> BTreeMap<String, String> {
        match self {
            Self::Source(_) => BTreeMap::new(),
            Self::WithArgs { args, .. } => {
                args.iter().map(|(k, v)| (k.clone(), unescape(v))).collect()
            }
        }
    }
}

fn escape(value: &str) -> String {
    value.replace("${{", "$${{")
}

fn unescape(value: &str) -> String {
    value.replace("$${{", "${{")
}

fn validate_name(name: &str) -> Result<(), String> {
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

fn workspace_path(path: &str) -> Result<PathBuf, String> {
    if !Path::new(path).is_absolute() {
        return Err("Choose an absolute workspace directory".into());
    }
    let canonical = std::fs::canonicalize(path).map_err(|e| format!("Workspace {path}: {e}"))?;
    if !canonical.is_dir() {
        return Err(format!("Workspace {path} is not a directory"));
    }
    Ok(canonical)
}

fn path_string(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(String::from)
        .ok_or_else(|| format!("Path is not valid Unicode: {}", path.display()))
}

fn ensure_outside(path: &Path, workspace: &Path) -> Result<(), String> {
    if path.starts_with(workspace) {
        return Err(format!(
            "Save the environment file outside the workspace: {} is inside {}",
            path.display(),
            workspace.display()
        ));
    }
    Ok(())
}

fn builtin(agent: &KitOption) -> bool {
    agent.schema_version == 2
        && agent.kind == "agent"
        && agent.provider_id == "builtin"
        && agent.source == agent.name
        && !agent.name.contains('/')
        && !agent.name.is_empty()
}

fn selected<'a>(agent: &'a KitOption, mixins: &'a [KitOption]) -> Vec<&'a KitOption> {
    let mut kits = Vec::new();
    if !builtin(agent) {
        kits.push(agent);
    }
    kits.extend(mixins);
    kits
}

fn validate_selection(agent: &KitOption, mixins: &[KitOption]) -> Result<(), String> {
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
        if let Some(required) = &kit.required_agent {
            if required != &agent.name {
                return Err(format!(
                    "{} requires agent {required}, but {} was selected",
                    kit.name, agent.name
                ));
            }
        }
    }
    if let Some(kit) = mixins.iter().find(|k| k.kind != "mixin") {
        return Err(format!("{} is not a mixin", kit.name));
    }
    Ok(())
}

fn validate_arguments(kit: &KitOption, supplied: &BTreeMap<String, String>) -> Result<(), String> {
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

fn records(root: &Path) -> Result<Vec<TemplateRecord>, String> {
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
        let parent = std::fs::canonicalize(parent)
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
        ensure_outside(
            &std::fs::canonicalize(ancestor).map_err(|e| e.to_string())?,
            &workspace,
        )?;
        std::fs::create_dir_all(&directory).map_err(|e| format!("{}: {e}", directory.display()))?;
        std::fs::canonicalize(directory)
            .map_err(|e| e.to_string())?
            .join("sbxenv.yaml")
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

fn read_template(record: &TemplateRecord) -> Result<(SavedTemplate, Environment), String> {
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
        return Err(format!("{}: kit sources changed on disk; save a new template to validate their metadata and compatibility", record.path));
    }
    for (entry, kit) in environment.kits.iter().zip(expected) {
        validate_arguments(kit, &entry.arguments())?;
    }
    Ok((summary(record, &environment, content), environment))
}

pub fn list_templates(root: &Path) -> Result<Vec<SavedTemplate>, String> {
    read_library(root).map(|(templates, _)| templates)
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

fn prefix_matches(prefix: &str, source: &str) -> bool {
    prefix == "*"
        || source == prefix.trim_end_matches('/')
        || source.starts_with(&format!("{}/", prefix.trim_end_matches('/')))
}

fn allow_source_with(
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
        return Err(format!("{prefix} is blocked by kit.allowedSources (source: {}). Ask the administrator or update the controlling environment policy; Sandy will not replace it", record["source"]));
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
        return Err(format!("{prefix} remains blocked by the evaluated kit.allowedSources policy; ask the administrator to authorize this repository"));
    }
    Ok(format!(
        "Authorized {prefix}; existing source prefixes preserved"
    ))
}

pub fn launch_template(
    root: &Path,
    id: &str,
    name: Option<String>,
    workspace: Option<String>,
) -> Result<String, String> {
    launch_with(root, id, name, workspace, &mut SbxRuntime)
}

trait SandboxRuntime {
    fn run(&mut self, args: &[&str]) -> Result<String, String>;
    fn exists(&mut self, name: &str) -> Result<bool, String>;
    fn start(&mut self, name: &str) -> Result<String, String>;
}

struct SbxRuntime;

impl SandboxRuntime for SbxRuntime {
    fn run(&mut self, args: &[&str]) -> Result<String, String> {
        crate::sbx::run_capture(args)
    }

    fn exists(&mut self, name: &str) -> Result<bool, String> {
        Ok(crate::sbx::list_sandboxes()?
            .iter()
            .any(|sandbox| sandbox.name == name))
    }

    fn start(&mut self, name: &str) -> Result<String, String> {
        crate::sbx::start_sandbox(name)
    }
}

fn launch_with(
    root: &Path,
    id: &str,
    name: Option<String>,
    workspace: Option<String>,
    runtime: &mut impl SandboxRuntime,
) -> Result<String, String> {
    let (record, saved, environment) = {
        let _guard = LIBRARY.lock().map_err(|e| e.to_string())?;
        let record = records(root)?
            .into_iter()
            .find(|r| r.id == id)
            .ok_or_else(|| format!("Unknown saved template: {id}"))?;
        let (saved, environment) = read_template(&record)?;
        (record, saved, environment)
    };
    let name = name.unwrap_or_else(|| saved.name.clone());
    validate_name(&name)?;
    // Resuming an existing name preserves its recorded configuration and does
    // not replay environment creation, host hooks, or source authorization.
    if runtime
        .exists(&name)
        .map_err(|error| format!("Cannot check whether sandbox {name} already exists:\n{error}"))?
    {
        let output = runtime.start(&name).map_err(|error| {
            format!("Existing sandbox {name} could not be started. Retry with sbx run --detach --name {name}:\n{error}")
        })?;
        return Ok(format!("Started existing sandbox {name} in the background using its existing agent, kits, and workspace.\n{output}"));
    }
    let workspace = workspace_path(workspace.as_deref().unwrap_or(&saved.workspace))?;
    let original =
        std::fs::canonicalize(&record.path).map_err(|e| format!("{}: {e}", record.path))?;
    ensure_outside(&original, &workspace)?;
    let providers = get_providers(root)?;
    let mut authorized = HashSet::new();
    for kit in selected(&record.agent, &record.mixins) {
        if let Some(url) = kit.source.strip_prefix("git+https://github.com/") {
            let repository = url
                .split('#')
                .next()
                .unwrap_or_default()
                .trim_end_matches(".git");
            let provider = providers
                .iter()
                .find(|p| {
                    p.enabled
                        && p.kind == "github"
                        && github_repository(&p.repository).is_ok_and(|r| r == repository)
                })
                .ok_or_else(|| format!("Enable the GitHub provider for {repository} before launching this template"))?;
            if authorized.insert(repository.to_string()) {
                allow_source_with(provider, &mut |args| runtime.run(args))?;
            }
        } else if !kit.source.starts_with("docker.io/") {
            return Err(format!("Unsupported saved source: {}. Select a kit from an enabled GitHub or Docker Hub provider", kit.source));
        }
    }
    // Use an immutable per-launch snapshot, preserving the original file and the
    // sbx environment journal's file identity for later env commands.
    let directory = root.join("launches").join(unique_id());
    let mut ancestor = directory.as_path();
    while !ancestor.exists() {
        ancestor = ancestor
            .parent()
            .ok_or("Launch directory has no existing parent")?;
    }
    ensure_outside(
        &std::fs::canonicalize(ancestor).map_err(|e| e.to_string())?,
        &workspace,
    )?;
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let path = std::fs::canonicalize(directory)
        .map_err(|e| e.to_string())?
        .join("sbxenv.yaml");
    ensure_outside(&path, &workspace)?;
    // Edit the decoded document so manually added environment settings survive.
    let mut document: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(&saved.content).map_err(|e| e.to_string())?;
    document["name"] = serde_yaml_ng::Value::String(name.clone());
    document["workspace"] = match environment.workspace {
        Workspace::Object { clone, .. } => serde_yaml_ng::to_value(Workspace::Object {
            path: escape(&path_string(&workspace)?),
            clone,
        })
        .map_err(|e| e.to_string())?,
        Workspace::Path(_) => serde_yaml_ng::Value::String(escape(&path_string(&workspace)?)),
    };
    let content = serde_yaml_ng::to_string(&document).map_err(|e| e.to_string())?;
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    file.write_all(content.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    let path = path_string(&path)?;
    let created = runtime.run(&["env", "create", "--force", "--name", &name, &path])
        .map_err(|e| format!("Environment creation failed. Launch file retained at {path} for diagnostics and retry:\n{e}"))?;
    let started = runtime.start(&name).map_err(|error| {
        format!("Environment {name} was created, but it could not be started. Retry with sbx run --detach --name {name}; launching this name again will resume it without recreating it.\nLaunch file: {path}\nCreate output: {created}\n{error}")
    })?;
    Ok(format!("Created and started sandbox {name} in the background without attaching an interactive session.\nLaunch file: {path}\n{created}\n{started}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::KitArgument;

    #[derive(Default)]
    struct MockRuntime {
        calls: Vec<Vec<String>>,
        existing: bool,
        fail_create: bool,
        fail_start: bool,
    }

    fn assert_library_available() {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while LIBRARY.try_lock().is_err() {
            assert!(
                std::time::Instant::now() < deadline,
                "library lock held during CLI work"
            );
            std::thread::yield_now();
        }
    }

    impl SandboxRuntime for MockRuntime {
        fn run(&mut self, args: &[&str]) -> Result<String, String> {
            assert_library_available();
            self.calls
                .push(args.iter().map(|arg| arg.to_string()).collect());
            if self.fail_create {
                return Err("create failed".into());
            }
            Ok("created".into())
        }

        fn exists(&mut self, name: &str) -> Result<bool, String> {
            assert_library_available();
            self.calls.push(vec!["exists".into(), name.into()]);
            Ok(self.existing)
        }

        fn start(&mut self, name: &str) -> Result<String, String> {
            assert_library_available();
            self.calls.push(vec![
                "run".into(),
                "--detach".into(),
                "--name".into(),
                name.into(),
            ]);
            if self.fail_start {
                return Err("start failed".into());
            }
            Ok("started".into())
        }
    }

    struct Fixture {
        base: PathBuf,
        root: PathBuf,
        workspace: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let base = std::env::temp_dir().join(format!("sandy-env-{}", unique_id()));
            let root = base.join("app-data");
            let workspace = base.join("workspace");
            std::fs::create_dir_all(&root).unwrap();
            std::fs::create_dir_all(&workspace).unwrap();
            Self {
                base,
                root,
                workspace,
            }
        }
        fn request(&self) -> SaveTemplateRequest {
            SaveTemplateRequest {
                name: "example".into(),
                workspace: path_string(&self.workspace).unwrap(),
                agent: kit("claude", "agent", 2, "builtin"),
                mixins: vec![],
                arguments: BTreeMap::new(),
                save_path: None,
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.base);
        }
    }
    fn kit(name: &str, kind: &str, version: u8, provider: &str) -> KitOption {
        KitOption {
            id: name.into(),
            name: name.into(),
            description: String::new(),
            source: if provider == "builtin" {
                name.into()
            } else {
                format!("docker.io/team/{name}:1.0")
            },
            schema_version: version,
            kind: kind.into(),
            provider_id: provider.into(),
            required_agent: None,
            arguments: vec![],
        }
    }

    #[test]
    fn rejects_version_mixing_required_agent_and_duplicate_names() {
        let fixture = Fixture::new();
        let mut request = fixture.request();
        request.mixins.push(kit("browser", "mixin", 3, "hub"));
        assert!(save_template(&fixture.root, request.clone())
            .unwrap_err()
            .contains("cannot be combined"));
        request.mixins[0].schema_version = 2;
        request.mixins[0].required_agent = Some("codex".into());
        assert!(save_template(&fixture.root, request.clone())
            .unwrap_err()
            .contains("requires agent codex"));
        request.mixins[0] = kit("claude", "mixin", 2, "hub");
        assert!(save_template(&fixture.root, request)
            .unwrap_err()
            .contains("unique"));
    }

    #[test]
    fn structural_yaml_arguments_and_custom_agent_sources_round_trip() {
        let fixture = Fixture::new();
        let mut request = fixture.request();
        request.agent = kit("custom", "agent", 2, "hub");
        request.agent.arguments.push(KitArgument {
            name: "message".into(),
            description: String::new(),
            required: true,
            default: None,
            choices: vec![],
        });
        let value = "true: 'quoted'\nnew: [items]\\\"${{ env.args.not_an_expression }}";
        request.arguments.insert(
            "custom".into(),
            BTreeMap::from([("message".into(), value.into())]),
        );
        let saved = save_template(&fixture.root, request).unwrap();
        let environment: Environment = serde_yaml_ng::from_str(&saved.content).unwrap();
        assert_eq!(environment.schema_version, "1");
        assert_eq!(environment.agent, "custom");
        assert_eq!(environment.kits[0].source(), "docker.io/team/custom:1.0");
        assert_eq!(environment.kits[0].arguments()["message"], value);
        assert_eq!(
            list_templates(&fixture.root).unwrap()[0].content,
            saved.content
        );
    }

    #[test]
    fn v3_workload_is_first_kit_and_persistence_reads_disk() {
        let fixture = Fixture::new();
        let mut request = fixture.request();
        request.agent = kit("sbx-kit-claude", "workload", 3, "hub");
        request.mixins.push(kit("browser", "mixin", 3, "hub"));
        let saved = save_template(&fixture.root, request).unwrap();
        let environment: Environment = serde_yaml_ng::from_str(&saved.content).unwrap();
        assert_eq!(environment.kits.len(), 2);
        assert_eq!(
            environment.kits[0].source(),
            "docker.io/team/sbx-kit-claude:1.0"
        );
        std::fs::write(
            &saved.path,
            saved.content.replace("name: example", "name: changed"),
        )
        .unwrap();
        assert_eq!(list_templates(&fixture.root).unwrap()[0].name, "changed");
        std::fs::remove_file(&saved.path).unwrap();
        assert!(list_templates(&fixture.root).unwrap().is_empty());
        let warnings = template_warnings(&fixture.root).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains(&saved.path));
    }

    #[test]
    fn stale_template_does_not_hide_readable_library_entries() {
        let fixture = Fixture::new();
        let readable = save_template(&fixture.root, fixture.request()).unwrap();
        let invalid = save_template(&fixture.root, fixture.request()).unwrap();
        std::fs::write(&invalid.path, "invalid: [yaml").unwrap();

        let templates = list_templates(&fixture.root).unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].id, readable.id);
        let warnings = template_warnings(&fixture.root).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains(&invalid.path));
        assert!(warnings[0].contains("invalid environment YAML"));
    }

    #[test]
    fn prevents_workspace_files_and_overwrite() {
        let fixture = Fixture::new();
        let mut request = fixture.request();
        request.save_path = Some(path_string(&fixture.workspace.join("sbxenv.yaml")).unwrap());
        assert!(save_template(&fixture.root, request.clone())
            .unwrap_err()
            .contains("outside"));
        let existing = fixture.base.join("important.yaml");
        std::fs::write(&existing, "important").unwrap();
        request.save_path = Some(path_string(&existing).unwrap());
        assert!(save_template(&fixture.root, request)
            .unwrap_err()
            .contains("never overwritten"));
        assert_eq!(std::fs::read_to_string(existing).unwrap(), "important");
    }

    #[test]
    fn launches_noninteractively_with_name_workspace_override_and_no_mutation() {
        let fixture = Fixture::new();
        let saved = save_template(&fixture.root, fixture.request()).unwrap();
        let other = fixture.base.join("other workspace");
        std::fs::create_dir(&other).unwrap();
        let mut runtime = MockRuntime::default();
        let result = launch_with(
            &fixture.root,
            &saved.id,
            Some("second".into()),
            Some(path_string(&other).unwrap()),
            &mut runtime,
        )
        .unwrap();
        assert!(result.contains("Created and started sandbox second"));
        assert_eq!(runtime.calls[0], ["exists", "second"]);
        assert_eq!(
            &runtime.calls[1][..5],
            ["env", "create", "--force", "--name", "second"]
        );
        assert_eq!(runtime.calls[2], ["run", "--detach", "--name", "second"]);
        let launch: Environment =
            serde_yaml_ng::from_str(&std::fs::read_to_string(&runtime.calls[1][5]).unwrap())
                .unwrap();
        assert_eq!(launch.workspace.path(), path_string(&other).unwrap());
        assert_eq!(std::fs::read_to_string(saved.path).unwrap(), saved.content);
    }

    #[test]
    fn existing_sandbox_is_resumed_without_recreating_environment() {
        let fixture = Fixture::new();
        let saved = save_template(&fixture.root, fixture.request()).unwrap();
        let mut runtime = MockRuntime {
            existing: true,
            ..Default::default()
        };
        let result = launch_with(&fixture.root, &saved.id, None, None, &mut runtime).unwrap();
        assert!(result.contains("Started existing sandbox example"));
        assert_eq!(
            runtime.calls,
            [
                vec!["exists", "example"],
                vec!["run", "--detach", "--name", "example"]
            ]
        );
        assert!(!fixture.root.join("launches").exists());
    }

    #[test]
    fn create_failure_does_not_attempt_to_start() {
        let fixture = Fixture::new();
        let saved = save_template(&fixture.root, fixture.request()).unwrap();
        let mut runtime = MockRuntime {
            fail_create: true,
            ..Default::default()
        };
        let error = launch_with(&fixture.root, &saved.id, None, None, &mut runtime).unwrap_err();
        assert!(error.contains("Environment creation failed"));
        assert_eq!(runtime.calls.len(), 2);
    }

    #[test]
    fn start_failure_reports_created_environment_and_retry_command() {
        let fixture = Fixture::new();
        let saved = save_template(&fixture.root, fixture.request()).unwrap();
        let mut runtime = MockRuntime {
            fail_start: true,
            ..Default::default()
        };
        let error = launch_with(&fixture.root, &saved.id, None, None, &mut runtime).unwrap_err();
        assert!(error.contains("was created, but it could not be started"));
        assert!(error.contains("sbx run --detach --name example"));
        assert_eq!(runtime.calls.len(), 3);
        assert_eq!(std::fs::read_to_string(saved.path).unwrap(), saved.content);
    }

    #[test]
    #[cfg(unix)]
    fn production_launch_starts_through_sbx_and_resumes_without_create() {
        use std::os::unix::fs::PermissionsExt;

        struct RestorePath(Option<std::ffi::OsString>);
        impl Drop for RestorePath {
            fn drop(&mut self) {
                if let Some(path) = &self.0 {
                    std::env::set_var("PATH", path);
                } else {
                    std::env::remove_var("PATH");
                }
            }
        }

        let _guard = crate::sbx::CLI_TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let fixture = Fixture::new();
        let saved = save_template(&fixture.root, fixture.request()).unwrap();
        let executable = fixture.base.join("sbx");
        std::fs::write(
            &executable,
            r#"#!/bin/sh
base="${0%/*}"
printf '%s\n' "$*" >> "$base/commands"
case "$1" in
  ls)
    if [ -f "$base/created" ]; then
      printf '%s\n' '[{"name":"example"}]'
    else
      printf '%s\n' '[]'
    fi
    ;;
  env)
    [ "$2" = create ] || exit 1
    : > "$base/created"
    printf '%s\n' 'created'
    ;;
  run) printf '%s\n' 'started' ;;
  *) exit 1 ;;
esac
"#,
        )
        .unwrap();
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
        let restore = RestorePath(std::env::var_os("PATH"));
        let path = std::env::join_paths(std::iter::once(fixture.base.clone()).chain(
            std::env::split_paths(restore.0.as_deref().unwrap_or_default()),
        ))
        .unwrap();
        std::env::set_var("PATH", path);

        let created = launch_template(&fixture.root, &saved.id, None, None).unwrap();
        assert!(created.contains("Created and started sandbox example"));
        let resumed = launch_template(&fixture.root, &saved.id, None, None).unwrap();
        assert!(resumed.contains("Started existing sandbox example"));
        let commands = std::fs::read_to_string(fixture.base.join("commands")).unwrap();
        let commands: Vec<_> = commands.lines().collect();
        assert_eq!(commands.len(), 7, "{commands:?}");
        assert_eq!(commands[0], "ls --json");
        assert!(commands[1].starts_with("env create --force --name example "));
        assert_eq!(commands[2], "ls --json");
        assert_eq!(commands[3], "run --detach --name example");
        assert_eq!(commands[4], "ls --json");
        assert_eq!(commands[5], "ls --json");
        assert_eq!(commands[6], "run --detach --name example");
        assert_eq!(std::fs::read_to_string(saved.path).unwrap(), saved.content);
    }

    #[test]
    fn source_authorization_merges_and_rejects_managed_policy() {
        let provider = Provider {
            id: "custom".into(),
            label: "Custom".into(),
            kind: "github".into(),
            repository: "team/kits".into(),
            reference: "v1".into(),
            enabled: true,
        };
        let mut calls = Vec::new();
        allow_source_with(&provider, &mut |args| {
            calls.push(args.iter().map(|a| a.to_string()).collect::<Vec<_>>());
            Ok(match calls.len() {
                1 => r#"{"source":"override","value":["docker.io/","ghcr.io/trusted/"]}"#.into(),
                2 => String::new(),
                _ => r#"{"source":"override","value":["docker.io/","ghcr.io/trusted/","github.com/team/kits","github.com/team/kits.git"]}"#.into(),
            })
        }).unwrap();
        let prefixes: Vec<String> = serde_json::from_str(&calls[1][3]).unwrap();
        assert_eq!(
            prefixes,
            [
                "docker.io/",
                "ghcr.io/trusted/",
                "github.com/team/kits",
                "github.com/team/kits.git"
            ]
        );
        assert!(allow_source_with(&provider, &mut |_| Ok(
            r#"{"source":"envvar","value":["docker.io/"]}"#.into()
        ))
        .unwrap_err()
        .contains("administrator"));
        assert!(!prefix_matches(
            "github.com/team/kits",
            "github.com/team/kits-evil"
        ));
    }
}
