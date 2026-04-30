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

/// Format a 2-byte little-endian scancode as a 4-hex-char uppercase string
/// (matching the format used by `RawScancodePair`).
fn bytes_to_hex_pair(b: [u8; 2]) -> String {
    format!("{:02X}{:02X}", b[0], b[1])
}

/// Parse a 4-hex-char little-endian scancode string back to its 2 bytes.
fn hex_pair_to_bytes(s: &str) -> Result<[u8; 2], String> {
    if s.len() != 4 {
        return Err(format!("expected 4 hex chars, got {:?}", s));
    }
    let lo = u8::from_str_radix(&s[0..2], 16).map_err(|e| format!("bad lo nibble: {e}"))?;
    let hi = u8::from_str_radix(&s[2..4], 16).map_err(|e| format!("bad hi nibble: {e}"))?;
    Ok([lo, hi])
}

/// Build a `RawScancodePair` from raw new/old byte arrays.
fn raw_pair(new: [u8; 2], old: [u8; 2]) -> RawScancodePair {
    RawScancodePair {
        new_code: bytes_to_hex_pair(new),
        old_code: bytes_to_hex_pair(old),
    }
}

/// Return the list of `RawScancodePair`s produced by a given toggle selection.
/// Pure: no I/O, no byte serialization. The order matches the byte-emission
/// order of `build_scancode_map` so a round-trip is byte-identical.
pub fn modifier_pairs_from_toggles(toggles: &ModifierToggles) -> Vec<RawScancodePair> {
    let mut pairs: Vec<RawScancodePair> = Vec::new();

    if toggles.swap_cmd_ctrl_left {
        pairs.push(raw_pair(LCTRL, LWIN));   // LWin pressed → LCtrl emitted
        pairs.push(raw_pair(LWIN,  LCTRL));  // LCtrl pressed → LWin emitted
    }
    if toggles.swap_cmd_ctrl_right {
        pairs.push(raw_pair(RCTRL, RWIN));
        pairs.push(raw_pair(RWIN,  RCTRL));
    }
    if toggles.caps_to_ctrl {
        pairs.push(raw_pair(LCTRL, CAPS));   // one-way: CapsLock → LCtrl
    }
    if toggles.swap_option_cmd {
        pairs.push(raw_pair(LALT, LWIN));
        pairs.push(raw_pair(LWIN, LALT));
        pairs.push(raw_pair(RALT, RWIN));
        pairs.push(raw_pair(RWIN, RALT));
    }

    pairs
}

/// Returns true when `old_code` (4-hex-char little-endian source scancode)
/// belongs to a key whose Scancode-Map entries are managed by the modifier
/// toggles subsystem. Used by other write paths (e.g. F12 remap) to avoid
/// clobbering modifier pairs and vice versa.
pub fn is_modifier_source(old_code: &str) -> bool {
    const MODIFIER_SOURCES: &[[u8; 2]] = &[LCTRL, RCTRL, LWIN, RWIN, LALT, RALT, CAPS];
    let upper = old_code.to_ascii_uppercase();
    MODIFIER_SOURCES
        .iter()
        .any(|b| bytes_to_hex_pair(*b) == upper)
}

/// Serialize an arbitrary list of `RawScancodePair`s into the binary Scancode
/// Map registry value. Returns an EMPTY vec when `pairs` is empty — the caller
/// should DELETE the registry value rather than write a header-only blob.
pub fn build_scancode_map_from_pairs(pairs: &[RawScancodePair]) -> Vec<u8> {
    if pairs.is_empty() {
        return Vec::new();
    }

    let count = (pairs.len() + 1) as u32; // +1 for the null terminator
    let mut buf = Vec::with_capacity(8 + 4 + pairs.len() * 4 + 4);
    buf.extend_from_slice(&[0u8; 8]);                  // header
    buf.extend_from_slice(&count.to_le_bytes());       // entry count
    for p in pairs {
        // Strings produced by us are always valid 4-hex-char codes. If a caller
        // hand-crafts a malformed pair, fall back to zero bytes rather than
        // panicking inside the elevated-write path.
        let new_b = hex_pair_to_bytes(&p.new_code).unwrap_or([0, 0]);
        let old_b = hex_pair_to_bytes(&p.old_code).unwrap_or([0, 0]);
        buf.extend_from_slice(&[new_b[0], new_b[1], old_b[0], old_b[1]]);
    }
    buf.extend_from_slice(&[0u8; 4]);                  // null terminator
    buf
}

/// Build the binary Scancode Map value from the user's toggle selection.
/// Returns an EMPTY vec when no toggles are active — the caller should DELETE
/// the registry value rather than write a header-only blob.
///
/// Thin wrapper used by the byte-identical refactor test that compares the
/// legacy toggle-only pipeline against `build_scancode_map_from_pairs`.
/// Gated `cfg(test)` because production callers (modifier-remap write path)
/// now go through the pair-level API directly.
#[cfg(test)]
pub fn build_scancode_map(toggles: &ModifierToggles) -> Vec<u8> {
    build_scancode_map_from_pairs(&modifier_pairs_from_toggles(toggles))
}

