use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtyPair, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::Emitter;

pub struct PtySession {
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    child: Option<Box<dyn portable_pty::Child + Send>>,
    reader_handle: Option<thread::JoinHandle<()>>,
}

#[derive(Default)]
pub struct AppState {
    pub pty: Mutex<Option<PtySession>>,
}

fn default_shell() -> String {
    if cfg!(windows) {
        std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".to_string())
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }
}

fn spawn_pty(app: &tauri::AppHandle, state: &AppState) -> Result<u64, String> {
    // If a session already exists, do nothing.
    if let Ok(guard) = state.pty.lock() {
        if guard.is_some() {
            return Ok(0);
        }
    }

    // Clean up any existing session to avoid duplicate outputs.
    if let Ok(mut guard) = state.pty.lock() {
        if let Some(mut session) = guard.take() {
            if let Some(mut child) = session.child.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
            if let Some(handle) = session.reader_handle.take() {
                let _ = handle.join();
            }
        }
    }

    let pty_system = native_pty_system();
    let pair: PtyPair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("openpty failed: {e}"))?;

    let shell = default_shell();
    let cmd = CommandBuilder::new(shell);
    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("spawn command failed: {e}"))?;

    let master = pair.master;
    let mut reader = master
        .try_clone_reader()
        .map_err(|e| format!("reader failed: {e}"))?;
    let writer = master
        .take_writer()
        .map_err(|e| format!("writer failed: {e}"))?;
    let master = Arc::new(Mutex::new(master));

    let app_handle = app.clone();
    let reader_handle = thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if let Ok(s) = std::str::from_utf8(&buf[..n]) {
                        let _ = app_handle.emit("term-data", s.to_string());
                    }
                }
                Err(_) => break,
            }
        }
    });

    let session = PtySession {
        master,
        writer: Arc::new(Mutex::new(writer)),
        child: Some(child),
        reader_handle: Some(reader_handle),
    };

    let mut guard = state
        .pty
        .lock()
        .map_err(|_| "pty mutex poisoned".to_string())?;
    *guard = Some(session);
    Ok(1)
}

#[tauri::command]
pub fn spawn_terminal(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<u64, String> {
    spawn_pty(&app, &state)
}

#[tauri::command]
pub fn write_to_terminal(data: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let guard = state
        .pty
        .lock()
        .map_err(|_| "pty mutex poisoned".to_string())?;
    if let Some(session) = guard.as_ref() {
        let mut writer = session
            .writer
            .lock()
            .map_err(|_| "writer mutex poisoned".to_string())?;
        writer
            .write_all(data.as_bytes())
            .map_err(|e| format!("write failed: {e}"))?;
        writer.flush().ok();
        Ok(())
    } else {
        Err("terminal not spawned".into())
    }
}

#[tauri::command]
pub fn resize_terminal(
    cols: u16,
    rows: u16,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let guard = state
        .pty
        .lock()
        .map_err(|_| "pty mutex poisoned".to_string())?;
    if let Some(session) = guard.as_ref() {
        let size = PtySize {
            cols,
            rows,
            pixel_width: 0,
            pixel_height: 0,
        };
        if let Ok(master) = session.master.lock() {
            master
                .resize(size)
                .map_err(|e| format!("resize failed: {e}"))?;
        }
        return Ok(());
    }
    Err("terminal not spawned".into())
}
