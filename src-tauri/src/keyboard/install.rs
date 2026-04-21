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
pub fn install_layout(layout: &Layout, app: &tauri::AppHandle) -> Result<u64, String> {
    use std::fs;
    use std::time::Instant;

    let started = Instant::now();

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

    // The elevated PS writes markers to HKLM (not a file, not stdout). File writes
    // from elevated -> user's LOCALAPPDATA fail silently in some configurations, and
    // stdout capture via Start-Process -Verb RunAs is unreliable. The registry is the
    // one place admin can always reliably write and the unprivileged parent can read.
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

# ── Wipe any existing registrations of this DLL ───────────────────────────
# Previous installs may have registered one or more KLIDs against the same
# DLL name (e.g. with the old "Apple French (AZERTY)" Layout Text, or a
# broken 64-bit-in-SysWOW64 install). To make re-install behave as a clean
# replace — and to avoid orphan KLIDs piling up — we delete all existing
# Keyboard Layouts subkeys whose 'Layout File' matches this DLL, and stash
# the deleted KLIDs in HKLM markers so the user-side activate step can also
# strip them from HKCU\Keyboard Layout\Preload.
$suffix        = $LocaleId.Substring(4, 4)
$kbLayoutsRoot = 'HKLM:\SYSTEM\CurrentControlSet\Control\Keyboard Layouts'
$expectedDll   = "$DllName.dll"
$staleKlids    = @()

Get-ChildItem -Path $kbLayoutsRoot -ErrorAction SilentlyContinue | ForEach-Object {{
    $existingFile = (Get-ItemProperty -LiteralPath $_.PSPath -Name 'Layout File' -ErrorAction SilentlyContinue).'Layout File'
    if ($existingFile -eq $expectedDll) {{
        $oldKlid = $_.PSChildName
        $staleKlids += $oldKlid
        Write-Host "Removing stale registration: $oldKlid"
        Remove-Item -LiteralPath $_.PSPath -Recurse -Force -ErrorAction SilentlyContinue
    }}
}}

# ── Generate a fresh KLID ─────────────────────────────────────────────────
$layoutId = $null
$regPath  = $null
$prefix   = 1
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

# Emit machine-parseable markers to HKLM so the parent Rust process can pick up the
# generated KLID and language tag, then add the layout to the current user's input
# methods in a separate (non-elevated) step. Registry is reliable from elevated PS;
# stdout capture and cross-user file writes are not.
$langIdHex = $LocaleId.Substring(4, 4)
$langTag   = [System.Globalization.CultureInfo]::new([int]("0x$langIdHex")).Name
$markerKey = 'HKLM:\SOFTWARE\MagicWindows'
if (-not (Test-Path -LiteralPath $markerKey)) {{
    New-Item -Path $markerKey -Force | Out-Null
}}
Set-ItemProperty -LiteralPath $markerKey -Name 'LastInstalledKLID'    -Value $layoutId  -Force
Set-ItemProperty -LiteralPath $markerKey -Name 'LastInstalledLANGID'  -Value $langIdHex -Force
Set-ItemProperty -LiteralPath $markerKey -Name 'LastInstalledLANGTAG' -Value $langTag   -Force
$staleJoined = ($staleKlids -join ',')
Set-ItemProperty -LiteralPath $markerKey -Name 'StaleKLIDs'           -Value $staleJoined -Force
Write-Host "Markers written to $markerKey (stale KLIDs: $staleJoined)"

# ── Copy DLL to System32 ───────────────────────────────────────────────────
# Only the 64-bit DLL is copied to System32. We deliberately do NOT mirror it
# to SysWOW64 because we currently only build for x86_64. SysWOW64 must hold a
# 32-bit (i686) DLL — a 64-bit DLL there crashes any 32-bit process that loads
# it (Explorer shell extensions, Office IMEs, etc) the moment the layout is
# activated. Better to skip 32-bit support than to ship a guaranteed crash.
# Also actively remove any stale 64-bit copy a previous bad install left in
# SysWOW64.
$dllFileName = "$DllName.dll"
$sys32       = Join-Path $env:SystemRoot 'System32'
$destSys32   = Join-Path $sys32 $dllFileName
Write-Host "Copying to $destSys32 ..."
Copy-Item -LiteralPath $DllPath -Destination $destSys32 -Force

$wow64 = Join-Path $env:SystemRoot 'SysWOW64'
if (Test-Path -LiteralPath $wow64) {{
    $staleWow64 = Join-Path $wow64 $dllFileName
    if (Test-Path -LiteralPath $staleWow64) {{
        Write-Host "Removing stale 64-bit DLL from SysWOW64: $staleWow64"
        Remove-Item -LiteralPath $staleWow64 -Force -ErrorAction SilentlyContinue
    }}
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

    let _ = run_elevated_ps(&install_dir, "install", &ps_script)?;
    log::info!("Layout {} installed successfully", layout.id);

    // ── 3. Activate: read markers from HKLM, then add the KLID to the user's input methods.
    //    Stale KLIDs from the wipe step are also purged from HKCU\Preload + input method tips.
    match read_install_markers_from_registry() {
        Ok((klid, lang_tag, stale_klids)) => {
            match activate_for_user(&install_dir, &klid, &lang_tag, &stale_klids) {
                Ok(()) => log::info!("Layout {} activated for current user (KLID {klid}, tag {lang_tag}; purged stale: {stale_klids:?})", layout.id),
                Err(e) => log::warn!("Layout installed but auto-activation failed: {e}"),
            }
        }
        Err(e) => {
            log::warn!("Could not read install markers from HKLM: {e}; skipping auto-activation");
        }
    }
    let elapsed_ms = started.elapsed().as_millis() as u64;
    log::info!("Layout {} install completed in {elapsed_ms} ms", layout.id);
    Ok(elapsed_ms)
}

/// Read the KLID, language tag, and list of stale KLIDs (purged by the install
/// step) from HKLM:\SOFTWARE\MagicWindows. Returns (klid, langTag, staleKlids).
/// Unprivileged PowerShell read.
#[cfg(target_os = "windows")]
fn read_install_markers_from_registry() -> Result<(String, String, Vec<String>), String> {
    use std::process::Command;
    let script = r#"
$key = 'HKLM:\SOFTWARE\MagicWindows'
if (-not (Test-Path -LiteralPath $key)) { Write-Output 'NONE'; exit 0 }
$klid    = (Get-ItemProperty -LiteralPath $key -Name 'LastInstalledKLID'    -ErrorAction SilentlyContinue).LastInstalledKLID
$langTag = (Get-ItemProperty -LiteralPath $key -Name 'LastInstalledLANGTAG' -ErrorAction SilentlyContinue).LastInstalledLANGTAG
$stale   = (Get-ItemProperty -LiteralPath $key -Name 'StaleKLIDs'           -ErrorAction SilentlyContinue).StaleKLIDs
if (-not $klid -or -not $langTag) { Write-Output 'NONE'; exit 0 }
if ($null -eq $stale) { $stale = '' }
Write-Output "$klid|$langTag|$stale"
"#;
    let out = Command::new("powershell")
        .args(["-ExecutionPolicy", "Bypass", "-NoProfile", "-Command", script])
        .output()
        .map_err(|e| format!("powershell invoke failed: {e}"))?;
    if !out.status.success() {
        return Err(format!("powershell read failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if stdout == "NONE" || stdout.is_empty() {
        return Err("HKLM markers absent".to_string());
    }
    let mut parts = stdout.splitn(3, '|');
    let klid  = parts.next().ok_or("malformed marker output")?.trim().to_string();
    let tag   = parts.next().ok_or("malformed marker output")?.trim().to_string();
    let stale = parts.next().unwrap_or("").trim().to_string();
    if klid.is_empty() || tag.is_empty() {
        return Err("empty KLID or LANGTAG in markers".to_string());
    }
    let stale_klids: Vec<String> = if stale.is_empty() {
        Vec::new()
    } else {
        stale.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    };
    Ok((klid, tag, stale_klids))
}

/// Adds the freshly-installed KLID to the current user's input methods so the
/// keyboard layout becomes selectable from the language bar without a manual
/// trip to Windows Settings. Also purges any stale KLIDs the install step
/// wiped out of HKLM, so dead Preload entries and dead InputMethodTips don't
/// linger in the user profile. Runs un-elevated (HKCU is per-user).
#[cfg(target_os = "windows")]
fn activate_for_user(
    work_dir: &std::path::Path,
    klid: &str,
    lang_tag: &str,
    stale_klids: &[String],
) -> Result<(), String> {
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

    // PowerShell-array literal of single-quoted KLID strings, e.g. @('a001040c','a002040c')
    // or @() if there are none. We escape single quotes in case (paranoid).
    let stale_array = if stale_klids.is_empty() {
        "@()".to_string()
    } else {
        let items: Vec<String> = stale_klids
            .iter()
            .map(|k| format!("'{}'", k.replace('\'', "''")))
            .collect();
        format!("@({})", items.join(","))
    };

    let script = format!(
        r#"
$ErrorActionPreference = 'Continue'
$tip       = '{tip}'
$langTag   = '{lang_tag}'
$klid      = '{klid}'
$staleKlids = {stale_array}

Write-Host "[activate] tip=$tip langTag=$langTag klid=$klid staleKlids=$($staleKlids -join ',')"

# ── Purge stale KLIDs from HKCU\Keyboard Layout\Preload ───────────────────
# The install step deleted these from HKLM, but their Preload references stay
# until we explicitly drop them — otherwise Windows logs errors trying to load
# a non-existent layout file at every logon.
try {{
    $preload = 'HKCU:\Keyboard Layout\Preload'
    if (Test-Path -LiteralPath $preload) {{
        $names = (Get-Item -LiteralPath $preload).GetValueNames() | Where-Object {{ $_ -match '^\d+$' }}
        foreach ($name in $names) {{
            $val = (Get-ItemProperty -LiteralPath $preload -Name $name).$name
            if ($staleKlids -contains $val) {{
                Write-Host "[activate] Purging stale Preload entry $name=$val"
                Remove-ItemProperty -LiteralPath $preload -Name $name -ErrorAction SilentlyContinue
            }}
        }}
    }}
}} catch {{
    Write-Host "[activate] Preload purge FAILED: $_"
}}

# ── Drop stale InputMethodTips from WinUserLanguageList ───────────────────
try {{
    $list = Get-WinUserLanguageList
    $changed = $false
    foreach ($lang in $list) {{
        $toRemove = @()
        foreach ($tip in $lang.InputMethodTips) {{
            $tipKlid = ($tip -split ':')[1]
            if ($staleKlids -contains $tipKlid) {{ $toRemove += $tip }}
        }}
        foreach ($t in $toRemove) {{
            Write-Host "[activate] Purging stale tip $t from $($lang.LanguageTag)"
            $null = $lang.InputMethodTips.Remove($t)
            $changed = $true
        }}
    }}
    if ($changed) {{
        Set-WinUserLanguageList $list -Force
        $list = Get-WinUserLanguageList
    }}
}} catch {{
    Write-Host "[activate] Stale-tip purge FAILED: $_"
}}

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

# ── Force the new layout to become the ACTIVE one in this session ─────────
# Without this, deleting the old KLID (replace step) leaves Windows on its
# fallback (US QWERTY) until the user manually picks the new layout from
# Win+Space. LoadKeyboardLayout + a WM_INPUTLANGCHANGEREQUEST broadcast
# switches every running top-level window over.
try {{
    Add-Type -Namespace 'MagicWindows' -Name 'Kbd' -MemberDefinition @'
[System.Runtime.InteropServices.DllImport("user32.dll", CharSet = System.Runtime.InteropServices.CharSet.Auto, SetLastError = true)]
public static extern System.IntPtr LoadKeyboardLayout(string pwszKLID, uint Flags);

[System.Runtime.InteropServices.DllImport("user32.dll", SetLastError = true)]
public static extern System.IntPtr SendMessageTimeout(System.IntPtr hWnd, uint Msg, System.IntPtr wParam, System.IntPtr lParam, uint fuFlags, uint uTimeout, out System.UIntPtr lpdwResult);
'@ -ErrorAction Stop

    $KLF_ACTIVATE       = 0x00000001
    $KLF_SUBSTITUTE_OK  = 0x00000002
    $KLF_SETFORPROCESS  = 0x00000100
    $hkl = [MagicWindows.Kbd]::LoadKeyboardLayout($klid, ($KLF_ACTIVATE -bor $KLF_SUBSTITUTE_OK -bor $KLF_SETFORPROCESS))
    if ($hkl -eq [System.IntPtr]::Zero) {{
        Write-Host "[activate] LoadKeyboardLayout returned NULL"
    }} else {{
        $HWND_BROADCAST = [System.IntPtr]0xFFFF
        $WM_INPUTLANGCHANGEREQUEST = 0x0050
        $INPUTLANGCHANGE_SYSCHARSET = 0x0001
        $SMTO_ABORTIFHUNG = 0x0002
        $result = [System.UIntPtr]::Zero
        $null = [MagicWindows.Kbd]::SendMessageTimeout(
            $HWND_BROADCAST,
            $WM_INPUTLANGCHANGEREQUEST,
            [System.IntPtr]$INPUTLANGCHANGE_SYSCHARSET,
            $hkl,
            $SMTO_ABORTIFHUNG,
            500,
            [ref]$result)
        Write-Host "[activate] Broadcast WM_INPUTLANGCHANGEREQUEST hkl=$hkl"
    }}
}} catch {{
    Write-Host "[activate] LoadKeyboardLayout/Broadcast FAILED: $_"
}}

Write-Host "[activate] done"
"#
    );

    write_ps_with_bom(&ps_path, &script)
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

# ── Remove HKLM registry entries and capture removed KLIDs ────────────────
$regPath = 'HKLM:\SYSTEM\CurrentControlSet\Control\Keyboard Layouts'
$removedKlids = @()
$entries = Get-ChildItem $regPath | Where-Object {{
    (Get-ItemProperty $_.PSPath).'Layout File' -eq "$DllName.dll"
}}
foreach ($entry in $entries) {{
    $removedKlids += $entry.PSChildName
    Remove-Item $entry.PSPath -Recurse -Force
}}

# Stash removed KLIDs in HKLM marker so the user-side post-step can purge HKCU.
$markerKey = 'HKLM:\SOFTWARE\MagicWindows'
if (-not (Test-Path -LiteralPath $markerKey)) {{
    New-Item -Path $markerKey -Force | Out-Null
}}
$staleJoined = ($removedKlids -join ',')
Set-ItemProperty -LiteralPath $markerKey -Name 'LastUninstalledKLIDs' -Value $staleJoined -Force
Write-Host "Removed KLIDs: $staleJoined"

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

    // ── User-side HKCU cleanup: purge dead Preload + InputMethodTips. ─────
    if let Err(e) = purge_hkcu_after_uninstall(&install_dir) {
        log::warn!("Layout uninstalled but HKCU purge failed: {e}");
    }
    Ok(())
}

/// Reads the LastUninstalledKLIDs marker the elevated uninstall step wrote, then
/// runs an unprivileged PowerShell to remove every matching value from
/// HKCU\Keyboard Layout\Preload and from each language's InputMethodTips list.
/// Without this, Windows logs "layout file missing" errors on every logon for
/// the dead KLIDs the user previously had loaded.
/// Run an unprivileged PowerShell to remove the given KLIDs from
/// `HKCU\Keyboard Layout\Preload` and from each language's `InputMethodTips`.
/// Called by both the layout-uninstall path and the new uninstall-by-klid path.
#[cfg(target_os = "windows")]
fn purge_hkcu_klids(work_dir: &std::path::Path, klids: &[String]) -> Result<(), String> {
    use std::process::Command;

    if klids.is_empty() {
        return Ok(());
    }

    let klids_ps_array = format!(
        "@({})",
        klids
            .iter()
            .map(|k| format!("'{}'", k.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(",")
    );

    let script = format!(r#"
$staleKlids = {klids_ps_array}

try {{
    $preload = 'HKCU:\Keyboard Layout\Preload'
    if (Test-Path -LiteralPath $preload) {{
        $names = (Get-Item -LiteralPath $preload).GetValueNames() | Where-Object {{ $_ -match '^\d+$' }}
        foreach ($name in $names) {{
            $val = (Get-ItemProperty -LiteralPath $preload -Name $name).$name
            if ($staleKlids -contains $val) {{
                Write-Host "Purging Preload $name=$val"
                Remove-ItemProperty -LiteralPath $preload -Name $name -ErrorAction SilentlyContinue
            }}
        }}
    }}
}} catch {{ Write-Host "Preload purge failed: $_" }}

try {{
    $list = Get-WinUserLanguageList
    $changed = $false
    foreach ($lang in $list) {{
        $toRemove = @()
        foreach ($tip in $lang.InputMethodTips) {{
            $tipKlid = ($tip -split ':')[1]
            if ($staleKlids -contains $tipKlid) {{ $toRemove += $tip }}
        }}
        foreach ($t in $toRemove) {{
            Write-Host "Purging tip $t from $($lang.LanguageTag)"
            $null = $lang.InputMethodTips.Remove($t)
            $changed = $true
        }}
    }}
    if ($changed) {{ Set-WinUserLanguageList $list -Force }}
}} catch {{ Write-Host "Tip purge failed: $_" }}
"#);

    let ps_path = work_dir.join("hkcu_purge.ps1");
    write_ps_with_bom(&ps_path, &script).map_err(|e| format!("write purge script: {e}"))?;
    let out = Command::new("powershell")
        .args(["-ExecutionPolicy", "Bypass", "-NoProfile", "-File", &ps_path.to_string_lossy()])
        .output()
        .map_err(|e| format!("spawn purge powershell: {e}"))?;
    if !out.status.success() {
        return Err(format!("HKCU purge failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(())
}

/// Backwards-compat wrapper: reads the `LastUninstalledKLIDs` marker from HKLM
/// (written by the elevated uninstall step) and purges HKCU accordingly.
#[cfg(target_os = "windows")]
fn purge_hkcu_after_uninstall(work_dir: &std::path::Path) -> Result<(), String> {
    use std::process::Command;

    let read_script = r#"
$key = 'HKLM:\SOFTWARE\MagicWindows'
if (-not (Test-Path -LiteralPath $key)) { Write-Host ''; exit 0 }
$v = (Get-ItemProperty -LiteralPath $key -Name 'LastUninstalledKLIDs' -ErrorAction SilentlyContinue).LastUninstalledKLIDs
if ($v) { Write-Host $v }
"#;
    let out = Command::new("powershell")
        .args(["-ExecutionPolicy", "Bypass", "-NoProfile", "-Command", read_script])
        .output()
        .map_err(|e| format!("spawn read marker: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let klids: Vec<String> = if stdout.is_empty() {
        Vec::new()
    } else {
        stdout
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };
    purge_hkcu_klids(work_dir, &klids)
}

/// Uninstall any keyboard layout by its registry KLID. Works for both
/// MagicWindows layouts (also removes the DLL from System32 and SysWOW64)
/// and OEM/system layouts (registry entry only; DLL in System32 stays put —
/// Microsoft's own DLLs are protected and re-addable via Windows Settings).
#[cfg(target_os = "windows")]
#[tauri::command]
pub fn uninstall_by_klid(klid: String) -> Result<(), String> {
    use std::fs;

    // KLID validation: 8 hex chars. Prevents PS injection via malformed klid.
    if klid.len() != 8 || !klid.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("Invalid KLID: '{klid}' (expected 8 hex chars)"));
    }

    let install_dir = get_install_dir();
    fs::create_dir_all(&install_dir)
        .map_err(|e| format!("Failed to create work dir: {e}"))?;

    let ps_script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {{
    throw "Administrator privileges are required to uninstall keyboard layouts."
}}

$klid = '{klid}'
$regPath = "HKLM:\SYSTEM\CurrentControlSet\Control\Keyboard Layouts\$klid"

if (-not (Test-Path -LiteralPath $regPath)) {{
    throw "Layout $klid is not installed."
}}

$layoutFile = (Get-ItemProperty -LiteralPath $regPath -Name 'Layout File' -ErrorAction SilentlyContinue).'Layout File'
Write-Host "Removing registry entry $klid (Layout File: $layoutFile)"
Remove-Item -LiteralPath $regPath -Recurse -Force

# Only delete the DLL if it is one of ours (kbdapl*). Microsoft DLLs must stay.
if ($layoutFile -and $layoutFile.ToLower().StartsWith('kbdapl')) {{
    $sys32Dll = "$env:SystemRoot\System32\$layoutFile"
    if (Test-Path -LiteralPath $sys32Dll) {{
        Write-Host "Removing MagicWindows DLL: $sys32Dll"
        Remove-Item -LiteralPath $sys32Dll -Force
    }}
    $wow64Dll = "$env:SystemRoot\SysWOW64\$layoutFile"
    if (Test-Path -LiteralPath $wow64Dll) {{
        Write-Host "Removing stray SysWOW64 DLL: $wow64Dll"
        Remove-Item -LiteralPath $wow64Dll -Force
    }}
}}

# Stash the removed KLID for the HKCU purge step that follows.
$markerKey = 'HKLM:\SOFTWARE\MagicWindows'
if (-not (Test-Path -LiteralPath $markerKey)) {{ New-Item -Path $markerKey -Force | Out-Null }}
Set-ItemProperty -LiteralPath $markerKey -Name 'LastUninstalledKLIDs' -Value $klid -Force

Write-Host "Uninstall complete."
"#
    );

    run_elevated_ps(&install_dir, "uninstall_by_klid", &ps_script)?;
    log::info!("Layout KLID {klid} uninstalled");

    if let Err(e) = purge_hkcu_klids(&install_dir, std::slice::from_ref(&klid)) {
        log::warn!("Layout uninstalled but HKCU purge failed: {e}");
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn uninstall_by_klid(_klid: String) -> Result<(), String> {
    Err("Uninstallation requires Windows.".to_string())
}

/// Write a PowerShell script to disk with a UTF-8 BOM. Windows PowerShell 5.1
/// reads `.ps1` files as ANSI/Windows-1252 by default unless a BOM is present —
/// without one, any non-ASCII byte (em-dash, box-drawing chars, accented
/// letters) gets misinterpreted, potentially corrupting string literals and
/// triggering bogus parser errors. Writing the BOM forces UTF-8 detection.
#[cfg(target_os = "windows")]
fn write_ps_with_bom(path: &std::path::Path, contents: &str) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::File::create(path)?;
    f.write_all(&[0xEF, 0xBB, 0xBF])?;
    f.write_all(contents.as_bytes())?;
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

    let ps_path         = work_dir.join(format!("{label}.ps1"));
    let transcript_path = work_dir.join(format!("{label}_transcript.txt"));
    let exitcode_path   = work_dir.join(format!("{label}_exitcode.txt"));

    // ── Build the child script with self-contained logging.
    //
    // `Start-Process -Verb RunAs` uses ShellExecuteEx, which does NOT support
    // -RedirectStandardOutput / -RedirectStandardError. Combining the two yields
    // a $null process object and (silently) a launcher exit of 0, making the
    // parent believe the install succeeded when nothing actually ran. The fix:
    //  1. Don't redirect from the parent.
    //  2. The child opens its own transcript and writes its exit code to a known
    //     file before exiting, so the parent can detect "child never ran" vs
    //     "child ran and failed" vs "child ran and succeeded".
    let wrapped = format!(
        r#"$ErrorActionPreference = 'Continue'
try {{ Start-Transcript -Path '{transcript}' -Force | Out-Null }} catch {{ }}
'PENDING' | Out-File -LiteralPath '{exitcode}' -Encoding ascii -Force
$childExit = 1
try {{
{body}
    $childExit = 0
}} catch {{
    Write-Host "[run_elevated_ps] child threw: $_"
    $childExit = 1
}}
try {{ Stop-Transcript | Out-Null }} catch {{ }}
$childExit | Out-File -LiteralPath '{exitcode}' -Encoding ascii -Force
exit $childExit
"#,
        transcript = transcript_path.display(),
        exitcode   = exitcode_path.display(),
        body       = ps_script,
    );

    write_ps_with_bom(&ps_path, &wrapped)
        .map_err(|e| format!("Failed to write {label} script: {e}"))?;

    // Pre-write a sentinel so we can distinguish "child never ran" from "child wrote 0".
    let _ = fs::write(&exitcode_path, "NEVER_RAN");

    // Launcher: spawn elevated child WITHOUT stream redirection (incompatible with -Verb RunAs).
    // Detect the $null-process case explicitly to surface UAC-cancel / launch failures.
    let launcher = format!(
        r#"
$ErrorActionPreference = 'Stop'
try {{
    $proc = Start-Process powershell `
        -ArgumentList @('-ExecutionPolicy','Bypass','-NoProfile','-File','{ps}') `
        -Verb RunAs `
        -Wait `
        -PassThru
}} catch {{
    Write-Error "Start-Process failed: $_"
    exit 2
}}
if ($null -eq $proc) {{
    Write-Error "Start-Process returned null (UAC cancelled?)"
    exit 3
}}
exit $proc.ExitCode
"#,
        ps = ps_path.display(),
    );

    let output = Command::new("powershell")
        .args(["-ExecutionPolicy", "Bypass", "-NoProfile", "-Command", &launcher])
        .output()
        .map_err(|e| format!("Failed to run PowerShell launcher: {e}"))?;

    let transcript = fs::read_to_string(&transcript_path).unwrap_or_default();
    let exitcode_str = fs::read_to_string(&exitcode_path).unwrap_or_default().trim().to_string();
    let launcher_stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    // The launcher must report success AND the child must have updated the exitcode file
    // with a 0. NEVER_RAN means Start-Process didn't actually launch the child.
    if !output.status.success() {
        return Err(format!(
            "Elevation launcher failed (exit {:?}): {}\nTranscript:\n{}",
            output.status.code(),
            launcher_stderr.trim(),
            transcript.trim(),
        ));
    }
    if exitcode_str == "NEVER_RAN" {
        return Err(format!(
            "Elevated child never ran (UAC cancelled or Start-Process failed silently). Launcher stderr: {}",
            launcher_stderr.trim(),
        ));
    }
    if exitcode_str == "PENDING" {
        return Err(format!(
            "Elevated child started but did not finish (crashed before exitcode write). Transcript:\n{}",
            transcript.trim(),
        ));
    }
    if exitcode_str != "0" {
        return Err(format!(
            "Elevated child failed (exit {}). Transcript:\n{}",
            exitcode_str,
            transcript.trim(),
        ));
    }

    Ok(transcript)
}

// ── Non-Windows stubs ───────────────────────────────────────────────────────

#[cfg(not(target_os = "windows"))]
pub fn install_layout(_layout: &Layout, _app: &tauri::AppHandle) -> Result<u64, String> {
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

// ── Settings page: list every registered keyboard layout ────────────────────

/// Enumerate every layout registered in `HKLM\SYSTEM\CurrentControlSet\Control\Keyboard Layouts`.
/// Unprivileged read (HKLM is world-readable). Returns a flag per layout indicating
/// whether it was installed by MagicWindows (DLL name starts with `kbdapl`) and
/// whether it is currently referenced by the user's `HKCU\Keyboard Layout\Preload`.
#[cfg(target_os = "windows")]
#[tauri::command]
pub fn list_all_installed_layouts() -> Result<Vec<super::InstalledLayoutInfo>, String> {
    use std::process::Command;

    // PowerShell is the simplest way to iterate HKLM Keyboard Layouts without
    // pulling in the `windows` crate. HKLM and HKCU are both user-readable so
    // no elevation required.
    let script = r#"
$ErrorActionPreference = 'Stop'
$root = 'HKLM:\SYSTEM\CurrentControlSet\Control\Keyboard Layouts'
$preload = 'HKCU:\Keyboard Layout\Preload'

# Collect all KLIDs currently loaded for this user (for the isInUse flag).
$inUse = @{}
if (Test-Path -LiteralPath $preload) {
    $names = (Get-Item -LiteralPath $preload).GetValueNames() | Where-Object { $_ -match '^\d+$' }
    foreach ($name in $names) {
        $val = (Get-ItemProperty -LiteralPath $preload -Name $name).$name
        if ($val) { $inUse[$val.ToLower()] = $true }
    }
}

# One line per layout: klid|layout_file|layout_text|isMagicWindows|isInUse
Get-ChildItem -LiteralPath $root -ErrorAction SilentlyContinue | ForEach-Object {
    $klid = $_.PSChildName
    $props = Get-ItemProperty -LiteralPath $_.PSPath -ErrorAction SilentlyContinue
    $file = if ($props.'Layout File') { $props.'Layout File' } else { '' }
    $text = if ($props.'Layout Text') { $props.'Layout Text' } else { '' }
    $isMw = $file.ToLower().StartsWith('kbdapl')
    $iu   = $inUse.ContainsKey($klid.ToLower())
    # Replace any literal pipes in text to keep the separator unambiguous on the Rust side.
    $safeText = $text -replace '\|', '/'
    "$klid|$file|$safeText|$isMw|$iu"
}
"#;

    let output = Command::new("powershell")
        .args(["-ExecutionPolicy", "Bypass", "-NoProfile", "-Command", script])
        .output()
        .map_err(|e| format!("Failed to invoke powershell: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "powershell list failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut out = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        let parts: Vec<&str> = line.splitn(5, '|').collect();
        if parts.len() != 5 { continue; }
        out.push(super::InstalledLayoutInfo {
            klid: parts[0].to_string(),
            layout_file: parts[1].to_string(),
            layout_text: parts[2].to_string(),
            is_magic_windows: parts[3].eq_ignore_ascii_case("True"),
            is_in_use: parts[4].eq_ignore_ascii_case("True"),
        });
    }

    // Deterministic ordering by KLID so the UI renders identically across runs.
    out.sort_by(|a, b| a.klid.cmp(&b.klid));
    Ok(out)
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn list_all_installed_layouts() -> Result<Vec<super::InstalledLayoutInfo>, String> {
    Err("Listing layouts requires Windows.".to_string())
}
