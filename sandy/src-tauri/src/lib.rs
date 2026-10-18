mod catalog;
mod commands;
mod environments;
mod sbx;
mod setup;
mod updates;

use tauri::{WebviewUrl, WebviewWindowBuilder};

// App entry point: there is no caller to return an error to, so a fatal
// startup failure may panic.
#[allow(clippy::expect_used, clippy::panic)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(commands::handlers())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Boot splash: a small transparent window with just the logo, a
            // spinner, and a label, shown while the hidden main window loads.
            // The main window's frontend reveals itself and closes this splash
            // once the first status check settles.
            WebviewWindowBuilder::new(app, "splash", WebviewUrl::App("index.html".into()))
                .title("Sandy")
                .inner_size(320.0, 200.0)
                .center()
                .decorations(false)
                .transparent(true)
                .resizable(false)
                .maximizable(false)
                .minimizable(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .focused(false)
                .build()?;
            Ok(())
        })
        // `run()` returns () and there is nowhere left to report a fatal
        // startup error; exiting with the message is the only option here.
        .run(tauri::generate_context!())
        .expect("error while running Sandy");
}
