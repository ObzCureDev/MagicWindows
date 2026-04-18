# Settings Page + Elevated Error Flow — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add (1) a Settings page listing every installed keyboard layout with per-row remove, (2) a one-click "remove Windows default" bloc on the Done page, (3) a reusable `ElevatedErrorPanel` component with Retry and Send bug report (mailto: to bug@mindvisionstudio.com), replacing raw error output across every elevated operation. Also produce a portable .exe alongside the existing NSIS installer.

**Architecture:** Three new Tauri commands (`list_all_keyboard_layouts`, `uninstall_layout_by_klid`, `collect_diagnostics`) sit next to the existing install plumbing and reuse `run_elevated_ps` for elevated ops. One Svelte component (`ElevatedErrorPanel`) centralises friendly error UX + retry + mailto handoff. The Settings page and Done-page bloc consume the new list/uninstall commands. Bundle config adds MSI + a post-build portable ZIP.

**Tech Stack:** Rust (Tauri v2 commands + elevated PowerShell), Svelte 5 ($state runes), TypeScript, Vitest (unit), vitest + manual QA (UI), NSIS + MSI + ZIP (distribution).

**Spec:** [docs/superpowers/specs/2026-04-19-settings-error-flow-design.md](../specs/2026-04-19-settings-error-flow-design.md)

---

## File Structure

| File | Status | Responsibility |
|------|--------|----------------|
| `src-tauri/src/keyboard/mod.rs` | modify | Add `InstalledLayoutInfo` struct |
| `src-tauri/src/keyboard/install.rs` | modify | Add `list_all_installed_layouts()`, `uninstall_by_klid()`; extract shared HKCU purge |
| `src-tauri/src/keyboard/diagnostics.rs` | NEW | Compose the Markdown diagnostics block from sub-collectors |
| `src-tauri/src/lib.rs` | modify | Register 3 new Tauri commands |
| `src-tauri/tauri.conf.json` | modify | Bundle targets: `["nsis", "msi"]`; portable handled by post-build script |
| `scripts/build-portable.mjs` | NEW | Post-build Node script that zips `magicwindows.exe` + `kbd_dlls/` + `layouts/` into `magicwindows-portable.zip` |
| `package.json` | modify | `"build:portable": "npm run tauri build && node scripts/build-portable.mjs"` script |
| `src/lib/types.ts` | modify | `InstalledLayoutInfo` TS interface; add `"settings"` to `Page` union |
| `src/lib/i18n.ts` | modify | ~20 new EN/FR keys |
| `src/components/ElevatedErrorPanel.svelte` | NEW | Friendly error + Retry + Send bug report |
| `src/pages/Settings.svelte` | NEW | List + remove any installed layout |
| `src/pages/Install.svelte` | modify | Replace raw error div with `<ElevatedErrorPanel>` |
| `src/pages/Done.svelte` | modify | Add Windows-default-removal bloc; replace modifier error + uninstall error with panels |
| `src/App.svelte` | modify | ⚙ top-bar button + `{:else if appState.page === "settings"}` route |

---

## Task 1: Backend — `InstalledLayoutInfo` struct + mod exports

**Files:**
- Modify: `src-tauri/src/keyboard/mod.rs`

- [ ] **Step 1: Add the struct to `mod.rs`**

Append at the end of `src-tauri/src/keyboard/mod.rs`:

```rust
/// Metadata about a keyboard layout registered on the system (ours or OEM).
///
/// Read from `HKLM\SYSTEM\CurrentControlSet\Control\Keyboard Layouts\{klid}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledLayoutInfo {
    /// 8-char registry subkey name, e.g. "0000040c" or "a001040c".
    pub klid: String,
    /// `Layout File` registry value, e.g. "KBDFR.DLL" or "kbdaplfr.dll".
    pub layout_file: String,
    /// `Layout Text` registry value; may be empty if the key is malformed.
    pub layout_text: String,
    /// True when `layout_file.to_lowercase().starts_with("kbdapl")`.
    pub is_magic_windows: bool,
    /// True when `klid` appears in `HKCU\Keyboard Layout\Preload`.
    pub is_in_use: bool,
}
```

- [ ] **Step 2: Build check**

Run: `cd src-tauri && cargo check`
Expected: compiles (the struct isn't referenced yet but should not introduce warnings).

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/keyboard/mod.rs
git commit -m "feat(keyboard): add InstalledLayoutInfo struct"
```

---

## Task 2: Backend — `list_all_installed_layouts()` command

**Files:**
- Modify: `src-tauri/src/keyboard/install.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add the Tauri command to `install.rs`**

Append to `src-tauri/src/keyboard/install.rs` (keep the existing `#[cfg(target_os = "windows")]` gating pattern):

```rust
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
    # Pipe separator with backtick escape for any stray pipes in text.
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

    // Deterministic ordering: system layouts first (0000...), then ours, then anything else.
    out.sort_by(|a, b| a.klid.cmp(&b.klid));
    Ok(out)
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn list_all_installed_layouts() -> Result<Vec<super::InstalledLayoutInfo>, String> {
    Err("Listing layouts requires Windows.".to_string())
}
```

- [ ] **Step 2: Register the command in `src-tauri/src/lib.rs`**

In the `invoke_handler![…]` list inside `run()`, append after the existing `modifiers::clear_scancode_map,` line:

```rust
            crate::keyboard::install::list_all_installed_layouts,
```

- [ ] **Step 3: Build check**

Run: `cd src-tauri && cargo build`
Expected: compiles cleanly.

- [ ] **Step 4: Manual smoke test via dev console**

Run: `npm run tauri dev` in one terminal. Once app loads, open DevTools (Ctrl+Shift+I) → Console → paste:

```js
await __TAURI__.core.invoke("list_all_installed_layouts")
```

Expected: an array containing at least `0000040c` (French standard) and `a001040c` (or similar) for our installed layout. Each entry has `klid`, `layoutFile`, `layoutText`, `isMagicWindows`, `isInUse`.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/keyboard/install.rs src-tauri/src/lib.rs
git commit -m "feat(keyboard): add list_all_installed_layouts command"
```

---

## Task 3: Backend — shared HKCU purge helper + `uninstall_by_klid`

**Files:**
- Modify: `src-tauri/src/keyboard/install.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Extract `purge_hkcu_klids` helper from the current uninstall path**

Locate the existing `purge_hkcu_after_uninstall` function (around line 553 of `install.rs`). Rename it to `purge_hkcu_klids` and change its interface so the caller explicitly provides the stale-KLID list (rather than reading it from the `LastUninstalledKLIDs` HKLM marker). Keep the existing marker-based reader as a thin wrapper that calls the new helper.

Replace the body of the current function:

