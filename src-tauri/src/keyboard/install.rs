//! Keyboard layout installation and uninstallation.
//!
//! On Windows the `install_layout` function:
//!   1. Resolves the pre-compiled DLL from the bundled `kbd_dlls/` resources.
//!   2. Copies it to `%SystemRoot%\System32` (and `SysWOW64` on 64-bit Windows).
//!   3. Creates the required registry entries via an elevated PowerShell script.
//!
//! The `uninstall_layout` function removes the registry entries and the DLL
//! files.  No external tools (kbdutool, MSKLC, …) are required at runtime.
use super::Layout;
use std::path::PathBuf;

/// Return the directory where MagicWindows stores temporary working files.
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

// ── Windows implementation ──────────────────────────────────────────────────

#[cfg(target_os = "windows")]
pub fn install_layout(layout: &Layout, app: &tauri::AppHandle) -> Result<(), String> {
    use std::fs;

    // ── 1. Locate the bundled pre-compiled DLL ──────────────────────────────
    let dll_src = resolve_bundled_dll(layout, app)?;

    let install_dir = get_install_dir();
    fs::create_dir_all(&install_dir)
        .map_err(|e| format!("Failed to create install dir: {e}"))?;

    // ── 2. Build the elevated PowerShell install script ─────────────────────
    //
    // The script performs three privileged operations:
    //   a. Copies the DLL to System32 (and SysWOW64 on 64-bit Windows).
    //   b. Computes a unique registry key ID.
    //   c. Creates the registry entries so Windows can discover the layout.
    //
    // We pass the DLL source path and layout metadata as parameters so the
    // script is fully general.
    let layout_name = layout
        .name
        .get("en")
        .map(|s| s.as_str())
        .unwrap_or(&layout.id);

    let ps_script = format!(
        r#"
$ErrorActionPreference = 'Stop'

# ── Privilege check ────────────────────────────────────────────────────────
$principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {{
    throw "MagicWindows must be run as Administrator to install keyboard layouts."
}}

$DllPath   = '{dll_src}'
$DllName   = '{dll_name}'
$LocaleId  = '{locale_id}'
$LayoutName = '{layout_name}'

# ── Validate DLL exists ────────────────────────────────────────────────────
if (-not (Test-Path -LiteralPath $DllPath)) {{
    throw "Bundled DLL not found at: $DllPath"
}}

# ── Generate unique registry key ID ───────────────────────────────────────
$suffix       = $LocaleId.Substring(4, 4)
$kbLayoutsRoot = 'HKLM:\SYSTEM\CurrentControlSet\Control\Keyboard Layouts'
$prefix = 1
do {{
    $layoutId = 'a{{0:x3}}{{1}}' -f $prefix, $suffix
    $regPath  = Join-Path $kbLayoutsRoot $layoutId
    $prefix++
}} while (Test-Path -LiteralPath $regPath)

# Derive unique 4-digit hex Layout Id value
$existingIds = @()
Get-ChildItem -Path $kbLayoutsRoot -ErrorAction SilentlyContinue | ForEach-Object {{
    $val = (Get-ItemProperty -LiteralPath $_.PSPath -Name 'Layout Id' -ErrorAction SilentlyContinue).'Layout Id'
    if ($val) {{ $existingIds += $val }}
}}
$layoutNumber = 1
do {{
    $layoutIdHex = '{{0:x4}}' -f $layoutNumber
    $layoutNumber++
}} while ($existingIds -contains $layoutIdHex)

Write-Host "Registry key : $layoutId"
Write-Host "Layout Id    : $layoutIdHex"

# ── Copy DLL to system directories ────────────────────────────────────────
$dllFileName = "$DllName.dll"
$sys32       = Join-Path $env:SystemRoot 'System32'
$destSys32   = Join-Path $sys32 $dllFileName
Write-Host "Copying to $destSys32 ..."
Copy-Item -LiteralPath $DllPath -Destination $destSys32 -Force

$wow64 = Join-Path $env:SystemRoot 'SysWOW64'
if (Test-Path -LiteralPath $wow64) {{
    $destWow64 = Join-Path $wow64 $dllFileName
    Write-Host "Copying to $destWow64 ..."
    Copy-Item -LiteralPath $DllPath -Destination $destWow64 -Force
}}

# ── Create registry entries ────────────────────────────────────────────────
Write-Host "Creating registry entries at $regPath ..."
New-Item -Path $regPath -Force | Out-Null
New-ItemProperty -LiteralPath $regPath -Name 'Layout File' -Value $dllFileName   -PropertyType String -Force | Out-Null
New-ItemProperty -LiteralPath $regPath -Name 'Layout Text' -Value $LayoutName    -PropertyType String -Force | Out-Null
New-ItemProperty -LiteralPath $regPath -Name 'Layout Id'   -Value $layoutIdHex   -PropertyType String -Force | Out-Null

Write-Host 'Keyboard layout installed successfully.'
"#,
        dll_src     = dll_src.display(),
        dll_name    = layout.dll_name,
        locale_id   = layout.locale_id,
        layout_name = layout_name.replace('\'', "\\'"),
    );

    run_elevated_ps(&install_dir, "install", &ps_script)?;
    log::info!("Layout {} installed successfully", layout.id);
    Ok(())
}

