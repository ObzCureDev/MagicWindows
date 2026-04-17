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

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn write_scancode_map(toggles: ModifierToggles) -> Result<(), String> {
    use super::scancode_map::build_scancode_map;
    use crate::keyboard::install::get_install_dir;
    use std::fs;

    let bytes = build_scancode_map(&toggles);
    let install_dir = get_install_dir();
    fs::create_dir_all(&install_dir)
        .map_err(|e| format!("Failed to create install dir: {e}"))?;

    // Empty bytes = user wants no mappings. Delegate to clear_scancode_map.
    if bytes.is_empty() {
        return delete_scancode_map_value(&install_dir);
    }

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {{
    throw "Administrator privileges are required to modify the keyboard layout registry."
}}

$path  = 'HKLM:\System\CurrentControlSet\Control\Keyboard Layout'
$name  = 'Scancode Map'
$bytes = [Convert]::FromBase64String('{b64}')

# Ensure the key exists (it does on every Windows install, but defensive).
if (-not (Test-Path -LiteralPath $path)) {{
    throw "Registry path not found: $path"
}}

Set-ItemProperty -LiteralPath $path -Name $name -Value $bytes -Type Binary -Force
Write-Host "Scancode Map written ($($bytes.Length) bytes)."
"#
    );

    super::install::run_elevated_ps_for_modifiers(&install_dir, "scancode_write", &script)?;
    Ok(())
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn clear_scancode_map() -> Result<(), String> {
    use crate::keyboard::install::get_install_dir;
    use std::fs;
    let install_dir = get_install_dir();
    fs::create_dir_all(&install_dir).map_err(|e| format!("Failed to create install dir: {e}"))?;
    delete_scancode_map_value(&install_dir)
}

#[cfg(target_os = "windows")]
fn delete_scancode_map_value(install_dir: &std::path::Path) -> Result<(), String> {
    let script = r#"
$ErrorActionPreference = 'Stop'
$principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw "Administrator privileges are required to modify the keyboard layout registry."
}

$path = 'HKLM:\System\CurrentControlSet\Control\Keyboard Layout'
Remove-ItemProperty -LiteralPath $path -Name 'Scancode Map' -ErrorAction SilentlyContinue
Write-Host "Scancode Map cleared (or already absent)."
"#;
    super::install::run_elevated_ps_for_modifiers(install_dir, "scancode_clear", script)?;
    Ok(())
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
