//! Tauri command layer for the Scancode Map registry value.
//!
//! Reading is unprivileged (HKLM is world-readable). Writing requires elevation
//! and goes through the same elevated-PowerShell helper used by the layout
//! installer in install.rs.

use super::scancode_map::{
    derive_state, parse_scancode_map, ModifierState, ModifierToggles,
};

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn read_scancode_map() -> Result<ModifierState, String> {
    use std::process::Command;

    // Read the REG_BINARY value as base64 so it survives the PowerShell -> stdout pipe cleanly.
    let script = r#"
$bytes = (Get-ItemProperty -Path 'HKLM:\System\CurrentControlSet\Control\Keyboard Layout' `
                            -Name 'Scancode Map' -ErrorAction SilentlyContinue).'Scancode Map'
if ($bytes) {
    [Convert]::ToBase64String($bytes)
} else {
    'NONE'
}
"#;

    let output = Command::new("powershell")
        .args(["-ExecutionPolicy", "Bypass", "-NoProfile", "-Command", script])
        .output()
        .map_err(|e| format!("Failed to invoke powershell: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "powershell read failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let bytes: Vec<u8> = if stdout == "NONE" || stdout.is_empty() {
        Vec::new()
    } else {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(&stdout)
            .map_err(|e| format!("Bad base64 from PowerShell: {e}"))?
    };

    let pairs = parse_scancode_map(&bytes)?;
    Ok(derive_state(&pairs))
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn read_scancode_map() -> Result<ModifierState, String> {
    Err("Modifier remapping requires Windows.".to_string())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn write_scancode_map(_toggles: ModifierToggles) -> Result<(), String> {
    Err("Modifier remapping requires Windows.".to_string())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn clear_scancode_map() -> Result<(), String> {
    Err("Modifier remapping requires Windows.".to_string())
}
