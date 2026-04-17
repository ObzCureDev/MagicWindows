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

    // The elevated PS writes markers to this file, NOT stdout — `Start-Process -Verb RunAs`
    // with -RedirectStandardOutput is unreliable, and `Write-Host` doesn't go to stdout
    // anyway (it writes to the Information stream in PS 5.1+). Reading a file is robust.
    let markers_path = install_dir.join("install_markers.txt");
    // Pre-delete any stale markers from a previous run so we never read stale data.
    let _ = std::fs::remove_file(&markers_path);

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

# ── Find or generate the registry key ID ──────────────────────────────────
# If a previous registration for this exact DLL exists, REUSE its KLID instead of
# creating a duplicate entry. Without this, every re-install would leave orphan
# KLIDs in the registry pointing at the same DLL.
$suffix       = $LocaleId.Substring(4, 4)
$kbLayoutsRoot = 'HKLM:\SYSTEM\CurrentControlSet\Control\Keyboard Layouts'
$expectedDll  = "$DllName.dll"
$layoutId = $null
$regPath  = $null

Get-ChildItem -Path $kbLayoutsRoot -ErrorAction SilentlyContinue | ForEach-Object {{
    if ($layoutId) {{ return }}
    $existingFile = (Get-ItemProperty -LiteralPath $_.PSPath -Name 'Layout File' -ErrorAction SilentlyContinue).'Layout File'
    if ($existingFile -eq $expectedDll) {{
        $layoutId = $_.PSChildName
        $regPath  = $_.PSPath
        Write-Host "Reusing existing KLID $layoutId for $expectedDll"
    }}
}}

if (-not $layoutId) {{
    $prefix = 1
    do {{
        $layoutId = 'a{{0:x3}}{{1}}' -f $prefix, $suffix
        $regPath  = Join-Path $kbLayoutsRoot $layoutId
        $prefix++
    }} while (Test-Path -LiteralPath $regPath)
}}

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

# Emit machine-parseable markers to a known FILE so the parent Rust process can pick up
# the generated KLID and language tag, then add the layout to the current user's input
# methods in a separate (non-elevated) step. We write to a file (not stdout) because
# Start-Process -Verb RunAs can't reliably redirect stdout from the elevated child.
$langIdHex = $LocaleId.Substring(4, 4)
$langTag   = [System.Globalization.CultureInfo]::new([int]("0x$langIdHex")).Name
$markersPath = '{markers_path}'
@(
    "MAGICWINDOWS_KLID=$layoutId",
    "MAGICWINDOWS_LANGID=$langIdHex",
    "MAGICWINDOWS_LANGTAG=$langTag"
) | Out-File -FilePath $markersPath -Encoding ASCII -Force
Write-Host "Markers written to $markersPath"

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
        dll_src       = dll_src.display(),
        dll_name      = layout.dll_name,
        locale_id     = layout.locale_id,
        layout_name   = layout_name.replace('\'', "\\'"),
        markers_path  = markers_path.display(),
    );

    let _ = run_elevated_ps(&install_dir, "install", &ps_script)?;
    log::info!("Layout {} installed successfully", layout.id);

    // ── 3. Activate: add the new KLID to the user's input methods ──────────
    // Read markers from the FILE the elevated PS wrote (not stdout — see comment above).
    // Then run a non-elevated PowerShell in the original user's context to add the KLID
    // to their language list AND to HKCU\Keyboard Layout\Preload (belt-and-suspenders).
    match std::fs::read_to_string(&markers_path) {
        Ok(content) => {
            if let Some((klid, lang_tag)) = parse_install_markers(&content) {
                match activate_for_user(&install_dir, &klid, &lang_tag) {
                    Ok(()) => log::info!("Layout {} activated for current user (KLID {klid}, tag {lang_tag})", layout.id),
                    Err(e) => log::warn!("Layout installed but auto-activation failed: {e}"),
                }
            } else {
                log::warn!("Markers file present but no KLID/LANGTAG parsed from {}", markers_path.display());
            }
        }
        Err(e) => {
            log::warn!("Could not read markers file {}: {e}; skipping auto-activation", markers_path.display());
        }
    }
    Ok(())
}

/// Pulls `MAGICWINDOWS_KLID=...` and `MAGICWINDOWS_LANGTAG=...` lines out of the
/// elevated install script's stdout. Returns (klid, langTag) if both are found.
#[cfg(target_os = "windows")]
fn parse_install_markers(stdout: &str) -> Option<(String, String)> {
    let mut klid: Option<String> = None;
    let mut tag: Option<String> = None;
    for line in stdout.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("MAGICWINDOWS_KLID=") {
            klid = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("MAGICWINDOWS_LANGTAG=") {
            tag = Some(v.trim().to_string());
        }
    }
    match (klid, tag) {
        (Some(k), Some(t)) if !k.is_empty() && !t.is_empty() => Some((k, t)),
        _ => None,
    }
}

