//! Compose a human-readable Markdown diagnostics block for bug reports.
//! Every collector is best-effort: failures inline a marker but never bubble
//! up — the user must always get *some* report to send.

#[cfg(target_os = "windows")]
use super::InstalledLayoutInfo;
#[cfg(target_os = "windows")]
use std::process::Command;

/// Tauri command: return a pre-formatted Markdown block suitable for pasting
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

    // Active HKL via a P/Invoke to GetKeyboardLayout(0).
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
            let mw: Vec<&InstalledLayoutInfo> =
                layouts.iter().filter(|l| l.is_magic_windows).collect();
            if mw.is_empty() {
                out.push_str("- *(none)*\n");
            } else {
                for l in &mw {
                    out.push_str(&format!(
                        "- `{}` / `{}` / \"{}\"\n",
                        l.klid, l.layout_file, l.layout_text
                    ));
                }
            }
            out.push('\n');

            out.push_str("**All installed keyboard layouts:**\n");
            for l in &layouts {
                out.push_str(&format!(
                    "- `{}` / `{}` / \"{}\"\n",
                    l.klid, l.layout_file, l.layout_text
                ));
            }
            out.push('\n');
        }
        Err(e) => {
            out.push_str(&format!(
                "**Installed layouts:** *(collector failed: {e})*\n\n"
            ));
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
                    out.push_str(&format!(
                        "- new=`{}` old=`{}`\n",
                        pair.new_code, pair.old_code
                    ));
                }
            }
            out.push('\n');
        }
        Err(e) => {
            out.push_str(&format!(
                "**Scancode Map:** *(collector failed: {e})*\n\n"
            ));
        }
    }

    // Last PS transcript tail.
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
