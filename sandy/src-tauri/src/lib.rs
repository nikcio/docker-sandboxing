mod sbx;

use serde::Serialize;
use sbx::{
    ensure_sbx_installed, is_sbx_installed, list_sandboxes, run_login, set_clipboard_image_paste,
};

#[derive(Serialize)]
pub struct SetupStatus {
    pub sbx_installed: bool,
    pub signed_in: bool,
    pub image_paste_on: bool,
    pub ssh_set_up: bool,
}

#[tauri::command]
fn status() -> Result<SetupStatus, String> {
    let sbx_installed = is_sbx_installed();
    if !sbx_installed {
        return Ok(SetupStatus {
            sbx_installed,
            signed_in: false,
            image_paste_on: false,
            ssh_set_up: false,
        });
    }
    let signed_in = sbx::is_signed_in()?;
    // Settings live behind the daemon; don't fail the whole status read
    // when it's down — report the setting as off instead.
    let image_paste_on = sbx::get_setting("clipboard.imagePaste")
        .map(|v| v == "true")
        .unwrap_or(false);
    let ssh_set_up = sbx::is_ssh_set_up()?;
    Ok(SetupStatus {
        sbx_installed,
        signed_in,
        image_paste_on,
        ssh_set_up,
    })
}

#[tauri::command]
fn install_sbx() -> Result<String, String> {
    ensure_sbx_installed()
}

#[tauri::command]
fn enable_image_paste() -> Result<(), String> {
    set_clipboard_image_paste()
}

#[tauri::command]
fn setup_ssh() -> Result<String, String> {
    sbx::run_capture(&["setup", sbx::SSH_SETUP_COMMAND])
}

#[tauri::command]
fn login() -> Result<String, String> {
    run_login()
}

#[tauri::command]
fn sandboxes() -> Result<Vec<sbx::SandboxInfo>, String> {
    list_sandboxes()
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            status,
            install_sbx,
            enable_image_paste,
            setup_ssh,
            login,
            sandboxes
        ])
        .run(tauri::generate_context!())
        .expect("error while running Sandy");
}
