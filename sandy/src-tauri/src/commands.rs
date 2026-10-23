//! Tauri command handlers. Each command is a thin wrapper that moves blocking
//! work onto Tauri's blocking worker pool; the domain logic lives in `sbx`,
//! `catalog`, `environments`, and `setup`.

use std::future::Future;
use std::path::PathBuf;

use tauri::Manager;

use crate::{catalog, diaglog, environments, sbx, setup};

/// Log every Tauri command outcome (command name, ok/error, and the error
/// text on failure). Successes carry no detail; the domain loggers already
/// recorded the underlying CLI/HTTP work.
async fn logged<T, E>(
    command: &'static str,
    work: impl Future<Output = Result<T, E>>,
) -> Result<T, E>
where
    E: std::fmt::Display,
{
    let result = work.await;
    match &result {
        Ok(_) => crate::diaglog::write_record("command", command, None, Some("ok")),
        Err(error) => crate::diaglog::write_record(
            "command",
            command,
            Some(&error.to_string()),
            Some("error"),
        ),
    }
    result
}

async fn blocking<T: Send + 'static>(
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
async fn check_setup() -> Result<setup::SetupStatus, String> {
    logged(
        "check_setup",
        async move { blocking(setup::check_setup).await },
    )
    .await
}

#[tauri::command]
async fn install_sbx() -> Result<String, String> {
    logged("install_sbx", async move {
        blocking(sbx::ensure_sbx_installed).await
    })
    .await
}

#[tauri::command]
async fn enable_image_paste() -> Result<(), String> {
    logged("enable_image_paste", async move {
        blocking(sbx::set_clipboard_image_paste).await
    })
    .await
}

#[tauri::command]
async fn setup_ssh() -> Result<String, String> {
    logged("setup_ssh", async move {
        blocking(|| sbx::run_capture(&["setup", sbx::SSH_SETUP_COMMAND])).await
    })
    .await
}

#[tauri::command]
async fn login() -> Result<String, String> {
    logged("login", async move { blocking(sbx::run_login).await }).await
}

#[tauri::command]
async fn sandboxes() -> Result<Vec<sbx::SandboxInfo>, String> {
    logged(
        "sandboxes",
        async move { blocking(sbx::list_sandboxes).await },
    )
    .await
}

#[tauri::command]
async fn start_sandbox(name: String) -> Result<String, String> {
    logged("start_sandbox", async move {
        blocking(move || sbx::start_sandbox(&name)).await
    })
    .await
}

#[tauri::command]
async fn stop_sandbox(name: String) -> Result<String, String> {
    logged("stop_sandbox", async move {
        blocking(move || sbx::stop_sandbox(&name)).await
    })
    .await
}

#[tauri::command]
async fn restart_sandbox(name: String) -> Result<String, String> {
    logged("restart_sandbox", async move {
        blocking(move || sbx::restart_sandbox(&name)).await
    })
    .await
}

#[tauri::command]
async fn remove_sandbox(name: String) -> Result<String, String> {
    logged("remove_sandbox", async move {
        blocking(move || sbx::remove_sandbox(&name)).await
    })
    .await
}

#[tauri::command]
async fn open_terminal(name: String) -> Result<(), String> {
    logged("open_terminal", async move {
        blocking(move || sbx::open_terminal(&name)).await
    })
    .await
}

#[tauri::command]
async fn connection_info(name: String) -> Result<sbx::ConnectionInfo, String> {
    logged("connection_info", async move {
        blocking(move || sbx::connection_info(&name)).await
    })
    .await
}

#[tauri::command]
async fn get_providers(app: tauri::AppHandle) -> Result<Vec<catalog::Provider>, String> {
    logged("get_providers", async move {
        let root = data_dir(&app)?;
        blocking(move || catalog::get_providers(&root)).await
    })
    .await
}

#[tauri::command]
async fn save_providers(
    app: tauri::AppHandle,
    providers: Vec<catalog::Provider>,
) -> Result<Vec<catalog::Provider>, String> {
    logged("save_providers", async move {
        let root = data_dir(&app)?;
        blocking(move || catalog::save_providers(&root, providers)).await
    })
    .await
}