```rust
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

/// Backwards-compat wrapper: reads the `LastUninstalledKLIDs` marker from
/// HKLM (written by the elevated uninstall step) and purges HKCU accordingly.
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
        stdout.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    };
    purge_hkcu_klids(work_dir, &klids)
}
```

Delete the old `purge_hkcu_after_uninstall` body (the original PS-embedded version). Keep all call sites of `purge_hkcu_after_uninstall` unchanged — they continue to work via the wrapper.

- [ ] **Step 2: Add the `uninstall_by_klid` Tauri command**

Append to `src-tauri/src/keyboard/install.rs`:

```rust
/// Uninstall any keyboard layout by its registry KLID — works for both
/// MagicWindows layouts (removes DLL from System32) and OEM/system layouts
/// (registry entry only, DLL stays in System32).
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

    // Elevated PS: remove the registry subkey. Also read its `Layout File` first
    // so we can decide whether to also delete a MagicWindows DLL from System32.
    let ps_script = format!(r#"
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

# If the layout file starts with kbdapl, it's ours — delete from System32 too.
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

# Mark for HKCU cleanup.
$markerKey = 'HKLM:\SOFTWARE\MagicWindows'
if (-not (Test-Path -LiteralPath $markerKey)) {{ New-Item -Path $markerKey -Force | Out-Null }}
Set-ItemProperty -LiteralPath $markerKey -Name 'LastUninstalledKLIDs' -Value $klid -Force

Write-Host "Uninstall complete."
"#);

    run_elevated_ps(&install_dir, "uninstall_by_klid", &ps_script)?;
    log::info!("Layout KLID {klid} uninstalled");

    // Unprivileged HKCU purge.
    if let Err(e) = purge_hkcu_klids(&install_dir, &[klid.clone()]) {
        log::warn!("Layout uninstalled but HKCU purge failed: {e}");
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn uninstall_by_klid(_klid: String) -> Result<(), String> {
    Err("Uninstallation requires Windows.".to_string())
}
```

- [ ] **Step 3: Register the command in `src-tauri/src/lib.rs`**

In `invoke_handler![…]`, append after the `list_all_installed_layouts,` line:

```rust
            crate::keyboard::install::uninstall_by_klid,
```

- [ ] **Step 4: Build check**

Run: `cd src-tauri && cargo build`
Expected: compiles cleanly. No unused-function warnings.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/keyboard/install.rs src-tauri/src/lib.rs
git commit -m "feat(keyboard): add uninstall_by_klid + share HKCU purge helper"
```

---

## Task 4: Backend — `collect_diagnostics` command

**Files:**
- Create: `src-tauri/src/keyboard/diagnostics.rs`
- Modify: `src-tauri/src/keyboard/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create `src-tauri/src/keyboard/diagnostics.rs`**

```rust
//! Compose a human-readable Markdown diagnostics block for bug reports.
//! Every collector is best-effort: failures inline a marker but never bubble
//! up — the user must always get *some* report to send.

use super::InstalledLayoutInfo;
use std::process::Command;

/// Tauri command: returns a pre-formatted Markdown block suitable for pasting
/// into an email body. Called by the ElevatedErrorPanel's "Send bug report".
#[cfg(target_os = "windows")]
#[tauri::command]
pub fn collect_diagnostics() -> Result<String, String> {
    let mut out = String::with_capacity(4 * 1024);
    out.push_str("## MagicWindows Bug Report\n\n");

    // App version (from Cargo.toml).
    out.push_str(&format!("**App version:** {}\n", env!("CARGO_PKG_VERSION")));

    // OS version via `cmd /c ver`.
    match Command::new("cmd").args(["/c", "ver"]).output() {
        Ok(o) if o.status.success() => {
            let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
            out.push_str(&format!("**OS:** {v}\n"));
        }
        _ => out.push_str("**OS:** *(ver command failed)*\n"),
    }

    // Active HKL via PowerShell (LoadKeyboardLayout would require P/Invoke).
    match Command::new("powershell")
        .args([
            "-ExecutionPolicy", "Bypass", "-NoProfile", "-Command",
            r#"
Add-Type 'using System; using System.Runtime.InteropServices; public class Hkl { [DllImport("user32.dll")] public static extern IntPtr GetKeyboardLayout(uint t); }'
"0x{0:X8}" -f [Hkl]::GetKeyboardLayout(0).ToInt32()
"#,
        ])
        .output()
    {
        Ok(o) if o.status.success() => {
            out.push_str(&format!(
                "**Active layout (HKL):** {}\n",
                String::from_utf8_lossy(&o.stdout).trim()
            ));
        }
        _ => out.push_str("**Active layout (HKL):** *(lookup failed)*\n"),
    }

    out.push('\n');

    // Installed layouts.
    match super::install::list_all_installed_layouts() {
        Ok(layouts) => {
            out.push_str("**Installed MagicWindows layouts:**\n");
            let mw: Vec<&InstalledLayoutInfo> = layouts.iter().filter(|l| l.is_magic_windows).collect();
            if mw.is_empty() {
                out.push_str("- *(none)*\n");
            } else {
                for l in &mw {
                    out.push_str(&format!("- `{}` / `{}` / \"{}\"\n", l.klid, l.layout_file, l.layout_text));
                }
            }
            out.push('\n');

            out.push_str("**All installed keyboard layouts:**\n");
            for l in &layouts {
                out.push_str(&format!("- `{}` / `{}` / \"{}\"\n", l.klid, l.layout_file, l.layout_text));
            }
            out.push('\n');
        }
        Err(e) => {
            out.push_str(&format!("**Installed layouts:** *(collector failed: {e})*\n\n"));
        }
    }

    // Scancode Map.
    match super::modifiers::read_scancode_map() {
        Ok(state) => {
            out.push_str("**Scancode Map (modifier swaps):**\n");
            if state.raw_entries.is_empty() {
                out.push_str("- *(none)*\n");
            } else {
                for pair in &state.raw_entries {
                    out.push_str(&format!("- new=`{}` old=`{}`\n", pair.new_code, pair.old_code));
                }
            }
            out.push('\n');
        }
        Err(e) => {
            out.push_str(&format!("**Scancode Map:** *(collector failed: {e})*\n\n"));
        }
    }

    // Last PS transcript tail (from %LOCALAPPDATA%\MagicWindows\layouts).
    out.push_str("**Last elevated PS transcript (tail 50 lines):**\n\n```\n");
    out.push_str(&collect_last_transcript(50));
    out.push_str("\n```\n");

    Ok(out)
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn collect_diagnostics() -> Result<String, String> {
    Ok(format!(
        "## MagicWindows Bug Report\n\n**App version:** {}\n**OS:** non-Windows (not supported)\n",
        env!("CARGO_PKG_VERSION")
    ))
}

