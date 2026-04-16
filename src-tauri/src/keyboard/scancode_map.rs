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

// ── Scancode constants (low byte, high byte) — high byte 0xE0 marks "extended" ───
const LCTRL: [u8; 2] = [0x1D, 0x00];
const RCTRL: [u8; 2] = [0x1D, 0xE0];
const LWIN:  [u8; 2] = [0x5B, 0xE0];
const RWIN:  [u8; 2] = [0x5C, 0xE0];
const LALT:  [u8; 2] = [0x38, 0x00];
const RALT:  [u8; 2] = [0x38, 0xE0];
const CAPS:  [u8; 2] = [0x3A, 0x00];

/// Encode a single mapping: pressing `old` causes the OS to report `new`.
fn entry(new: [u8; 2], old: [u8; 2]) -> [u8; 4] {
    [new[0], new[1], old[0], old[1]]
}

/// Build the binary Scancode Map value from the user's toggle selection.
/// Returns an EMPTY vec when no toggles are active — the caller should DELETE
/// the registry value rather than write a header-only blob.
pub fn build_scancode_map(toggles: &ModifierToggles) -> Vec<u8> {
    let mut entries: Vec<[u8; 4]> = Vec::new();

    if toggles.swap_cmd_ctrl_left {
        entries.push(entry(LCTRL, LWIN));   // LWin pressed → LCtrl emitted
        entries.push(entry(LWIN,  LCTRL));  // LCtrl pressed → LWin emitted
    }
    if toggles.swap_cmd_ctrl_right {
        entries.push(entry(RCTRL, RWIN));
        entries.push(entry(RWIN,  RCTRL));
    }
    if toggles.caps_to_ctrl {
        entries.push(entry(LCTRL, CAPS));   // one-way: CapsLock → LCtrl
    }
    if toggles.swap_option_cmd {
        entries.push(entry(LALT, LWIN));
        entries.push(entry(LWIN, LALT));
        entries.push(entry(RALT, RWIN));
        entries.push(entry(RWIN, RALT));
    }

    if entries.is_empty() {
        return Vec::new();
    }

    let count = (entries.len() + 1) as u32; // +1 for the null terminator
    let mut buf = Vec::with_capacity(8 + 4 + entries.len() * 4 + 4);
    buf.extend_from_slice(&[0u8; 8]);                  // header
    buf.extend_from_slice(&count.to_le_bytes());       // entry count
    for e in &entries {
        buf.extend_from_slice(e);
    }
    buf.extend_from_slice(&[0u8; 4]);                  // null terminator
    buf
}

pub fn parse_scancode_map(_bytes: &[u8]) -> Result<Vec<RawScancodePair>, String> {
    unimplemented!("Task 3")
}

pub fn derive_state(_pairs: &[RawScancodePair]) -> ModifierState {
    unimplemented!("Task 3")
}

#[cfg(test)]
mod build_tests {
    use super::*;

    fn header(entry_count: u32) -> Vec<u8> {
        let mut v = vec![0u8; 8];
        v.extend_from_slice(&entry_count.to_le_bytes());
        v
    }

    fn terminator() -> [u8; 4] { [0, 0, 0, 0] }

    #[test]
    fn empty_toggles_produces_empty_vec() {
        let bytes = build_scancode_map(&ModifierToggles::default());
        assert!(bytes.is_empty(),
            "all-off toggles should produce an empty vec so the caller can DELETE the registry value");
    }

    #[test]
    fn caps_to_ctrl_only() {
        let bytes = build_scancode_map(&ModifierToggles { caps_to_ctrl: true, ..Default::default() });
        let mut expected = header(2); // 1 mapping + 1 terminator
        expected.extend_from_slice(&[0x1D, 0x00, 0x3A, 0x00]); // LCtrl ← CapsLock
        expected.extend_from_slice(&terminator());
        assert_eq!(bytes, expected);
    }

    #[test]
    fn swap_cmd_ctrl_left_produces_two_entries() {
        let bytes = build_scancode_map(&ModifierToggles { swap_cmd_ctrl_left: true, ..Default::default() });
        let mut expected = header(3); // 2 mappings + 1 terminator
        expected.extend_from_slice(&[0x1D, 0x00, 0x5B, 0xE0]); // LCtrl ← LWin
        expected.extend_from_slice(&[0x5B, 0xE0, 0x1D, 0x00]); // LWin ← LCtrl
        expected.extend_from_slice(&terminator());
        assert_eq!(bytes, expected);
    }

    #[test]
    fn swap_cmd_ctrl_both_sides() {
        let bytes = build_scancode_map(&ModifierToggles {
            swap_cmd_ctrl_left:  true,
            swap_cmd_ctrl_right: true,
            ..Default::default()
        });
        let mut expected = header(5); // 4 mappings + 1 terminator
        expected.extend_from_slice(&[0x1D, 0x00, 0x5B, 0xE0]); // LCtrl ← LWin
        expected.extend_from_slice(&[0x5B, 0xE0, 0x1D, 0x00]); // LWin ← LCtrl
        expected.extend_from_slice(&[0x1D, 0xE0, 0x5C, 0xE0]); // RCtrl ← RWin
        expected.extend_from_slice(&[0x5C, 0xE0, 0x1D, 0xE0]); // RWin ← RCtrl
        expected.extend_from_slice(&terminator());
        assert_eq!(bytes, expected);
    }

    #[test]
    fn swap_option_cmd_both_sides() {
        let bytes = build_scancode_map(&ModifierToggles { swap_option_cmd: true, ..Default::default() });
        let mut expected = header(5);
        expected.extend_from_slice(&[0x38, 0x00, 0x5B, 0xE0]); // LAlt ← LWin
        expected.extend_from_slice(&[0x5B, 0xE0, 0x38, 0x00]); // LWin ← LAlt
        expected.extend_from_slice(&[0x38, 0xE0, 0x5C, 0xE0]); // RAlt ← RWin
        expected.extend_from_slice(&[0x5C, 0xE0, 0x38, 0xE0]); // RWin ← RAlt
        expected.extend_from_slice(&terminator());
        assert_eq!(bytes, expected);
    }
}