/// Adds the freshly-installed KLID to the current user's input methods so the
/// keyboard layout becomes selectable from the language bar without a manual
/// trip to Windows Settings. Runs un-elevated (HKCU is per-user).
#[cfg(target_os = "windows")]
fn activate_for_user(work_dir: &std::path::Path, klid: &str, lang_tag: &str) -> Result<(), String> {
    use std::fs;
    use std::process::Command;

    let ps_path     = work_dir.join("activate.ps1");
    let stdout_path = work_dir.join("activate_stdout.txt");
    let stderr_path = work_dir.join("activate_stderr.txt");

    // KLID is the 8-char registry key name (e.g. a001040c). InputMethodTip format
    // is "<langid_hex>:<klid>" — we derive langid from the last 4 chars of the KLID
    // (which itself encodes the locale suffix per the install script).
    let lang_id_hex = &klid[klid.len().saturating_sub(4)..];
    let tip = format!("{lang_id_hex}:{klid}");

    let script = format!(
        r#"
$ErrorActionPreference = 'Continue'
$tip      = '{tip}'
$langTag  = '{lang_tag}'
$klid     = '{klid}'

Write-Host "[activate] tip=$tip langTag=$langTag klid=$klid"

# ── Path A: modern WinUserLanguageList API ────────────────────────────────
try {{
    $list = Get-WinUserLanguageList
    $lang = $list | Where-Object {{ $_.LanguageTag -eq $langTag }}
    if (-not $lang) {{
        Write-Host "[activate] Language $langTag not in list — adding it"
        $list.Add($langTag)
        $lang = $list | Where-Object {{ $_.LanguageTag -eq $langTag }}
    }}
    if ($lang.InputMethodTips -notcontains $tip) {{
        $lang.InputMethodTips.Add($tip)
        Set-WinUserLanguageList $list -Force
        Write-Host "[activate] Set-WinUserLanguageList: added $tip to $langTag"
    }} else {{
        Write-Host "[activate] Set-WinUserLanguageList: $tip already present"
    }}
}} catch {{
    Write-Host "[activate] Set-WinUserLanguageList FAILED: $_"
}}

# ── Path B: direct HKCU\Keyboard Layout\Preload write ─────────────────────
# More reliable than the cmdlet on some Windows builds; survives logoff and
# is the legacy "this layout exists for me" registry entry.
try {{
    $preload = 'HKCU:\Keyboard Layout\Preload'
    if (-not (Test-Path -LiteralPath $preload)) {{
        New-Item -Path $preload -Force | Out-Null
    }}
    $existing = (Get-Item -LiteralPath $preload).GetValueNames() |
        Where-Object {{ $_ -match '^\d+$' }}
    $alreadyPreloaded = $false
    foreach ($name in $existing) {{
        $val = (Get-ItemProperty -LiteralPath $preload -Name $name).$name
        if ($val -eq $klid) {{ $alreadyPreloaded = $true; break }}
    }}
    if (-not $alreadyPreloaded) {{
        $maxIdx = 0
        foreach ($name in $existing) {{
            $i = [int]$name
            if ($i -gt $maxIdx) {{ $maxIdx = $i }}
        }}
        $nextIdx = ($maxIdx + 1).ToString()
        Set-ItemProperty -LiteralPath $preload -Name $nextIdx -Value $klid -Type String
        Write-Host "[activate] HKCU Preload: added $klid as index $nextIdx"
    }} else {{
        Write-Host "[activate] HKCU Preload: $klid already present"
    }}
}} catch {{
    Write-Host "[activate] HKCU Preload write FAILED: $_"
}}

Write-Host "[activate] done"
"#
    );

    fs::write(&ps_path, &script)
        .map_err(|e| format!("Failed to write activate script: {e}"))?;

    let output = Command::new("powershell")
        .args([
            "-ExecutionPolicy", "Bypass",
            "-NoProfile",
            "-File", &ps_path.to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("Failed to run activation PowerShell: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let _ = fs::write(&stdout_path, &stdout);
    let _ = fs::write(&stderr_path, &stderr);

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("Activation PS failed: {}", stderr.trim()))
    }
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
/// process.  Blocks until the elevated child exits, then returns the captured
/// stdout on success or a descriptive error on failure.
#[cfg(target_os = "windows")]
fn run_elevated_ps(
    work_dir: &std::path::Path,
    label: &str,
    ps_script: &str,
) -> Result<String, String> {
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
        let child_stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
        return Ok(child_stdout);
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

/// Public re-export of the elevated PowerShell runner so other modules in the
/// keyboard crate (e.g. modifiers.rs) can use the same UAC + capture logic.
#[cfg(target_os = "windows")]
pub fn run_elevated_ps_for_modifiers(
    work_dir: &std::path::Path,
    label: &str,
    ps_script: &str,
) -> Result<String, String> {
    run_elevated_ps(work_dir, label, ps_script)
}
