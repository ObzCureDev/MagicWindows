mod keyboard;

use keyboard::{DetectionResult, Layout, LayoutMeta};
use std::path::PathBuf;
use tauri::Manager;

/// Resolve the path to the bundled `layouts/` resource directory.
fn layouts_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .resource_dir()
        .map(|p| p.join("layouts"))
        .map_err(|e| format!("Cannot resolve resource dir: {e}"))
}

/// Read and parse every `*.json` layout file from the bundled resources,
/// skipping `schema.json`.
fn load_all_layouts(app: &tauri::AppHandle) -> Result<Vec<Layout>, String> {
    let dir = layouts_dir(app)?;
    let entries = std::fs::read_dir(&dir)
        .map_err(|e| format!("Cannot read layouts directory {}: {e}", dir.display()))?;

    let mut layouts = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Directory entry error: {e}"))?;
        let path = entry.path();

        // Only process .json files, skip schema.json
        let is_json = path
            .extension()
            .map(|ext| ext == "json")
            .unwrap_or(false);
        let is_schema = path
            .file_name()
            .map(|n| n == "schema.json")
            .unwrap_or(false);

        if is_json && !is_schema {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
            let layout: Layout = serde_json::from_str(&content)
                .map_err(|e| format!("Cannot parse {}: {e}", path.display()))?;
            layouts.push(layout);
        }
    }

    // Sort by ID for stable ordering.
    layouts.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(layouts)
}

/// Load a single layout by its `id` field.
fn load_layout(app: &tauri::AppHandle, id: &str) -> Result<Layout, String> {
    let dir = layouts_dir(app)?;
    // Convention: the file is named `{id}.json`.
    let path = dir.join(format!("{id}.json"));
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read layout '{id}': {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Cannot parse layout '{id}': {e}"))
}

// ── Tauri commands ──────────────────────────────────────────────────────

/// List metadata for all available keyboard layouts.
#[tauri::command]
fn list_layouts(app: tauri::AppHandle) -> Result<Vec<LayoutMeta>, String> {
    let layouts = load_all_layouts(&app)?;
    Ok(layouts.into_iter().map(|l| l.meta()).collect())
}

/// Return the full layout data for a specific layout.
#[tauri::command]
fn get_layout(app: tauri::AppHandle, id: String) -> Result<Layout, String> {
    load_layout(&app, &id)
}

/// Return detection keys for the specified layout IDs (used by the
/// auto-detection UI).  Keys are de-duplicated by `eventCode`.
#[tauri::command]
fn get_detection_keys(
    app: tauri::AppHandle,
    layout_ids: Vec<String>,
) -> Result<Vec<keyboard::DetectionKey>, String> {
    let mut seen = std::collections::HashSet::new();
    let mut keys = Vec::new();

    for id in &layout_ids {
        let layout = load_layout(&app, id)?;
        for dk in layout.detection_keys {
            if seen.insert(dk.event_code.clone()) {
                keys.push(dk);
            }
        }
    }

    Ok(keys)
}

/// Accept key-press results from the auto-detection UI and return the best
/// matching layout ID, or `null` if the results are ambiguous.
#[tauri::command]
fn match_detection(
    app: tauri::AppHandle,
    results: Vec<DetectionResult>,
) -> Result<Option<String>, String> {
    let layouts = load_all_layouts(&app)?;
    Ok(keyboard::detect::find_best_match(&layouts, &results))
}

/// Generate a .klc file for the given layout and return its content.
#[tauri::command]
fn generate_klc(app: tauri::AppHandle, id: String) -> Result<String, String> {
    let layout = load_layout(&app, &id)?;
    Ok(keyboard::klc::generate_klc(&layout))
}

/// Generate and install a keyboard layout on the system.
#[tauri::command]
fn install_layout(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let layout = load_layout(&app, &id)?;
    let klc_content = keyboard::klc::generate_klc(&layout);
    keyboard::install::install_layout(&layout, &klc_content)
}

/// Uninstall a previously installed keyboard layout.
#[tauri::command]
fn uninstall_layout(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let layout = load_layout(&app, &id)?;
    keyboard::install::uninstall_layout(&layout)
}

// ── Tauri entry point ───────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            list_layouts,
            get_layout,
            get_detection_keys,
            match_detection,
            generate_klc,
            install_layout,
            uninstall_layout,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MagicWindows");
}
