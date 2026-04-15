use super::Layout;
use std::path::PathBuf;

/// Return the directory where MagicWindows stores installed layouts.
pub fn get_install_dir() -> PathBuf {
    let mut dir = dirs_next_or_temp();
    dir.push("MagicWindows");
    dir.push("layouts");
    dir
}

/// Best-effort local-data directory; falls back to the system temp dir.
fn dirs_next_or_temp() -> PathBuf {
    std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
}

// ── Windows implementation ──────────────────────────────────────────────

#[cfg(target_os = "windows")]
pub fn install_layout(layout: &Layout, klc_content: &str) -> Result<(), String> {
    use std::fs;
    use std::process::Command;

    let install_dir = get_install_dir();
    fs::create_dir_all(&install_dir).map_err(|e| format!("Failed to create install dir: {e}"))?;

    // Write the .klc file
    let klc_path = install_dir.join(format!("{}.klc", layout.dll_name));
    fs::write(&klc_path, klc_content)
        .map_err(|e| format!("Failed to write KLC file: {e}"))?;

    // PowerShell install script
    let ps_script = format!(
        r#"
$ErrorActionPreference = 'Stop'

$klcPath = '{klc}'
$dllName = '{dll}'

# Use msklc's kbdutool or the built-in Windows API to compile and install.
# For now we rely on the KLC file being loadable by the system.
# The recommended approach is to ship a small C helper or use the
# Microsoft Keyboard Layout Creator (MSKLC) command-line tool.

# Attempt to run kbdutool if available on the PATH
$kbdutool = Get-Command kbdutool -ErrorAction SilentlyContinue
if ($kbdutool) {{
    & kbdutool -u -s "$klcPath"
    if ($LASTEXITCODE -ne 0) {{
        throw "kbdutool failed with exit code $LASTEXITCODE"
    }}
}} else {{
    throw "kbdutool not found. Please install Microsoft Keyboard Layout Creator (MSKLC)."
}}
"#,
        klc = klc_path.display(),
        dll = layout.dll_name,
    );

    let ps_path = install_dir.join("install.ps1");
    fs::write(&ps_path, &ps_script)
        .map_err(|e| format!("Failed to write install script: {e}"))?;

    let output = Command::new("powershell")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-NoProfile",
            "-File",
            &ps_path.to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {e}"))?;

    if output.status.success() {
        log::info!("Layout {} installed successfully", layout.id);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Installation failed: {stderr}"))
    }
}

#[cfg(target_os = "windows")]
pub fn uninstall_layout(layout: &Layout) -> Result<(), String> {
    use std::process::Command;

    let ps_script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$dllName = '{dll}'

# Remove the keyboard layout DLL and registry entries.
$regPath = 'HKLM:\SYSTEM\CurrentControlSet\Control\Keyboard Layouts'
$entries = Get-ChildItem $regPath | Where-Object {{
    (Get-ItemProperty $_.PSPath).'Layout File' -eq "$dllName.dll"
}}
foreach ($entry in $entries) {{
    Remove-Item $entry.PSPath -Force
}}

# Remove the DLL from System32
$dllPath = "$env:SystemRoot\System32\$dllName.dll"
if (Test-Path $dllPath) {{
    Remove-Item $dllPath -Force
}}

# Clean up local files
$installDir = '{install_dir}'
if (Test-Path $installDir) {{
    Remove-Item "$installDir\$dllName.*" -Force -ErrorAction SilentlyContinue
}}
"#,
        dll = layout.dll_name,
        install_dir = get_install_dir().display(),
    );

    let output = Command::new("powershell")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-NoProfile",
            "-Command",
            &ps_script,
        ])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {e}"))?;

    if output.status.success() {
        log::info!("Layout {} uninstalled successfully", layout.id);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Uninstallation failed: {stderr}"))
    }
}

// ── Non-Windows stubs ───────────────────────────────────────────────────

#[cfg(not(target_os = "windows"))]
pub fn install_layout(_layout: &Layout, _klc_content: &str) -> Result<(), String> {
    Err("Installation requires Windows.".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn uninstall_layout(_layout: &Layout) -> Result<(), String> {
    Err("Uninstallation requires Windows.".to_string())
}