/// Parse a raw Scancode Map binary into a list of (new, old) pairs.
/// Returns an empty list for empty input or a header-only blob.
pub fn parse_scancode_map(bytes: &[u8]) -> Result<Vec<RawScancodePair>, String> {
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    if bytes.len() < 16 {
        return Err(format!("Scancode Map too short: {} bytes (need >= 16)", bytes.len()));
    }
    // Skip 8-byte header, read count (LE u32) at offset 8
    let count = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
    if count == 0 {
        return Err("Scancode Map count is 0; expected >= 1 (terminator)".to_string());
    }
    // Body = count * 4 bytes (the last 4 bytes are the terminator)
    let body_len = count * 4;
    let expected_total = 12 + body_len;
    if bytes.len() < expected_total {
        return Err(format!(
            "Scancode Map truncated: header says {} entries (need {} bytes total) but got {}",
            count, expected_total, bytes.len()
        ));
    }
    let mut pairs = Vec::with_capacity(count.saturating_sub(1));
    for i in 0..count {
        let off = 12 + i * 4;
        let chunk = &bytes[off..off + 4];
        // Last entry should be the terminator (all zeros) — skip it
        if i == count - 1 {
            if chunk != [0u8; 4] {
                return Err("Scancode Map missing null terminator".to_string());
            }
            break;
        }
        pairs.push(RawScancodePair {
            new_code: format!("{:02X}{:02X}", chunk[0], chunk[1]),
            old_code: format!("{:02X}{:02X}", chunk[2], chunk[3]),
        });
    }
    Ok(pairs)
}

