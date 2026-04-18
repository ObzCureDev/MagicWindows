pub mod detect;
pub mod install;
#[allow(dead_code)]
pub mod kbd_c;
pub mod klc;
pub mod scancode_map;
pub mod modifiers;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Minimal metadata for listing layouts in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutMeta {
    pub id: String,
    pub name: HashMap<String, String>,
    pub locale: String,
    pub description: HashMap<String, String>,
}

/// A single detection-key entry used by the auto-detection UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionKey {
    pub event_code: String,
    pub prompt: HashMap<String, String>,
    pub expected_base: String,
}

/// One key mapping row (values are hex-codepoint strings or "-1").
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyMapping {
    pub vk: String,
    pub cap: String,
    pub base: String,
    pub shift: String,
    #[serde(default)]
    pub ctrl: String,
    #[serde(default)]
    pub altgr: String,
    #[serde(default, rename = "altgrShift")]
    pub altgr_shift: String,
}

/// A dead-key definition (base char -> composed result).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadKey {
    pub name: String,
    pub combinations: HashMap<String, String>,
}

/// Full layout as stored in JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layout {
    pub id: String,
    pub name: HashMap<String, String>,
    pub locale: String,
    pub locale_id: String,
    pub dll_name: String,
    #[serde(default)]
    pub description: HashMap<String, String>,
    pub detection_keys: Vec<DetectionKey>,
    pub keys: HashMap<String, KeyMapping>,
    #[serde(default)]
    pub dead_keys: HashMap<String, DeadKey>,
}

/// Result from one key-press during auto-detection (sent by the frontend).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionResult {
    pub event_code: String,
    pub received_char: String,
}

impl Layout {
    /// Build a `LayoutMeta` from the full layout.
    pub fn meta(&self) -> LayoutMeta {
        LayoutMeta {
            id: self.id.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            description: self.description.clone(),
        }
    }

    /// Validate layout invariants that match the JSON schema constraints.
    pub fn validate(&self) -> Result<(), String> {
        // ID must match apple-{xx}-{type}
        if !self.id.starts_with("apple-") || self.id.len() < 10 {
            return Err(format!("Layout ID '{}' must start with 'apple-' and include language and type", self.id));
        }

        // DLL name max 8 chars, lowercase alphanumeric
        if self.dll_name.is_empty() || self.dll_name.len() > 8 {
            return Err(format!("DLL name '{}' must be 1-8 characters", self.dll_name));
        }
        if !self.dll_name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()) {
            return Err(format!("DLL name '{}' must be lowercase alphanumeric", self.dll_name));
        }

        // Detection keys: 3-5
        if self.detection_keys.len() < 3 || self.detection_keys.len() > 5 {
            return Err(format!("Layout '{}' must have 3-5 detection keys, has {}", self.id, self.detection_keys.len()));
        }

        // Must have at least one key mapping
        if self.keys.is_empty() {
            return Err(format!("Layout '{}' has no key mappings", self.id));
        }

        // Locale must be xx-XX format
        if self.locale.len() != 5 || self.locale.as_bytes()[2] != b'-' {
            return Err(format!("Layout '{}' locale '{}' must be xx-XX format", self.id, self.locale));
        }

        // Locale ID must be 8 hex chars
        if self.locale_id.len() != 8 || !self.locale_id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("Layout '{}' locale_id '{}' must be 8 hex chars", self.id, self.locale_id));
        }

        Ok(())
    }
}

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