/// Resolve the path to the pre-compiled keyboard layout DLL bundled with the
/// application.
///
/// Tauri bundles resources under `<resource_dir>/kbd_dlls/<name>.dll`.
#[cfg(target_os = "windows")]
fn resolve_bundled_dll(
    layout: &Layout,
    app: &tauri::AppHandle,
) -> Result<PathBuf, String> {
    use tauri::Manager;

    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Cannot resolve resource dir: {e}"))?;

    let dll_path = resource_dir
        .join("kbd_dlls")
        .join(format!("{}.dll", layout.dll_name));

    if !dll_path.exists() {
        return Err(format!(
            "Bundled keyboard DLL not found: {}. \
             This is a build issue – please reinstall MagicWindows.",
            dll_path.display()
        ));
    }

    Ok(dll_path)
}

#[cfg(target_os = "windows")]
pub fn uninstall_layout(layout: &Layout) -> Result<(), String> {
    use std::fs;

    let install_dir = get_install_dir();
    fs::create_dir_all(&install_dir)
        .map_err(|e| format!("Failed to create install dir: {e}"))?;

    let ps_script = format!(
        r#"
$ErrorActionPreference = 'Stop'

$principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {{
    throw "MagicWindows must be run as Administrator to uninstall keyboard layouts."
}}

$DllName = '{dll}'

# ── Remove registry entries ────────────────────────────────────────────────
$regPath = 'HKLM:\SYSTEM\CurrentControlSet\Control\Keyboard Layouts'
$entries = Get-ChildItem $regPath | Where-Object {{
    (Get-ItemProperty $_.PSPath).'Layout File' -eq "$DllName.dll"
}}
foreach ($entry in $entries) {{
    Remove-Item $entry.PSPath -Force
}}

# ── Remove DLL from System32 ───────────────────────────────────────────────
$sys32Dll = "$env:SystemRoot\System32\$DllName.dll"
if (Test-Path $sys32Dll) {{ Remove-Item $sys32Dll -Force }}

# ── Remove DLL from SysWOW64 ──────────────────────────────────────────────
$wow64Dll = "$env:SystemRoot\SysWOW64\$DllName.dll"
if (Test-Path $wow64Dll) {{ Remove-Item $wow64Dll -Force }}

Write-Host 'Keyboard layout uninstalled successfully.'
"#,
        dll = layout.dll_name,
    );

    run_elevated_ps(&install_dir, "uninstall", &ps_script)?;
    log::info!("Layout {} uninstalled successfully", layout.id);
    Ok(())
}

/// Write `ps_script` to a `.ps1` file and run it in an elevated PowerShell
/// process.  Blocks until the elevated child exits, then returns success or
/// a descriptive error.
#[cfg(target_os = "windows")]
fn run_elevated_ps(
    work_dir: &std::path::Path,
    label: &str,
    ps_script: &str,
) -> Result<(), String> {
    use std::fs;
    use std::process::Command;

    let ps_path     = work_dir.join(format!("{label}.ps1"));
    let stdout_path = work_dir.join(format!("{label}_stdout.txt"));
    let stderr_path = work_dir.join(format!("{label}_stderr.txt"));

    fs::write(&ps_path, ps_script)
        .map_err(|e| format!("Failed to write {label} script: {e}"))?;

    // Spawn the elevated child and capture its streams via Start-Process.
    let launcher = format!(
        r#"
$proc = Start-Process powershell `
    -ArgumentList @('-ExecutionPolicy','Bypass','-NoProfile','-File','{ps}') `
    -Verb RunAs `
    -Wait `
    -PassThru `
    -RedirectStandardOutput '{stdout}' `
    -RedirectStandardError  '{stderr}'
exit $proc.ExitCode
"#,
        ps     = ps_path.display(),
        stdout = stdout_path.display(),
        stderr = stderr_path.display(),
    );

    let output = Command::new("powershell")
        .args(["-ExecutionPolicy", "Bypass", "-NoProfile", "-Command", &launcher])
        .output()
        .map_err(|e| format!("Failed to run PowerShell launcher: {e}"))?;

    if output.status.success() {
        return Ok(());
    }

    let child_stderr   = fs::read_to_string(&stderr_path).unwrap_or_default();
    let child_stdout   = fs::read_to_string(&stdout_path).unwrap_or_default();
    let launcher_stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    let detail = [child_stderr.trim(), child_stdout.trim(), launcher_stderr.trim()]
        .iter()
        .filter(|s| !s.is_empty())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");

    Err(format!("Operation failed: {detail}"))
}

// ── Non-Windows stubs ───────────────────────────────────────────────────────

#[cfg(not(target_os = "windows"))]
pub fn install_layout(_layout: &Layout, _app: &tauri::AppHandle) -> Result<(), String> {
    Err("Installation requires Windows.".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn uninstall_layout(_layout: &Layout) -> Result<(), String> {
    Err("Uninstallation requires Windows.".to_string())
}
