use std::fs;

#[tauri::command]
pub fn list_directory(path: Option<String>) -> Result<Vec<String>, String> {
    let path = path.unwrap_or_else(|| ".".to_string());
    match fs::read_dir(path) {
        Ok(entries) => {
            let list = entries
                .filter_map(|entry| entry.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            Ok(list)
        }
        Err(err) => Err(err.to_string()),
    }
}