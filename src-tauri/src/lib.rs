use ignore::WalkBuilder;
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::Emitter;

const MENU_OPEN_FOLDER: &str = "open-folder";
const MENU_QUIT: &str = "quit";

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn list_files(path: &str) -> Result<Vec<String>, String> {
    let base = std::path::PathBuf::from(path);
    if !base.is_dir() {
        return Err("Not a directory".into());
    }

    let walker = WalkBuilder::new(&base)
        .hidden(true)
        .ignore(true)
        .git_ignore(true)
        .git_exclude(true)
        .parents(true)
        .build();

    let mut files = Vec::new();
    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.components().any(|c| c.as_os_str() == ".git") {
            continue;
        }
        if let Ok(rel) = path.strip_prefix(&base) {
            if let Some(rel_str) = rel.to_str() {
                files.push(rel_str.replace('\\', "/"));
            }
        }
    }

    Ok(files)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let file_open = MenuItemBuilder::new("Open Folder")
                .id(MENU_OPEN_FOLDER)
                .accelerator("CmdOrCtrl+O")
                .build(app)?;

            let quit = MenuItemBuilder::new("Quit Visor")
                .id(MENU_QUIT)
                .accelerator("CmdOrCtrl+Q")
                .build(app)?;

            let app_menu = SubmenuBuilder::new(app, "Visor")
                .items(&[&quit])
                .build()?;

            let file_menu = SubmenuBuilder::new(app, "File")
                .items(&[&file_open])
                .build()?;

            let menu = MenuBuilder::new(app)
                .items(&[&app_menu, &file_menu])
                .build()?;

            app.set_menu(menu)?;

            Ok(())
        })
        .on_menu_event(|app, event| {
            if event.id() == MENU_OPEN_FOLDER {
                let _ = app.emit("menu://open-folder", ());
            }
            if event.id() == MENU_QUIT {
                let _ = app.exit(0);
            }
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet, list_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, write};
    use tempfile::TempDir;

    #[test]
    fn list_files_respects_gitignore() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        create_dir_all(root.join("src")).unwrap();
        create_dir_all(root.join("build")).unwrap();
        create_dir_all(root.join(".git")).unwrap();

        write(root.join("src/main.rs"), "// main").unwrap();
        write(root.join("src/ignore.me"), "ignored").unwrap();
        write(root.join("build/output.js"), "bundle").unwrap();
        write(root.join(".git/config"), "config").unwrap();
        write(
            root.join(".gitignore"),
            "build/\nsrc/ignore.me\n.DS_Store\n",
        )
        .unwrap();

        let files = list_files(root.to_str().unwrap()).unwrap();
        assert!(files.contains(&"src/main.rs".to_string()));
        assert!(!files.iter().any(|f| f.contains("build/output.js")));
        assert!(!files.iter().any(|f| f.contains("src/ignore.me")));
        assert!(!files.iter().any(|f| f.contains(".git/")));
    }
}
