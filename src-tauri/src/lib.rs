use crate::terminal::shell::{async_create_shell, async_resize_pty, async_write_to_pty, AppState};
use portable_pty::{native_pty_system, PtySize};
use std::sync::Arc;
use tauri::{async_runtime::Mutex as AsyncMutex, State};
use crate::terminal::navigate_dir::list_directory;

pub mod terminal;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let pty_system = native_pty_system();
    let pty_pair = match pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        }) {
        Ok(pty_pair) => pty_pair,
        Err(e) => {
            log::debug!("Error opening pty: {}", e);
            std::process::exit(1);
        }
    };

    let writer = pty_pair.master.take_writer().unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .manage(AppState {
            pty_pair: Arc::new(AsyncMutex::new(pty_pair)),
            writer: Arc::new(AsyncMutex::new(writer)),
        })
        .invoke_handler(tauri::generate_handler![
            async_create_shell,
            async_write_to_pty,
            async_resize_pty,
            list_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