#[cfg(target_os = "windows")]
fn collect_last_transcript(max_lines: usize) -> String {
    use std::fs;

    let dir = super::install::get_install_dir();
    if !dir.exists() {
        return "*(no transcript directory yet)*".to_string();
    }

    // Find the most recent *_transcript.txt file.
    let newest = match fs::read_dir(&dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .ends_with("_transcript.txt")
            })
            .max_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok())),
        Err(_) => None,
    };

    let Some(entry) = newest else {
        return "*(no transcript files)*".to_string();
    };

    let content = match fs::read_to_string(entry.path()) {
        Ok(c) => c,
        Err(e) => return format!("*(failed to read transcript: {e})*"),
    };

    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    let tail = lines[start..].join("\n");

    // Truncate to reasonable size for mailto URL limits.
    const MAX_BYTES: usize = 8 * 1024;
    if tail.len() > MAX_BYTES {
        format!(
            "*(truncated from {} to {} bytes)*\n{}",
            tail.len(),
            MAX_BYTES,
            &tail[tail.len() - MAX_BYTES..]
        )
    } else {
        tail
    }
}
```

- [ ] **Step 2: Export the new module in `src-tauri/src/keyboard/mod.rs`**

Add to the `pub mod …;` list at the top:

```rust
pub mod diagnostics;
```

- [ ] **Step 3: Register the command in `src-tauri/src/lib.rs`**

In `invoke_handler![…]`, append after the `uninstall_by_klid,` line:

```rust
            crate::keyboard::diagnostics::collect_diagnostics,
```

- [ ] **Step 4: Build check**

Run: `cd src-tauri && cargo build`
Expected: compiles cleanly.

- [ ] **Step 5: Manual smoke test**

Run: `npm run tauri dev`. In DevTools console:

```js
console.log(await __TAURI__.core.invoke("collect_diagnostics"))
```

Expected: Markdown block with `## MagicWindows Bug Report`, OS line, list of layouts, scancode map (or `(none)`), and a transcript tail (or `(no transcript files)` on first run).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/keyboard/diagnostics.rs src-tauri/src/keyboard/mod.rs src-tauri/src/lib.rs
git commit -m "feat(keyboard): add collect_diagnostics command for bug reports"
```

---

## Task 5: Frontend — TypeScript types + Page union

**Files:**
- Modify: `src/lib/types.ts`

- [ ] **Step 1: Add `"settings"` to the `Page` union**

In `src/lib/types.ts`, change the `Page` type line from:

```ts
export type Page = "welcome" | "detect" | "select" | "preview" | "install" | "test" | "done" | "about" | "modifiers";
```

to:

```ts
export type Page = "welcome" | "detect" | "select" | "preview" | "install" | "test" | "done" | "about" | "modifiers" | "settings";
```

- [ ] **Step 2: Add `InstalledLayoutInfo` interface**

Append at the end of the file:

```ts
// ── Settings page: system layout management
// (see docs/superpowers/specs/2026-04-19-settings-error-flow-design.md)

export interface InstalledLayoutInfo {
  /** 8-char registry subkey name, e.g. "0000040c". */
  klid: string;
  /** `Layout File` registry value, e.g. "KBDFR.DLL" or "kbdaplfr.dll". */
  layoutFile: string;
  /** `Layout Text` registry value; may be empty. */
  layoutText: string;
  /** True when this layout was installed by MagicWindows (layoutFile starts with "kbdapl"). */
  isMagicWindows: boolean;
  /** True when this KLID appears in HKCU\Keyboard Layout\Preload. */
  isInUse: boolean;
}
```

- [ ] **Step 3: Verify types compile**

Run: `npm run check`
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/types.ts
git commit -m "feat(types): InstalledLayoutInfo + settings Page variant"
```

---

## Task 6: Frontend — i18n keys

**Files:**
- Modify: `src/lib/i18n.ts`

- [ ] **Step 1: Add new keys (EN + FR)**

Inside both the `en` and `fr` objects of `src/lib/i18n.ts`, add the following keys. Keep them grouped in a new `settings`, `elevatedError`, `bugReport` section and extend the existing `done` section.

**English section additions:**

```ts
    settings: {
      topbarTitle: "Settings",
      title: "Installed keyboard layouts",
      subtitle: "Review and remove any keyboard layout on your system.",
      badgeMagic: "MagicWindows",
      badgeSystem: "System",
      badgeInUse: "In use",
      remove: "Remove",
      reactivateInfo: "To re-enable: Windows Settings \u2192 Time & language \u2192 Language \u2192 [language] \u2192 Options \u2192 Add a keyboard.",
      confirmRemove: 'Remove "{name}"?',
      confirmRemoveInUse: 'This layout is currently used by your session. Remove anyway?',
      empty: "No keyboard layouts found.",
      loading: "Loading layouts\u2026",
    },
    elevatedError: {
      title: "Something went wrong",
      uacHint: "MagicWindows didn't receive administrator rights. This can happen if you closed the UAC prompt or clicked No.",
      generic: "An unexpected error occurred. Retry, and if the problem persists, send us a report.",
      retry: "Retry",
      retrying: "Retrying\u2026",
      sendReport: "Send bug report",
      sending: "Preparing\u2026",
      techDetails: "Technical details",
    },
    bugReport: {
      subject: "[MagicWindows Bug] {op} failed",
      bodyIntro: "Hello, an error occurred while running MagicWindows. Full diagnostics below \u2014 feel free to add context above.",
    },
```

And extend the existing `done` section:

```ts
      removeWindowsDefault: "Windows still has its default {locale} layout installed. Removing it prevents confusion when switching via Win+Space.",
      removeWindowsDefaultBtn: "Remove the Windows default {locale} layout",
      windowsDefaultRemoved: "Windows layout removed \u2713",
```

**French section additions (same keys, translated):**

