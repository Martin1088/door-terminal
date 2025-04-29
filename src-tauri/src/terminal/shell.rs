use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::{
    io::{BufRead, BufReader, Read, Write},
    process::exit,
    sync::Arc,
    thread::{self},
};
use tauri::{async_runtime::Mutex as AsyncMutex, AppHandle, Emitter, State};
use crate::terminal::process_parser::strip_ansi_codes;

pub struct AppState {
    pub pty_pair: Arc<AsyncMutex<PtyPair>>,
    pub writer: Arc<AsyncMutex<Box<dyn Write + Send>>>,
}

#[tauri::command]
pub async fn async_write_to_pty(data: &str, state: State<'_, AppState>) -> Result<(), ()> {
    write!(state.writer.lock().await, "{}", data).map_err(|_| ())
}

#[tauri::command]
pub async fn async_resize_pty(rows: u16, cols: u16, state: State<'_, AppState>) -> Result<(), ()> {
    state
        .pty_pair
        .lock()
        .await
        .master
        .resize(PtySize {
            rows,
            cols,
            ..Default::default()
        })
        .map_err(|_| ())
}

#[tauri::command]
pub async fn async_create_shell(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    log::info!("Creating shell");
    #[cfg(target_os = "windows")]
    let mut cmd = CommandBuilder::new("powershell.exe");

    #[cfg(not(target_os = "windows"))]
    let mut cmd = CommandBuilder::new("zsh");
    log::debug!("Prepared CommandBuilder for zsh");

    #[cfg(target_os = "windows")]
    cmd.env("TERM", "cygwin");

    #[cfg(not(target_os = "windows"))]
    cmd.env("TERM", "xterm-256color");

    let pty_pair = state.pty_pair.lock().await;
    let slave = &pty_pair.slave;
    log::debug!("Got PTY slave");
    let mut child = match slave.spawn_command(cmd) {
        Ok(child) => {
            log::info!("Shell spawned successfully");
            child
        }
        Err(err) => {
            log::error!("Failed to spawn shell: {}", err);
            return Err(format!("Spawn error: {}", err));
        }
    };

    // Spawn reader thread
    let reader = match pty_pair
        .master
        .try_clone_reader() {
        Ok(reader) => reader,
        Err(err) => {
            log::error!("Failed to clone PTY reader: {}", err);
            exit(1);
        }
    };
    drop(pty_pair);
    spawn_reader_thread(reader, app.clone());

    // Optional: monitor shell exit
    thread::spawn(move || {
        if let Ok(status) = child.wait() {
            let _ = app.emit("shell-exit", status.to_string());
        }
    });
    log::info!("Shell created");

    Ok(())
}

fn spawn_reader_thread<R: Read + Send + 'static>(mut reader: R, app_handle: AppHandle) {
    log::info!("Spawned PTY reader thread");
    thread::spawn(move || {
        let mut buffer = [0u8; 1024];
        loop {
            match reader.read(&mut buffer) {
                Ok(n) if n > 0 => {
                    let raw_output = String::from_utf8_lossy(&buffer[..n]).to_string();
                    let clean_output = strip_ansi_codes(&raw_output);
                    log::debug!("PTY Output: {}", clean_output);
                    let _ = app_handle.emit("terminal-output", clean_output);
                }
                Err(err) => {
                    log::error!("PTY reader error: {}", err);
                    break;
                }
                _ => break,
            }
        }
    });
}