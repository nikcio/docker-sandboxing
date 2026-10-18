//! Tauri command handlers. Each command is a thin wrapper that moves blocking
//! work onto Tauri's blocking worker pool; the domain logic lives in `sbx`,
//! `catalog`, `environments`, and `setup`.

use std::path::PathBuf;

use tauri::Manager;

use crate::{catalog, environments, sbx, setup};

pub(crate) async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|error| error.to_string())?
}

fn data_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|error| error.to_string())
}

#[tauri::command]
async fn status() -> Result<setup::SetupStatus, String> {
    blocking(setup::read_status).await
}

#[tauri::command]
async fn install_sbx() -> Result<String, String> {
    blocking(sbx::ensure_sbx_installed).await
}

#[tauri::command]
async fn enable_image_paste() -> Result<(), String> {
    blocking(sbx::set_clipboard_image_paste).await
}

#[tauri::command]
async fn setup_ssh() -> Result<String, String> {
    blocking(|| sbx::run_capture(&["setup", sbx::SSH_SETUP_COMMAND])).await
}

#[tauri::command]
async fn login() -> Result<String, String> {
    blocking(sbx::run_login).await
}

#[tauri::command]
async fn sandboxes() -> Result<Vec<sbx::SandboxInfo>, String> {
    blocking(sbx::list_sandboxes).await
}

#[tauri::command]
async fn start_sandbox(name: String) -> Result<String, String> {
    blocking(move || sbx::start_sandbox(&name)).await
}

#[tauri::command]
async fn stop_sandbox(name: String) -> Result<String, String> {
    blocking(move || sbx::stop_sandbox(&name)).await
}

#[tauri::command]
async fn restart_sandbox(name: String) -> Result<String, String> {
    blocking(move || sbx::restart_sandbox(&name)).await
}

#[tauri::command]
async fn remove_sandbox(name: String) -> Result<String, String> {
    blocking(move || sbx::remove_sandbox(&name)).await
}

#[tauri::command]
async fn open_terminal(name: String) -> Result<(), String> {
    blocking(move || sbx::open_terminal(&name)).await
}

#[tauri::command]
async fn connection_info(name: String) -> Result<sbx::ConnectionInfo, String> {
    blocking(move || sbx::connection_info(&name)).await
}

#[tauri::command]
async fn get_providers(app: tauri::AppHandle) -> Result<Vec<catalog::Provider>, String> {
    let root = data_dir(&app)?;
    blocking(move || catalog::get_providers(&root)).await
}

#[tauri::command]
async fn save_providers(
    app: tauri::AppHandle,
    providers: Vec<catalog::Provider>,
) -> Result<Vec<catalog::Provider>, String> {
    let root = data_dir(&app)?;
    blocking(move || catalog::save_providers(&root, providers)).await
}

#[tauri::command]
async fn provider_versions(provider: catalog::Provider) -> Result<Vec<String>, String> {
    blocking(move || catalog::provider_versions(provider)).await
}

#[tauri::command]
async fn discover_kits(
    providers: Vec<catalog::Provider>,
    query: Option<String>,
) -> Result<catalog::Catalog, String> {
    blocking(move || catalog::discover_kits(providers, query)).await
}

#[tauri::command]
async fn save_template(
    app: tauri::AppHandle,
    request: environments::SaveTemplateRequest,
) -> Result<environments::SavedTemplate, String> {
    let root = data_dir(&app)?;
    blocking(move || environments::save_template(&root, request)).await
}

#[tauri::command]
async fn list_templates(app: tauri::AppHandle) -> Result<Vec<environments::SavedTemplate>, String> {
    let root = data_dir(&app)?;
    blocking(move || environments::list_templates(&root)).await
}

#[tauri::command]
async fn template_warnings(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let root = data_dir(&app)?;
    blocking(move || environments::template_warnings(&root)).await
}

#[tauri::command]
async fn launch_template(
    app: tauri::AppHandle,
    id: String,
    name: Option<String>,
    workspace: Option<String>,
) -> Result<String, String> {
    let root = data_dir(&app)?;
    blocking(move || environments::launch_template(&root, &id, name, workspace)).await
}

#[tauri::command]
async fn allow_source(app: tauri::AppHandle, provider_id: String) -> Result<String, String> {
    let root = data_dir(&app)?;
    blocking(move || environments::allow_source(&root, &provider_id)).await
}

#[tauri::command]
async fn pick_workspace() -> Result<Option<String>, String> {
    blocking(environments::pick_workspace).await
}

#[tauri::command]
async fn pick_template_path() -> Result<Option<String>, String> {
    blocking(environments::pick_template_path).await
}

pub(crate) fn handlers() -> Box<tauri::ipc::InvokeHandler<tauri::Wry>> {
    Box::new(tauri::generate_handler![
        status,
        install_sbx,
        enable_image_paste,
        setup_ssh,
        login,
        sandboxes,
        start_sandbox,
        stop_sandbox,
        restart_sandbox,
        remove_sandbox,
        open_terminal,
        connection_info,
        get_providers,
        save_providers,
        provider_versions,
        discover_kits,
        save_template,
        list_templates,
        template_warnings,
        launch_template,
        allow_source,
        pick_workspace,
        pick_template_path
    ])
}
