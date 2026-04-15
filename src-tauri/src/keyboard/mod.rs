pub mod detect;
pub mod install;
pub mod klc;

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
}
