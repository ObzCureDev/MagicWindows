//! Microsoft Scancode Map binary serialization & parsing.
//!
//! The `HKLM\System\CurrentControlSet\Control\Keyboard Layout\Scancode Map`
//! registry value is a `REG_BINARY` whose layout is:
//!   - 8 bytes header (all zeros)
//!   - 4 bytes little-endian count = (number of mapping entries) + 1 for the terminator
//!   - N × 4 bytes mappings: 2 bytes "new" scancode + 2 bytes "old" scancode (little-endian)
//!   - 4 bytes null terminator
//!
//! A scancode is 2 bytes: low byte first, then high byte (00 = normal, E0 = extended).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifierToggles {
    pub swap_cmd_ctrl_left:  bool,
    pub swap_cmd_ctrl_right: bool,
    pub caps_to_ctrl:        bool,
    pub swap_option_cmd:     bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawScancodePair {
    pub new_code: String, // 4 hex chars, little-endian (e.g. "1D00")
    pub old_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifierState {
    pub current: ModifierToggles,
    pub has_external_mappings: bool,
    pub raw_entries: Vec<RawScancodePair>,
}

// Stubs — implementations land in tasks 2 and 3.
pub fn build_scancode_map(_toggles: &ModifierToggles) -> Vec<u8> {
    unimplemented!("Task 2")
}

pub fn parse_scancode_map(_bytes: &[u8]) -> Result<Vec<RawScancodePair>, String> {
    unimplemented!("Task 3")
}

pub fn derive_state(_pairs: &[RawScancodePair]) -> ModifierState {
    unimplemented!("Task 3")
}
