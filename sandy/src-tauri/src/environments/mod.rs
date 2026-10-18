//! Persisted environment library. All paths are supplied by the caller (Tauri's
//! AppHandle.path().app_data_dir()); tests use isolated temporary directories.
//! `launch_template` creates when needed, then starts the sandbox without attaching.
//!
//! The environment schema remains "1" for both kit generations. Custom v2
//! agents and v3 workloads register their name via kits; built-in v2 shortcuts
//! need only agent=name. See Docker's configuration/environment-files reference.
//! `sbx env create --help` verifies --force, --name, and file PATH support;
//! it explicitly directs callers to env run to attach afterward.

mod launch;
mod library;
mod schema;
mod source_policy;
mod validation;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests;

pub use launch::launch_template;
pub use library::{
    SaveTemplateRequest, SavedTemplate, delete_template, list_templates, save_template,
    template_warnings,
};
pub use source_policy::allow_source;

use std::sync::Mutex;

use self::validation::{path_string, strip_verbatim};

static LIBRARY: Mutex<()> = Mutex::new(());
static SOURCE_POLICY: Mutex<()> = Mutex::new(());

pub fn pick_workspace() -> Result<Option<String>, String> {
    rfd::FileDialog::new()
        .set_title("Choose the sandbox workspace")
        .pick_folder()
        .map(|path| path_string(&strip_verbatim(&path)))
        .transpose()
}

pub fn pick_template_path() -> Result<Option<String>, String> {
    rfd::FileDialog::new()
        .set_title("Save the environment template outside your workspace")
        .set_file_name("sbxenv.yaml")
        .add_filter("Environment YAML", &["yaml", "yml"])
        .save_file()
        .map(|path| path_string(&strip_verbatim(&path)))
        .transpose()
}
