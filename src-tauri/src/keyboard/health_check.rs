//! Control-key regression probe.
//!
//! Loads the **target** keyboard layout via `LoadKeyboardLayoutW(klid, …)`
//! and calls `ToUnicodeEx` for VK_RETURN, Shift+VK_RETURN, VK_TAB, VK_BACK,
//! VK_ESCAPE against that exact HKL. Each one must produce no `wchar` output
//! — if it does, Chromium-based apps (Antigravity, VSCode, Slack…) will
//! swallow the keystroke as text input. This is the regression that broke
//! Shift+Enter before commit a8f8b7f.
//!
//! We deliberately accept the target KLID as a parameter rather than calling
//! `GetKeyboardLayout(0)`: the active foreground layout in the PowerShell
//! child process is not guaranteed to be the MagicWindows layout the user is
//! testing.

use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ControlKeyResult {
    pub name: String, // "Enter", "Shift+Enter", ...
    pub vk: u8,
    pub shift: bool,
    pub passed: bool,
    /// hex representation of what ToUnicodeEx produced (empty = no output)
    pub produced: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ControlKeyReport {
    /// Echo of the KLID that was loaded for the probe.
    pub klid: String,
    pub results: Vec<ControlKeyResult>,
    pub all_passed: bool,
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn health_check_control_keys(klid: String) -> Result<ControlKeyReport, String> {
    use std::process::Command;

    // Sanity-check the KLID before injecting it into a PowerShell string.
    // KLIDs are 8 hex characters (the MagicWindows ones start with 'a').
    if klid.len() != 8 || !klid.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("invalid KLID: {klid:?}"));
    }

    // We mirror the existing pattern in diagnostics.rs: shell out to
    // PowerShell with an inline P/Invoke. Avoids bringing in `windows-rs`
    // for a single call.
    let script = format!(
        r#"
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public class Probe {{
  [DllImport("user32.dll", CharSet=CharSet.Unicode)]
  public static extern IntPtr LoadKeyboardLayout(string klid, uint flags);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)]
  public static extern int ToUnicodeEx(uint vk, uint sc, byte[] state, StringBuilder buf, int sz, uint flags, IntPtr hkl);
  [DllImport("user32.dll")]
  public static extern uint MapVirtualKeyEx(uint code, uint mapType, IntPtr hkl);

  public static string Probe(IntPtr hkl, uint vk, bool shift) {{
    // MAPVK_VK_TO_VSC_EX = 4 (returns the scancode, including extended bit)
    uint sc = MapVirtualKeyEx(vk, 4, hkl);
    var state = new byte[256];
    if (shift) state[0x10] = 0x80;
    var buf = new StringBuilder(8);
    int n = ToUnicodeEx(vk, sc, state, buf, buf.Capacity, 0, hkl);
    return n > 0 ? buf.ToString() : "";
  }}
}}
'@

# KLF_NOTELLSHELL = 0x80, KLF_SUBSTITUTE_OK = 0x02. Loads without changing
# the active foreground layout for the calling thread.
$hkl = [Probe]::LoadKeyboardLayout('{klid}', 0x82)
if ($hkl -eq [IntPtr]::Zero) {{
  Write-Error "LoadKeyboardLayout failed for KLID '{klid}'"
  exit 1
}}

$o = New-Object PSObject -Property @{{
  klid = '{klid}'
  enter = [Probe]::Probe($hkl, 0x0D, $false)
  shiftEnter = [Probe]::Probe($hkl, 0x0D, $true)
  tab = [Probe]::Probe($hkl, 0x09, $false)
  back = [Probe]::Probe($hkl, 0x08, $false)
  esc = [Probe]::Probe($hkl, 0x1B, $false)
}}
$o | ConvertTo-Json -Compress
"#
    );

    let output = Command::new("powershell")
        .args([
            "-ExecutionPolicy",
            "Bypass",
            "-NoProfile",
            "-Command",
            &script,
        ])
        .output()
        .map_err(|e| format!("spawn powershell: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "probe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(json.trim())
        .map_err(|e| format!("parse probe output: {e}\nraw: {json}"))?;

    let mk = |name: &str, vk: u8, shift: bool, key: &str| {
        let produced = parsed[key].as_str().unwrap_or("");
        // PASS = no wchar produced. Any character (including 0x000A or
        // 0x000D) is a fail because Chromium will treat it as text input.
        let passed = produced.is_empty();
        let produced_hex = produced
            .chars()
            .map(|c| format!("{:04X}", c as u32))
            .collect::<Vec<_>>()
            .join(",");
        ControlKeyResult {
            name: name.into(),
            vk,
            shift,
            passed,
            produced: produced_hex,
        }
    };

    let results = vec![
        mk("Enter", 0x0D, false, "enter"),
        mk("Shift+Enter", 0x0D, true, "shiftEnter"),
        mk("Tab", 0x09, false, "tab"),
        mk("Backspace", 0x08, false, "back"),
        mk("Escape", 0x1B, false, "esc"),
    ];
    let all_passed = results.iter().all(|r| r.passed);

    Ok(ControlKeyReport {
        klid,
        results,
        all_passed,
    })
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn health_check_control_keys(_klid: String) -> Result<ControlKeyReport, String> {
    Err("health_check is only available on Windows".into())
}