#[tauri::command]
async fn provider_versions(provider: catalog::Provider) -> Result<Vec<String>, String> {
    logged("provider_versions", async move {
        blocking(move || catalog::provider_versions(provider)).await
    })
    .await
}

#[tauri::command]
async fn discover_kits(
    providers: Vec<catalog::Provider>,
    query: Option<String>,
) -> Result<catalog::Catalog, String> {
    logged("discover_kits", async move {
        blocking(move || catalog::discover_kits(providers, query)).await
    })
    .await
}

#[tauri::command]
async fn save_template(
    app: tauri::AppHandle,
    request: environments::SaveTemplateRequest,
) -> Result<environments::SavedTemplate, String> {
    logged("save_template", async move {
        let root = data_dir(&app)?;
        blocking(move || environments::save_template(&root, request)).await
    })
    .await
}

#[tauri::command]
async fn list_templates(app: tauri::AppHandle) -> Result<Vec<environments::SavedTemplate>, String> {
    logged("list_templates", async move {
        let root = data_dir(&app)?;
        blocking(move || environments::list_templates(&root)).await
    })
    .await
}

#[tauri::command]
async fn template_warnings(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    logged("template_warnings", async move {
        let root = data_dir(&app)?;
        blocking(move || environments::template_warnings(&root)).await
    })
    .await
}

#[tauri::command]
async fn delete_template(app: tauri::AppHandle, id: String) -> Result<(), String> {
    logged("delete_template", async move {
        let root = data_dir(&app)?;
        blocking(move || environments::delete_template(&root, &id)).await
    })
    .await
}

#[tauri::command]
async fn launch_template(
    app: tauri::AppHandle,
    id: String,
    name: Option<String>,
    workspace: Option<String>,
) -> Result<String, String> {
    logged("launch_template", async move {
        let root = data_dir(&app)?;
        blocking(move || environments::launch_template(&root, &id, name, workspace)).await
    })
    .await
}

#[tauri::command]
async fn allow_source(app: tauri::AppHandle, provider_id: String) -> Result<String, String> {
    logged("allow_source", async move {
        let root = data_dir(&app)?;
        blocking(move || environments::allow_source(&root, &provider_id)).await
    })
    .await
}

#[tauri::command]
async fn pick_workspace() -> Result<Option<String>, String> {
    logged("pick_workspace", async move {
        blocking(environments::pick_workspace).await
    })
    .await
}

#[tauri::command]
async fn pick_template_path() -> Result<Option<String>, String> {
    logged("pick_template_path", async move {
        blocking(environments::pick_template_path).await
    })
    .await
}

/// The diagnostic log file, read from the end (newest last). The cap keeps
/// the payload bounded at 10 MiB.
#[tauri::command]
async fn read_debug_log(app: tauri::AppHandle) -> Result<String, String> {
    let root = data_dir(&app)?;
    blocking(move || {
        let path = diaglog::log_path(&root);
        std::fs::read_to_string(&path).or_else(|error| match error.kind() {
            // A missing file is an empty log, not a failure: nothing has
            // been logged yet (or the user cleared it).
            std::io::ErrorKind::NotFound => Ok(String::new()),
            _ => Err(format!("Cannot read {}: {error}", path.display())),
        })
    })
    .await
}

/// Delete the diagnostic log. The next write recreates it.
#[tauri::command]
async fn clear_debug_log(app: tauri::AppHandle) -> Result<(), String> {
    let root = data_dir(&app)?;
    blocking(move || {
        let path = diaglog::log_path(&root);
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!("Cannot clear {}: {error}", path.display())),
        }
    })
    .await
}

pub(crate) fn handlers() -> Box<tauri::ipc::InvokeHandler<tauri::Wry>> {
    Box::new(tauri::generate_handler![
        check_setup,
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
        delete_template,
        launch_template,
        allow_source,
        pick_workspace,
        pick_template_path,
        read_debug_log,
        clear_debug_log
    ])
}