/// Reverse-derive which toggles the user has already enabled, based on the raw
/// pairs read from the registry. Pairs that don't match any known toggle group
/// flip `has_external_mappings = true` (used by the UI to warn before overwrite).
pub fn derive_state(pairs: &[RawScancodePair]) -> ModifierState {
    fn pair(new: [u8; 2], old: [u8; 2]) -> RawScancodePair {
        RawScancodePair {
            new_code: format!("{:02X}{:02X}", new[0], new[1]),
            old_code: format!("{:02X}{:02X}", old[0], old[1]),
        }
    }

    let cmd_ctrl_left  = vec![pair(LCTRL, LWIN), pair(LWIN, LCTRL)];
    let cmd_ctrl_right = vec![pair(RCTRL, RWIN), pair(RWIN, RCTRL)];
    let caps           = vec![pair(LCTRL, CAPS)];
    let option_cmd     = vec![
        pair(LALT, LWIN), pair(LWIN, LALT),
        pair(RALT, RWIN), pair(RWIN, RALT),
    ];

    let pair_set: std::collections::HashSet<&RawScancodePair> = pairs.iter().collect();
    let group_present = |group: &[RawScancodePair]| group.iter().all(|p| pair_set.contains(p));

    let toggles = ModifierToggles {
        swap_cmd_ctrl_left:  group_present(&cmd_ctrl_left),
        swap_cmd_ctrl_right: group_present(&cmd_ctrl_right),
        caps_to_ctrl:        group_present(&caps),
        swap_option_cmd:     group_present(&option_cmd),
    };

    // External = any pair in the registry that isn't part of an active group.
    let mut accounted: std::collections::HashSet<&RawScancodePair> = std::collections::HashSet::new();
    if toggles.swap_cmd_ctrl_left  { for p in &cmd_ctrl_left  { accounted.insert(p); } }
    if toggles.swap_cmd_ctrl_right { for p in &cmd_ctrl_right { accounted.insert(p); } }
    if toggles.caps_to_ctrl        { for p in &caps           { accounted.insert(p); } }
    if toggles.swap_option_cmd     { for p in &option_cmd     { accounted.insert(p); } }

    let has_external_mappings = pairs.iter().any(|p| !accounted.contains(p));

    ModifierState {
        current: toggles,
        has_external_mappings,
        raw_entries: pairs.to_vec(),
    }
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

    /// The pure pair-based pipeline (`build_scancode_map_from_pairs ∘
    /// modifier_pairs_from_toggles`) MUST emit the exact same bytes as the
    /// original `build_scancode_map(toggles)`. This guards the refactor — if
    /// it ever drifts, every existing on-disk Scancode Map blob would change
    /// out from under users.
    #[test]
    fn pair_pipeline_is_byte_identical_to_legacy_for_all_toggle_combos() {
        for n in 0u8..16 {
            let toggles = ModifierToggles {
                swap_cmd_ctrl_left:  (n & 0b0001) != 0,
                swap_cmd_ctrl_right: (n & 0b0010) != 0,
                caps_to_ctrl:        (n & 0b0100) != 0,
                swap_option_cmd:     (n & 0b1000) != 0,
            };
            let legacy_bytes = build_scancode_map(&toggles);
            let via_pairs = build_scancode_map_from_pairs(&modifier_pairs_from_toggles(&toggles));
            assert_eq!(
                legacy_bytes, via_pairs,
                "byte mismatch for toggles {:?}", toggles
            );
        }
    }

    #[test]
    fn is_modifier_source_recognizes_known_modifiers() {
        // All modifier source scancodes the toggles emit/consume.
        assert!(is_modifier_source("1D00")); // LCTRL
        assert!(is_modifier_source("1DE0")); // RCTRL
        assert!(is_modifier_source("5BE0")); // LWIN
        assert!(is_modifier_source("5CE0")); // RWIN
        assert!(is_modifier_source("3800")); // LALT
        assert!(is_modifier_source("38E0")); // RALT
        assert!(is_modifier_source("3A00")); // CAPS
    }

    #[test]
    fn is_modifier_source_is_case_insensitive() {
        assert!(is_modifier_source("5be0"));
        assert!(is_modifier_source("5Be0"));
    }

    #[test]
    fn is_modifier_source_rejects_non_modifier_scancodes() {
        assert!(!is_modifier_source("5800")); // F12 / Eject
        assert!(!is_modifier_source("0000"));
        assert!(!is_modifier_source("21E0")); // Calculator
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;

    fn make_pair(new_lo: u8, new_hi: u8, old_lo: u8, old_hi: u8) -> RawScancodePair {
        RawScancodePair {
            new_code: format!("{:02X}{:02X}", new_lo, new_hi),
            old_code: format!("{:02X}{:02X}", old_lo, old_hi),
        }
    }

    #[test]
    fn parse_empty_input_returns_empty_list() {
        assert_eq!(parse_scancode_map(&[]).unwrap(), Vec::new());
    }

    #[test]
    fn parse_header_only_blob() {
        // 8-byte header + count=1 (just terminator) + 4-byte terminator = 16 bytes
        let bytes = [0u8; 16];
        let mut bytes_with_count = bytes.to_vec();
        bytes_with_count[8] = 1; // count = 1
        assert_eq!(parse_scancode_map(&bytes_with_count).unwrap(), Vec::new());
    }

    #[test]
    fn parse_caps_to_ctrl_blob() {
        let mut bytes = vec![0u8; 8];
        bytes.extend_from_slice(&2u32.to_le_bytes());      // count = 2
        bytes.extend_from_slice(&[0x1D, 0x00, 0x3A, 0x00]); // LCtrl ← CapsLock
        bytes.extend_from_slice(&[0u8; 4]);                 // terminator
        let parsed = parse_scancode_map(&bytes).unwrap();
        assert_eq!(parsed, vec![make_pair(0x1D, 0x00, 0x3A, 0x00)]);
    }

    #[test]
    fn parse_rejects_truncated_input() {
        // Header says 5 entries but only 1 follows
        let mut bytes = vec![0u8; 8];
        bytes.extend_from_slice(&5u32.to_le_bytes());
        bytes.extend_from_slice(&[0x1D, 0x00, 0x3A, 0x00]);
        // missing 3 entries + terminator
        assert!(parse_scancode_map(&bytes).is_err());
    }

    #[test]
    fn derive_state_recognizes_caps_only() {
        let pairs = vec![make_pair(0x1D, 0x00, 0x3A, 0x00)];
        let state = derive_state(&pairs);
        assert!(state.current.caps_to_ctrl);
        assert!(!state.current.swap_cmd_ctrl_left);
        assert!(!state.has_external_mappings);
    }

    #[test]
    fn derive_state_recognizes_cmd_ctrl_left() {
        let pairs = vec![
            make_pair(0x1D, 0x00, 0x5B, 0xE0), // LCtrl ← LWin
            make_pair(0x5B, 0xE0, 0x1D, 0x00), // LWin ← LCtrl
        ];
        let state = derive_state(&pairs);
        assert!(state.current.swap_cmd_ctrl_left);
        assert!(!state.current.swap_cmd_ctrl_right);
        assert!(!state.has_external_mappings);
    }

    #[test]
    fn derive_state_flags_external_mappings() {
        // Half a swap pair (only one direction) — not a recognized toggle
        let pairs = vec![make_pair(0x1D, 0x00, 0x5B, 0xE0)];
        let state = derive_state(&pairs);
        assert!(!state.current.swap_cmd_ctrl_left);
        assert!(state.has_external_mappings);
    }

    #[test]
    fn derive_state_recognizes_compound_toggles() {
        // Cmd↔Ctrl both sides + Caps→Ctrl
        let pairs = vec![
            make_pair(0x1D, 0x00, 0x5B, 0xE0),
            make_pair(0x5B, 0xE0, 0x1D, 0x00),
            make_pair(0x1D, 0xE0, 0x5C, 0xE0),
            make_pair(0x5C, 0xE0, 0x1D, 0xE0),
            make_pair(0x1D, 0x00, 0x3A, 0x00),
        ];
        let state = derive_state(&pairs);
        assert!(state.current.swap_cmd_ctrl_left);
        assert!(state.current.swap_cmd_ctrl_right);
        assert!(state.current.caps_to_ctrl);
        assert!(!state.current.swap_option_cmd);
        assert!(!state.has_external_mappings);
    }
}
