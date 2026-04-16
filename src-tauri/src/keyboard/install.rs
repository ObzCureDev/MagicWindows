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

    // PowerShell install script.
    //
    // kbdutool discovery order:
    //   1. Common MSKLC 1.4 install locations (x86 and x64 Program Files)
    //   2. Older MSKLC install location without version suffix
    //   3. PATH via Get-Command
    //
    // If none of those succeed the script throws a human-readable error that
    // includes the official MSKLC download URL so non-technical users know
    // exactly what to install.
    //
    // The script is written to a .ps1 file and then launched in a *new*
    // elevated (administrator) PowerShell process via Start-Process -Verb RunAs
    // so that copying the DLL into System32 succeeds.  The outer process waits
    // for that elevated child to finish and surfaces its exit code.
    let ps_script = format!(
        r#"
$ErrorActionPreference = 'Stop'

# ── Privilege check ────────────────────────────────────────────────────────
$currentPrincipal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
$isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {{
    throw "MagicWindows must be run as Administrator to install keyboard layouts. Right-click the application and choose 'Run as administrator'."
}}

# ── Locate kbdutool ────────────────────────────────────────────────────────
$candidatePaths = @(
    'C:\Program Files (x86)\Microsoft Keyboard Layout Creator 1.4\bin\i386\kbdutool.exe',
    'C:\Program Files\Microsoft Keyboard Layout Creator 1.4\bin\i386\kbdutool.exe',
    'C:\Program Files (x86)\Microsoft Keyboard Layout Creator\bin\i386\kbdutool.exe'
)

$kbdutoolPath = $null
foreach ($candidate in $candidatePaths) {{
    if (Test-Path $candidate) {{
        $kbdutoolPath = $candidate
        break
    }}
}}

if (-not $kbdutoolPath) {{
    $fromPath = Get-Command kbdutool -ErrorAction SilentlyContinue
    if ($fromPath) {{
        $kbdutoolPath = $fromPath.Source
    }}
}}

if (-not $kbdutoolPath) {{
    throw @"
kbdutool.exe was not found. MagicWindows requires Microsoft Keyboard Layout Creator (MSKLC) to compile and install keyboard layouts.

Please download and install MSKLC 1.4 from:
  https://www.microsoft.com/en-us/download/details.aspx?id=102134

After installing MSKLC, try again.
"@
}}

# ── Compile and install via kbdutool ──────────────────────────────────────
$klcPath = '{klc}'
$dllName = '{dll}'

& "$kbdutoolPath" -u -s "$klcPath"
if ($LASTEXITCODE -ne 0) {{
    throw "kbdutool failed with exit code $LASTEXITCODE"
}}
"#,
        klc = klc_path.display(),
        dll = layout.dll_name,
    );

    let ps_path = install_dir.join("install.ps1");
    fs::write(&ps_path, &ps_script)
        .map_err(|e| format!("Failed to write install script: {e}"))?;

    // Launch the script in an elevated PowerShell process.  Start-Process
    // -Verb RunAs triggers the UAC prompt; -Wait makes this process block
    // until the elevated child exits; -PassThru lets us capture the exit code.
    // Stdout/stderr from the elevated child are not automatically inherited, so
    // we redirect them to temporary files and read them back after the child
    // exits.
    let stdout_path = install_dir.join("install_stdout.txt");
    let stderr_path = install_dir.join("install_stderr.txt");

    // This outer script uses Start-Process to spawn the elevated child and
    // captures its streams.
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
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-NoProfile",
            "-Command",
            &launcher,
        ])
        .output()
        .map_err(|e| format!("Failed to run PowerShell launcher: {e}"))?;

    if output.status.success() {
        log::info!("Layout {} installed successfully", layout.id);
        Ok(())
    } else {
        // Try to surface the error text written by the elevated child first;
        // fall back to the launcher's own stderr if those files are absent.
        let child_stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
        let child_stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
        let launcher_stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        let detail = [child_stderr.trim(), child_stdout.trim(), launcher_stderr.trim()]
            .iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        Err(format!("Installation failed: {detail}"))
    }
}

#[cfg(target_os = "windows")]
pub fn uninstall_layout(layout: &Layout) -> Result<(), String> {
    use std::fs;
    use std::process::Command;

    let install_dir = get_install_dir();
    // Ensure the directory exists so we can write the temporary script and
    // capture stream files even when the layout was never fully installed.
    fs::create_dir_all(&install_dir)
        .map_err(|e| format!("Failed to create install dir: {e}"))?;

    let ps_script = format!(
        r#"
$ErrorActionPreference = 'Stop'

# ── Privilege check ────────────────────────────────────────────────────────
$currentPrincipal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
$isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {{
    throw "MagicWindows must be run as Administrator to uninstall keyboard layouts. Right-click the application and choose 'Run as administrator'."
}}

$dllName = '{dll}'

# ── Remove registry entries ────────────────────────────────────────────────
$regPath = 'HKLM:\SYSTEM\CurrentControlSet\Control\Keyboard Layouts'
$entries = Get-ChildItem $regPath | Where-Object {{
    (Get-ItemProperty $_.PSPath).'Layout File' -eq "$dllName.dll"
}}
foreach ($entry in $entries) {{
    Remove-Item $entry.PSPath -Force
}}

# ── Remove the DLL from System32 ──────────────────────────────────────────
$sys32Path = "$env:SystemRoot\System32\$dllName.dll"
if (Test-Path $sys32Path) {{
    Remove-Item $sys32Path -Force
}}

# ── Remove the DLL from SysWOW64 (32-bit copy on 64-bit Windows) ──────────
$wow64Path = "$env:SystemRoot\SysWOW64\$dllName.dll"
if (Test-Path $wow64Path) {{
    Remove-Item $wow64Path -Force
}}

# ── Clean up local working files ──────────────────────────────────────────
$installDir = '{install_dir}'
if (Test-Path $installDir) {{
    Remove-Item "$installDir\$dllName.*" -Force -ErrorAction SilentlyContinue
}}
"#,
        dll         = layout.dll_name,
        install_dir = install_dir.display(),
    );

    let ps_path = install_dir.join("uninstall.ps1");
    fs::write(&ps_path, &ps_script)
        .map_err(|e| format!("Failed to write uninstall script: {e}"))?;

    let stdout_path = install_dir.join("uninstall_stdout.txt");
    let stderr_path = install_dir.join("uninstall_stderr.txt");

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
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-NoProfile",
            "-Command",
            &launcher,
        ])
        .output()
        .map_err(|e| format!("Failed to run PowerShell launcher: {e}"))?;

    if output.status.success() {
        log::info!("Layout {} uninstalled successfully", layout.id);
        Ok(())
    } else {
        let child_stderr = fs::read_to_string(&stderr_path).unwrap_or_default();
        let child_stdout = fs::read_to_string(&stdout_path).unwrap_or_default();
        let launcher_stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        let detail = [child_stderr.trim(), child_stdout.trim(), launcher_stderr.trim()]
            .iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        Err(format!("Uninstallation failed: {detail}"))
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