```ts
    settings: {
      topbarTitle: "Param\u00e8tres",
      title: "Layouts install\u00e9s",
      subtitle: "V\u00e9rifiez et retirez n'importe quel layout clavier de votre syst\u00e8me.",
      badgeMagic: "MagicWindows",
      badgeSystem: "Syst\u00e8me",
      badgeInUse: "En usage",
      remove: "Retirer",
      reactivateInfo: "Pour r\u00e9activer : Param\u00e8tres Windows \u2192 Heure et langue \u2192 Langue \u2192 [langue] \u2192 Options \u2192 Ajouter un clavier.",
      confirmRemove: "Retirer \u00ab {name} \u00bb ?",
      confirmRemoveInUse: "Ce layout est actuellement utilis\u00e9 par votre session. Le retirer quand m\u00eame ?",
      empty: "Aucun layout clavier trouv\u00e9.",
      loading: "Chargement des layouts\u2026",
    },
    elevatedError: {
      title: "Une erreur est survenue",
      uacHint: "MagicWindows n'a pas re\u00e7u les droits administrateur. Cela peut arriver si vous avez ferm\u00e9 la fen\u00eatre UAC ou cliqu\u00e9 Non.",
      generic: "Une erreur inattendue s'est produite. R\u00e9essayez, et si le probl\u00e8me persiste, envoyez-nous un rapport.",
      retry: "R\u00e9essayer",
      retrying: "Nouvel essai\u2026",
      sendReport: "Envoyer un rapport de bug",
      sending: "Pr\u00e9paration\u2026",
      techDetails: "D\u00e9tails techniques",
    },
    bugReport: {
      subject: "[MagicWindows Bug] {op} failed",
      bodyIntro: "Bonjour, une erreur est survenue lors de l'utilisation de MagicWindows. Diagnostics complets ci-dessous \u2014 n'h\u00e9sitez pas \u00e0 ajouter du contexte au-dessus.",
    },
```

And for `done` (French):

```ts
      removeWindowsDefault: "Windows a encore son layout {locale} standard install\u00e9. Le retirer \u00e9vite toute confusion lors du changement via Win+Space.",
      removeWindowsDefaultBtn: "Retirer le layout Windows {locale} par d\u00e9faut",
      windowsDefaultRemoved: "Layout Windows retir\u00e9 \u2713",
```

- [ ] **Step 2: Verify the i18n loader's interpolation supports `{op}`, `{name}`, `{locale}`**

Open `src/lib/i18n.ts` and look for the `t()` function. If it already supports `{placeholder}` interpolation (the existing `install.error` key uses `{message}`), no change is needed. If not, extend the existing interpolation to support named placeholders.

Test by running: `npm run check` — expected: no type errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/i18n.ts
git commit -m "feat(i18n): add settings, elevatedError, bugReport keys + done extensions"
```

---

## Task 7: Frontend — `ElevatedErrorPanel` component

**Files:**
- Create: `src/components/ElevatedErrorPanel.svelte`

- [ ] **Step 1: Write the component**

Create `src/components/ElevatedErrorPanel.svelte`:

```svelte
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";

  interface Props {
    error: string | null;
    onRetry: () => Promise<void>;
    operationName: string;
    context?: Record<string, unknown>;
    attemptCount: number;
  }

  let { error, onRetry, operationName, context = {}, attemptCount }: Props = $props();

  let submitting = $state(false);
  let sending = $state(false);

  function classifyError(msg: string): "uac" | "generic" {
    const lower = msg.toLowerCase();
    if (/admin|privilege|uac|access denied|cancell|cancelled|a annul|runas/.test(lower)) {
      return "uac";
    }
    return "generic";
  }

  async function handleRetry() {
    if (submitting) return;
    submitting = true;
    try {
      await onRetry();
    } finally {
      submitting = false;
    }
  }

  async function handleSendReport() {
    if (sending) return;
    sending = true;
    try {
      const diagnostics = await invoke<string>("collect_diagnostics");
      const ctxJson = Object.keys(context).length > 0
        ? `\n\n**Context:**\n\`\`\`json\n${JSON.stringify(context, null, 2)}\n\`\`\``
        : "";
      const intro = t(appState.lang, "bugReport.bodyIntro");
      const subject = t(appState.lang, "bugReport.subject", { op: operationName });
      const body = `${intro}\n\n${diagnostics}${ctxJson}\n\n--- Votre commentaire ---\n\n`;
      const url = `mailto:bug@mindvisionstudio.com?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body)}`;
      const { open } = await import("@tauri-apps/plugin-shell");
      await open(url);
    } catch (e) {
      console.error("Failed to open bug report:", e);
    } finally {
      sending = false;
    }
  }
</script>

