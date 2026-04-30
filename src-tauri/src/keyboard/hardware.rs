//! Apple keyboard HID enumeration.
//!
//! Used by:
//!   - the Bluetooth Pairing Assistant on Welcome (decides whether to
//!     surface the pairing flow)
//!   - the same flow's polling watcher (waits for a new device to show up
//!     after the user opens the Windows Bluetooth panel)
//!
//! Apple's USB Vendor ID is 0x05AC. We list any "Keyboard" PnP device whose
//! HardwareID contains `VID_05AC`. Bluetooth and USB Apple keyboards both
//! match this filter on Windows 10/11.

use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppleKeyboardInfo {
    /// Friendly name shown in Device Manager (e.g. "Magic Keyboard").
    pub friendly_name: String,
    /// Hardware ID string (e.g. `BTHENUM\Dev_F0F61C123456&...`).
    pub hardware_id: String,
    /// `OK` when the device is currently usable.
    pub status: String,
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn enumerate_apple_keyboards() -> Result<Vec<AppleKeyboardInfo>, String> {
    use std::process::Command;

    let script = r#"
$ErrorActionPreference = 'Stop'
$devices = Get-PnpDevice -Class Keyboard -PresentOnly -ErrorAction SilentlyContinue |
    Where-Object { ($_.HardwareID -join ';') -match 'VID_05AC' } |
    Select-Object FriendlyName, @{Name='HardwareID';Expression={($_.HardwareID -join ';')}}, Status
ConvertTo-Json -Depth 3 -Compress -InputObject @($devices)
"#;

    let output = Command::new("powershell")
        .args(["-ExecutionPolicy", "Bypass", "-NoProfile", "-Command", script])
        .output()
        .map_err(|e| format!("spawn powershell: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "enumerate failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json = String::from_utf8_lossy(&output.stdout);
    let trimmed = json.trim();
    if trimmed.is_empty() || trimmed == "null" {
        return Ok(vec![]);
    }
    parse_pnp_output(trimmed)
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn enumerate_apple_keyboards() -> Result<Vec<AppleKeyboardInfo>, String> {
    Ok(vec![])
}

/// Pure parser for the JSON shape PowerShell emits. Extracted so it's
/// unit-testable without spawning PowerShell.
fn parse_pnp_output(json: &str) -> Result<Vec<AppleKeyboardInfo>, String> {
    let v: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| format!("parse pnp output: {e}\nraw: {json}"))?;

    // ConvertTo-Json on a single object emits an object; on an array, an
    // array. Wrap a single object into a one-element array.
    let arr = match v {
        serde_json::Value::Array(a) => a,
        serde_json::Value::Object(_) => vec![v],
        _ => return Ok(vec![]),
    };

    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        out.push(AppleKeyboardInfo {
            friendly_name: item["FriendlyName"].as_str().unwrap_or("").to_string(),
            hardware_id: item["HardwareID"].as_str().unwrap_or("").to_string(),
            status: item["Status"].as_str().unwrap_or("").to_string(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_array_of_devices() {
        let json = r#"[{"FriendlyName":"Magic Keyboard","HardwareID":"BTHENUM\\Dev_VID_05AC","Status":"OK"}]"#;
        let r = parse_pnp_output(json).unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].friendly_name, "Magic Keyboard");
    }

    #[test]
    fn parses_single_device_object() {
        // PowerShell's ConvertTo-Json collapses a 1-element array to an object.
        let json = r#"{"FriendlyName":"Magic Keyboard","HardwareID":"BTHENUM\\Dev_VID_05AC","Status":"OK"}"#;
        let r = parse_pnp_output(json).unwrap();
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn missing_fields_default_to_empty_string() {
        let json = r#"[{"FriendlyName":null}]"#;
        let r = parse_pnp_output(json).unwrap();
        assert_eq!(r[0].friendly_name, "");
        assert_eq!(r[0].hardware_id, "");
    }

    #[test]
    fn null_top_level_yields_empty_vec() {
        // serde parses "null" as Value::Null, which our match catches.
        let r = parse_pnp_output("null").unwrap();
        assert!(r.is_empty());
    }
}