{#if error}
  <div class="elevated-error" role="alert">
    <div class="elevated-error__header">
      <span class="elevated-error__icon" aria-hidden="true">&#9888;</span>
      <h3 class="elevated-error__title">{t(appState.lang, "elevatedError.title")}</h3>
    </div>
    <p class="elevated-error__hint">
      {classifyError(error) === "uac"
        ? t(appState.lang, "elevatedError.uacHint")
        : t(appState.lang, "elevatedError.generic")}
    </p>

    <div class="elevated-error__actions">
      <button class="btn btn-primary" onclick={handleRetry} disabled={submitting || sending}>
        {submitting ? t(appState.lang, "elevatedError.retrying") : t(appState.lang, "elevatedError.retry")}
      </button>
      {#if attemptCount >= 2}
        <button class="btn btn-secondary" onclick={handleSendReport} disabled={submitting || sending}>
          &#128231;&nbsp;{sending ? t(appState.lang, "elevatedError.sending") : t(appState.lang, "elevatedError.sendReport")}
        </button>
      {/if}
    </div>

    <details class="elevated-error__details">
      <summary>{t(appState.lang, "elevatedError.techDetails")}</summary>
      <pre class="elevated-error__raw">{error}</pre>
    </details>
  </div>
{/if}

<style>
  .elevated-error {
    width: 100%;
    max-width: 560px;
    margin: 12px auto;
    padding: 16px 18px;
    border: 1px solid var(--color-danger-border, rgba(220, 53, 69, 0.45));
    background: var(--color-danger-bg, rgba(220, 53, 69, 0.06));
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .elevated-error__header {
    display: flex;
    gap: 10px;
    align-items: center;
  }
  .elevated-error__icon {
    font-size: 1.4rem;
    color: var(--color-danger, #dc3545);
  }
  .elevated-error__title {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }
  .elevated-error__hint {
    margin: 0;
    font-size: 0.9rem;
    line-height: 1.4;
    color: var(--color-text-secondary);
  }
  .elevated-error__actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }
  .elevated-error__details {
    margin-top: 4px;
    font-size: 0.8rem;
  }
  .elevated-error__details > summary {
    cursor: pointer;
    color: var(--color-text-secondary);
  }
  .elevated-error__raw {
    margin: 8px 0 0;
    padding: 8px;
    background: var(--color-bg-elevated, rgba(0, 0, 0, 0.08));
    border-radius: 6px;
    font-size: 0.75rem;
    white-space: pre-wrap;
    word-break: break-all;
    max-height: 180px;
    overflow: auto;
  }
</style>
```

- [ ] **Step 2: Verify the component compiles**

Run: `npm run check`
Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/components/ElevatedErrorPanel.svelte
git commit -m "feat(ui): ElevatedErrorPanel component (retry + mailto bug report)"
```

---

## Task 8: Frontend — refactor Install page to use `ElevatedErrorPanel`

**Files:**
- Modify: `src/pages/Install.svelte`

- [ ] **Step 1: Replace the error block + UAC heuristic**

Replace the entire contents of `src/pages/Install.svelte` with:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import ElevatedErrorPanel from "../components/ElevatedErrorPanel.svelte";

  let installing = $state(true);
  let success = $state(false);
  let error = $state<string | null>(null);
  let attemptCount = $state(0);

  async function runInstall() {
    if (!appState.selectedLayoutId) {
      appState.page = "select";
      return;
    }
    installing = true;
    appState.installing = true;
    error = null;
    try {
      await invoke("install_layout", { id: appState.selectedLayoutId });
      success = true;
    } catch (err) {
      console.error("Installation failed:", err);
      error = String(err);
      attemptCount += 1;
    } finally {
      installing = false;
      appState.installing = false;
    }
  }

  async function openSettings() {
    try {
      const { open } = await import("@tauri-apps/plugin-shell");
      await open("ms-settings:regionlanguage");
    } catch (e) {
      console.error("Could not open settings:", e);
    }
  }

  function goDone() {
    appState.page = "test";
  }

  function goBack() {
    appState.page = "preview";
  }

  onMount(runInstall);
</script>

<div class="page">
  <div class="page__header">
    <h1 class="page__title">{t(appState.lang, "install.title")}</h1>
  </div>

  <div class="page__content">
    {#if installing}
      <div class="spinner"></div>
      <p class="text-secondary">{t(appState.lang, "install.installing")}</p>
    {:else if success}
      <div class="status status--success">
        {t(appState.lang, "install.success")}
      </div>

      <p class="text-secondary text-center" style="max-width: 420px;">
        {t(appState.lang, "done.instructions")}
      </p>

      <div class="page__actions">
        <button class="btn btn-secondary" onclick={openSettings}>
          {t(appState.lang, "install.openSettings")}
        </button>
        <button class="btn btn-primary" onclick={goDone}>
          {t(appState.lang, "install.done")}
        </button>
      </div>
    {:else if error}
      <ElevatedErrorPanel
        {error}
        onRetry={runInstall}
        operationName="install_layout"
        context={{ layoutId: appState.selectedLayoutId }}
        {attemptCount}
      />

      <div class="page__actions">
        <button class="btn btn-secondary" onclick={goBack}>
          {t(appState.lang, "common.back")}
        </button>
      </div>
    {/if}
  </div>
</div>
```

- [ ] **Step 2: Verify**

Run: `npm run check`
Expected: no errors.

- [ ] **Step 3: Manual smoke test**

Run: `npm run tauri dev`. Go Welcome → Select → Preview → Install. When the UAC prompt appears, **click No**. Expected:
- Red ElevatedErrorPanel appears
- Says "Une erreur est survenue" with UAC-specific hint
- Shows Retry button only (no bug report yet)
- Click Retry → UAC prompt re-appears
- Click No again → now Retry + Send bug report buttons appear
- Click Send bug report → default mail client opens with pre-filled subject/body

- [ ] **Step 4: Commit**

```bash
git add src/pages/Install.svelte
git commit -m "refactor(install): use ElevatedErrorPanel for retry + bug report"
```

---

## Task 9: Frontend — refactor Done page errors to `ElevatedErrorPanel`

**Files:**
- Modify: `src/pages/Done.svelte`

- [ ] **Step 1: Replace the `modError` block with `ElevatedErrorPanel`**

In `src/pages/Done.svelte`, inside `<script>`:

1. Import `ElevatedErrorPanel`:
   ```ts
   import ElevatedErrorPanel from "../components/ElevatedErrorPanel.svelte";
   ```

2. Add attempt counters next to the existing state:
   ```ts
   let modAttempt = $state(0);
   let uninstallError = $state<string | null>(null);
   let uninstallAttempt = $state(0);
   ```

3. Rework `applyPreset` to update counters and avoid swallowing the error string:
   ```ts
   async function applyPreset() {
     if (selectedPreset === "none") return;
     applying = true;
     modError = null;
     try {
       const toggles = presetToToggles(selectedPreset);
       await invoke("write_scancode_map", { toggles });
       applied = true;
     } catch (err) {
       console.error("write_scancode_map failed:", err);
       modError = String(err);
       modAttempt += 1;
     } finally {
       applying = false;
     }
   }
   ```

4. Rework `uninstall` to track its own error + attempts:
   ```ts
   async function uninstall() {
     if (!appState.selectedLayoutId) return;
     uninstallError = null;
     try {
       await invoke("uninstall_layout", { id: appState.selectedLayoutId });
       appState.selectedLayoutId = null;
       appState.page = "welcome";
     } catch (err) {
       console.error("Uninstall failed:", err);
       uninstallError = String(err);
       uninstallAttempt += 1;
     }
   }
   ```

- [ ] **Step 2: Replace the `{#if modError}` raw div with `<ElevatedErrorPanel>`**

Inside the template, find:

```svelte
{#if modError}
  <div class="status status--error">{modError}</div>
{/if}
```

Replace with:

```svelte
<ElevatedErrorPanel
  error={modError}
  onRetry={applyPreset}
  operationName="write_scancode_map"
  context={{ preset: selectedPreset }}
  attemptCount={modAttempt}
/>
```

Note: `ElevatedErrorPanel` self-hides when `error` is null, so no `{#if}` wrapper is needed.

- [ ] **Step 3: Add an `<ElevatedErrorPanel>` next to the uninstall button for the uninstall path**

Just above the uninstall `<button>` at the bottom of the page, add:

```svelte
<ElevatedErrorPanel
  error={uninstallError}
  onRetry={uninstall}
  operationName="uninstall_layout"
  context={{ layoutId: appState.selectedLayoutId }}
  attemptCount={uninstallAttempt}
/>
```

- [ ] **Step 4: Verify**

Run: `npm run check`
Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add src/pages/Done.svelte
git commit -m "refactor(done): ElevatedErrorPanel for modifier + uninstall errors"
```

---

## Task 10: Frontend — Settings page

**Files:**
- Create: `src/pages/Settings.svelte`

- [ ] **Step 1: Write the page**

Create `src/pages/Settings.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import type { InstalledLayoutInfo } from "../lib/types";
  import ElevatedErrorPanel from "../components/ElevatedErrorPanel.svelte";

  let layouts = $state<InstalledLayoutInfo[]>([]);
  let loading = $state(true);
  let listError = $state<string | null>(null);

  let removeError = $state<string | null>(null);
  let removeAttempt = $state(0);
  let lastRemoveTarget = $state<InstalledLayoutInfo | null>(null);

  async function loadLayouts() {
    loading = true;
    listError = null;
    try {
      layouts = await invoke<InstalledLayoutInfo[]>("list_all_installed_layouts");
    } catch (err) {
      console.error("Failed to list layouts:", err);
      listError = String(err);
    } finally {
      loading = false;
    }
  }

  async function doRemove(layout: InstalledLayoutInfo) {
    removeError = null;
    try {
      if (layout.isMagicWindows) {
        const id = layout.layoutFile.toLowerCase().replace(/\.dll$/, "");
        // MagicWindows DLLs are named kbdapl<locale>.dll; we don't have a
        // perfect reverse-lookup from file to layout id, so use uninstall_by_klid
        // which handles both paths identically at the registry level.
        await invoke("uninstall_by_klid", { klid: layout.klid });
      } else {
        await invoke("uninstall_by_klid", { klid: layout.klid });
      }
      await loadLayouts();
    } catch (err) {
      console.error("Remove failed:", err);
      removeError = String(err);
      removeAttempt += 1;
    }
  }

  async function retryLastRemove() {
    if (lastRemoveTarget) {
      await doRemove(lastRemoveTarget);
    }
  }

  async function requestRemove(layout: InstalledLayoutInfo) {
    const key = layout.isInUse ? "settings.confirmRemoveInUse" : "settings.confirmRemove";
    const msg = t(appState.lang, key, { name: layout.layoutText || layout.klid });
    if (!window.confirm(msg)) return;
    lastRemoveTarget = layout;
    removeAttempt = 0;
    await doRemove(layout);
  }

  function goBack() {
    appState.page = "welcome";
  }

  onMount(loadLayouts);
</script>

<div class="page">
  <div class="page__header">
    <h1 class="page__title">{t(appState.lang, "settings.title")}</h1>
    <p class="page__subtitle">{t(appState.lang, "settings.subtitle")}</p>
  </div>

  <div class="page__content">
    <ElevatedErrorPanel
      error={removeError}
      onRetry={retryLastRemove}
      operationName="uninstall_by_klid"
      context={{ klid: lastRemoveTarget?.klid, layoutFile: lastRemoveTarget?.layoutFile }}
      attemptCount={removeAttempt}
    />

    {#if loading}
      <div class="spinner"></div>
      <p class="text-secondary">{t(appState.lang, "settings.loading")}</p>
    {:else if listError}
      <div class="status status--error">{listError}</div>
    {:else if layouts.length === 0}
      <p class="text-secondary text-center">{t(appState.lang, "settings.empty")}</p>
    {:else}
      <div class="layout-list">
        {#each layouts as layout (layout.klid)}
          <div class="layout-row">
            <div class="layout-row__main">
              <div class="layout-row__title">{layout.layoutText || layout.klid}</div>
              <div class="layout-row__meta">
                <code>{layout.layoutFile}</code>
                <span class="layout-row__dot">·</span>
                <code>{layout.klid}</code>
              </div>
              <div class="layout-row__badges">
                {#if layout.isMagicWindows}
                  <span class="badge badge--magic">{t(appState.lang, "settings.badgeMagic")}</span>
                {:else if layout.klid.startsWith("0000")}
                  <span class="badge badge--system">{t(appState.lang, "settings.badgeSystem")}</span>
                {/if}
                {#if layout.isInUse}
                  <span class="badge badge--in-use">{t(appState.lang, "settings.badgeInUse")}</span>
                {/if}
              </div>
            </div>
            <div class="layout-row__actions">
              <button
                class="btn btn-danger btn-sm"
                onclick={() => requestRemove(layout)}
              >
                {t(appState.lang, "settings.remove")}
              </button>
              <span
                class="info-icon"
                title={t(appState.lang, "settings.reactivateInfo")}
                aria-label={t(appState.lang, "settings.reactivateInfo")}
              >&#9432;</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <div class="page__actions">
      <button class="btn btn-secondary" onclick={goBack}>
        {t(appState.lang, "common.back")}
      </button>
    </div>
  </div>
</div>

<style>
  .layout-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    max-width: 640px;
    margin: 0 auto;
  }
  .layout-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 14px;
    background: var(--color-bg-elevated, rgba(255,255,255,0.04));
    border: 1px solid var(--color-border, rgba(0,0,0,0.15));
    border-radius: 8px;
  }
  .layout-row__main { display: flex; flex-direction: column; gap: 4px; min-width: 0; flex: 1; }
  .layout-row__title { font-weight: 600; }
  .layout-row__meta { font-size: 0.78rem; color: var(--color-text-secondary); display: flex; gap: 6px; align-items: center; }
  .layout-row__meta code { font-size: 0.78rem; }
  .layout-row__dot { opacity: 0.5; }
  .layout-row__badges { display: flex; gap: 6px; margin-top: 2px; flex-wrap: wrap; }
  .badge {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 999px;
    font-size: 0.7rem;
    font-weight: 500;
    white-space: nowrap;
  }
  .badge--magic { background: rgba(59, 130, 246, 0.15); color: #3b82f6; }
  .badge--system { background: rgba(156, 163, 175, 0.15); color: #6b7280; }
  .badge--in-use { background: rgba(34, 197, 94, 0.15); color: #22c55e; }
  .layout-row__actions { display: flex; align-items: center; gap: 8px; }
  .info-icon {
    cursor: help;
    color: var(--color-text-secondary);
    font-size: 1.05rem;
  }
  .btn-sm { padding: 4px 10px; font-size: 0.85rem; }
  .btn-danger { background: #dc3545; color: white; border-color: #dc3545; }
  .btn-danger:hover { background: #c82333; }
</style>
```

- [ ] **Step 2: Verify**

Run: `npm run check`
Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/pages/Settings.svelte
git commit -m "feat(ui): Settings page — list and remove installed layouts"
```

---

## Task 11: Frontend — App.svelte routing + gear top-bar button

**Files:**
- Modify: `src/App.svelte`

- [ ] **Step 1: Import and route**

In the `<script>` block of `src/App.svelte`, add the import:

```ts
  import Settings from "./pages/Settings.svelte";
```

Then extend the page-router chain in the template. After the `{:else if appState.page === "modifiers"}<Modifiers />` line, add:

```svelte
{:else if appState.page === "settings"}
  <Settings />
```

- [ ] **Step 2: Add the ⚙ top-bar button**

In the `<div class="top-bar__controls">` block, insert a new button **before** the ⌘ modifiers button so the order is Settings → Modifiers → Theme → Lang:

```svelte
    <button
      class="theme-toggle"
      onclick={() => (appState.page = "settings")}
      title={t(appState.lang, "settings.topbarTitle")}
      aria-label={t(appState.lang, "settings.topbarTitle")}
    >
      &#9881;
    </button>
```

- [ ] **Step 3: Verify + manual test**

Run: `npm run check`
Expected: no errors.

Run: `npm run tauri dev`. Click the ⚙ icon in the top-bar. Expected: Settings page opens, lists at least `0000040c` and any MagicWindows layouts installed. Click Remove on a MagicWindows one → confirmation → UAC → success → row disappears.

- [ ] **Step 4: Commit**

```bash
git add src/App.svelte
git commit -m "feat(app): settings route + gear top-bar button"
```

---

## Task 12: Frontend — Done page "Remove Windows default" bloc

**Files:**
- Modify: `src/pages/Done.svelte`

- [ ] **Step 1: Extend `<script>` with detection + removal**

Inside `src/pages/Done.svelte`'s `<script>` block (already imports `invoke`, `appState`, `t`, `ElevatedErrorPanel` from Task 9), add:

```ts
  import { onMount } from "svelte";
  import type { InstalledLayoutInfo } from "../lib/types";

  let installedSelectedLayout = $state<InstalledLayoutInfo | null>(null);
  let installedWindowsDefault = $state<InstalledLayoutInfo | null>(null);
  let windowsDefaultRemoved = $state(false);

  let removeWinError = $state<string | null>(null);
  let removeWinAttempt = $state(0);

  async function detectWindowsDefault() {
    try {
      const layouts = await invoke<InstalledLayoutInfo[]>("list_all_installed_layouts");
      // Find our just-installed MagicWindows layout (matches selectedLayoutId).
      const ours = layouts.find(
        (l) => l.isMagicWindows && appState.selectedLayoutId && l.layoutFile.toLowerCase().includes(appState.selectedLayoutId.replace("apple-", "").replace(/-.*/, "")),
      );
      installedSelectedLayout = ours ?? null;
      // The locale hex is the last 4 chars of the KLID. Find the system default
      // (klid starts with "0000") with the same locale hex.
      if (ours) {
        const localeHex = ours.klid.slice(-4).toLowerCase();
        installedWindowsDefault =
          layouts.find((l) => l.klid.startsWith("0000") && l.klid.slice(-4).toLowerCase() === localeHex) ?? null;
      }
    } catch (err) {
      console.error("detectWindowsDefault failed:", err);
    }
  }

  async function removeWindowsDefault() {
    if (!installedWindowsDefault) return;
    const msg = t(appState.lang, "settings.confirmRemove", { name: installedWindowsDefault.layoutText || installedWindowsDefault.klid });
    if (!window.confirm(msg)) return;
    removeWinError = null;
    try {
      await invoke("uninstall_by_klid", { klid: installedWindowsDefault.klid });
      windowsDefaultRemoved = true;
      installedWindowsDefault = null;
    } catch (err) {
      console.error("Remove Windows default failed:", err);
      removeWinError = String(err);
      removeWinAttempt += 1;
    }
  }

  async function retryRemoveWindowsDefault() {
    await removeWindowsDefault();
  }

  onMount(detectWindowsDefault);
```

- [ ] **Step 2: Add the bloc to the template**

In the template of `Done.svelte`, between the success banner/instructions and the existing "mod-offer" bloc, add:

```svelte
    <!-- ── Optional: remove the Windows default layout for this locale ── -->
    {#if installedWindowsDefault}
      <div class="win-default-offer">
        <h2 class="win-default-offer__title">
          {t(appState.lang, "done.removeWindowsDefault", { locale: installedWindowsDefault.layoutText })}
          <span
            class="info-icon"
            title={t(appState.lang, "settings.reactivateInfo")}
            aria-label={t(appState.lang, "settings.reactivateInfo")}
          >&#9432;</span>
        </h2>
        <button class="btn btn-secondary" onclick={removeWindowsDefault}>
          {t(appState.lang, "done.removeWindowsDefaultBtn", { locale: installedWindowsDefault.layoutText })}
        </button>
        <ElevatedErrorPanel
          error={removeWinError}
          onRetry={retryRemoveWindowsDefault}
          operationName="uninstall_by_klid"
          context={{ klid: installedWindowsDefault.klid }}
          attemptCount={removeWinAttempt}
        />
      </div>
    {:else if windowsDefaultRemoved}
      <div class="status status--success" style="max-width: 460px;">
        {t(appState.lang, "done.windowsDefaultRemoved")}
      </div>
    {/if}
```

- [ ] **Step 3: Add minimal styling at the end of the `<style>` block**

```css
  .win-default-offer {
    width: 100%;
    max-width: 480px;
    margin: 12px auto 4px;
    padding: 14px 16px;
    box-sizing: border-box;
    background: var(--color-bg-elevated, rgba(255,255,255,0.04));
    border: 1px solid var(--color-border, rgba(0,0,0,0.15));
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .win-default-offer__title {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .info-icon { cursor: help; color: var(--color-text-secondary); }
```

- [ ] **Step 4: Verify + manual test**

Run: `npm run check` → no errors.

Run: `npm run tauri dev`. Install FR layout → navigate to Done. Expected: the "Layout Windows détecté" bloc appears (because `0000040c` exists). Click Retirer → confirm → UAC approve → bloc replaces itself with "Layout Windows retiré ✓". Reopen Settings to verify `0000040c` is gone.

- [ ] **Step 5: Commit**

```bash
git add src/pages/Done.svelte
git commit -m "feat(done): offer removal of Windows default layout for same locale"
```

---

## Task 13: Build — keep NSIS + add MSI + portable ZIP

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Create: `scripts/build-portable.mjs`
- Modify: `package.json`

- [ ] **Step 1: Add MSI to the bundle targets**

In `src-tauri/tauri.conf.json`, change:

```json
    "targets": ["nsis"],
```

to:

```json
    "targets": ["nsis", "msi"],
```

- [ ] **Step 2: Create `scripts/build-portable.mjs`**

```javascript
#!/usr/bin/env node
/**
 * Post-build helper: zip the release exe + its bundled resources into a
 * portable archive. The output ZIP is self-contained — the user unzips it,
 * runs `magicwindows.exe`, and the app finds `kbd_dlls/` and `layouts/`
 * via resource_dir() (same directory as the exe).
 *
 * Run AFTER `npm run tauri build` (or chain via `npm run build:portable`).
 */
import { readdirSync, statSync, mkdirSync, existsSync } from "node:fs";
import { dirname, join, basename } from "node:path";
import { fileURLToPath } from "node:url";
import { execSync } from "node:child_process";
import archiver from "archiver";
import { createWriteStream } from "node:fs";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = dirname(here);

const releaseDir = join(repoRoot, "src-tauri", "target", "release");
const exePath = join(releaseDir, "magicwindows.exe");
const kbdDllsDir = join(repoRoot, "target", "kbd_dlls");
const layoutsDir = join(repoRoot, "layouts");

const bundleDir = join(releaseDir, "bundle", "portable");
mkdirSync(bundleDir, { recursive: true });

// Read app version from package.json for the ZIP filename.
const pkg = JSON.parse(
  execSync("node -p \"JSON.stringify(require('./package.json'))\"", {
    cwd: repoRoot,
  }).toString(),
);
const version = pkg.version || "dev";
const outZip = join(bundleDir, `MagicWindows_${version}_portable.zip`);

for (const p of [exePath, kbdDllsDir, layoutsDir]) {
  if (!existsSync(p)) {
    console.error(`[build-portable] Missing required input: ${p}`);
    process.exit(1);
  }
}

console.log(`[build-portable] Creating ${outZip}`);

const output = createWriteStream(outZip);
const archive = archiver("zip", { zlib: { level: 9 } });

output.on("close", () => {
  const mb = (archive.pointer() / 1024 / 1024).toFixed(2);
  console.log(`[build-portable] Done: ${outZip} (${mb} MB)`);
});
archive.on("error", (err) => { throw err; });
archive.pipe(output);

// Add the main exe at the root of the archive.
archive.file(exePath, { name: "magicwindows.exe" });

// Add kbd_dlls/ (only the .dll files, no .exp/.lib).
for (const file of readdirSync(kbdDllsDir)) {
  if (file.endsWith(".dll")) {
    archive.file(join(kbdDllsDir, file), { name: `kbd_dlls/${file}` });
  }
}

// Add layouts/ (all .json files including schema.json — harmless and useful).
for (const file of readdirSync(layoutsDir)) {
  if (file.endsWith(".json")) {
    archive.file(join(layoutsDir, file), { name: `layouts/${file}` });
  }
}

// Add a minimal README explaining the portable layout.
const readme = `MagicWindows portable — version ${version}

Usage:
  1. Keep magicwindows.exe next to the kbd_dlls/ and layouts/ folders.
  2. Double-click magicwindows.exe to run.
  3. The app will ask for administrator rights (UAC) to install the
     selected keyboard layout into the Windows system directory.

To uninstall a layout afterwards, re-launch the exe and use the
"Settings" button (top bar, gear icon).

Requires Microsoft Edge WebView2 (present by default on Windows 10/11).
`;
archive.append(readme, { name: "README.txt" });

await archive.finalize();
```

- [ ] **Step 3: Install `archiver` as a dev dependency**

Run: `npm install --save-dev archiver`

- [ ] **Step 4: Add the npm script**

In `package.json`, inside `"scripts"`, after `"tauri": "tauri"` add:

```json
    "build:portable": "npm run tauri build && node scripts/build-portable.mjs"
```

- [ ] **Step 5: Verify**

Run: `npm run build:portable`
Expected:
- NSIS + MSI installers produced in `src-tauri/target/release/bundle/nsis/` and `…/msi/`
- Portable ZIP in `src-tauri/target/release/bundle/portable/MagicWindows_0.1.3_portable.zip`
- ZIP contains `magicwindows.exe`, `kbd_dlls/*.dll`, `layouts/*.json`, `README.txt`

Spot-check by unzipping to a fresh directory and running the exe there — it should find resources correctly.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/tauri.conf.json scripts/build-portable.mjs package.json package-lock.json
git commit -m "build: add MSI target + portable ZIP post-build script"
```

---

## Task 14: Final manual QA checklist

No code changes. Run through this list in order to validate the full feature end-to-end:

- [ ] **QA 1 — Fresh install happy path**
  Build: `npm run build:portable`. Install the NSIS output. Launch. Install FR layout → UAC approve → success. Done page shows both the "Layout Windows détecté" bloc AND the modifier-preset offer.

- [ ] **QA 2 — Remove Windows default**
  On Done, click "Retirer le layout Windows FR" → confirm → UAC approve → bloc flips to "Layout Windows retiré ✓". Open Settings → verify `0000040c` is gone.

- [ ] **QA 3 — UAC cancel → retry → bug report**
  Uninstall MagicWindows. Relaunch. Install FR → when UAC prompt appears, click No. Expected:
  - ElevatedErrorPanel shows with UAC hint + Retry button (no bug report yet)
  - Click Retry → UAC prompt reappears. Click No again.
  - Now Retry + Send bug report buttons visible.
  - Click Send bug report → default email client opens with `bug@mindvisionstudio.com`, subject `[MagicWindows Bug] install_layout failed`, body contains diagnostics Markdown.

- [ ] **QA 4 — Settings page**
  Open ⚙ Settings. Verify: every HKLM layout is listed with correct badges. Remove a MagicWindows layout → DLL disappears from `C:\Windows\System32\`. Remove a system layout (e.g., `en-US`) → registry entry gone, DLL in System32 still present.

- [ ] **QA 5 — Portable ZIP**
  Extract `MagicWindows_0.1.3_portable.zip` to `C:\Users\<you>\Downloads\magicwindows-portable\`. Run `magicwindows.exe` from there. Verify full install flow works (layouts are resolved from the adjacent `kbd_dlls/` and `layouts/` folders).

- [ ] **QA 6 — Chromium compat (regression check of the dcaba05 fix)**
  With our FR layout active, open VSCode and Antigravity. Press ENTER in a text area → newline. Press Backspace → deletes last char. No 'm' insertion. This verifies the control-key wchar tables are still working.

Commit once all QA steps pass:

```bash
# No file changes; just a chore commit documenting the QA pass
git commit --allow-empty -m "chore: manual QA pass for settings + error flow + portable"
```

---

## Self-review addendum (post-plan)

**Spec coverage:** ✅
- Settings page — Tasks 1–5, 10, 11
- Done removal bloc — Tasks 5, 6, 12
- ElevatedErrorPanel — Tasks 6, 7, 8, 9, 10, 12
- list_all_keyboard_layouts — Task 2
- uninstall_layout_by_klid — Task 3 (named `uninstall_by_klid` in code for brevity)
- collect_diagnostics — Task 4
- i18n — Task 6
- Top-bar gear — Task 11
- Edge cases (confirm-in-use, system DLL protection, truncate transcript) — covered in the command + panel code
- Portable exe — Task 13 (addition requested post-brainstorm)

**Type consistency:** ✅ `InstalledLayoutInfo` uses identical camelCase fields in Rust `#[serde(rename_all = "camelCase")]` and TS interface. `uninstall_by_klid` takes a single `klid: String` arg — consumed the same way from both Settings and Done.

**Placeholder scan:** no "TBD", "TODO", or hand-waved steps. Every code step contains the actual code to paste.
