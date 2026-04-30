//! F12 / Eject key remap.
//!
//! Apple Magic Keyboards expose F12 as Eject, which has no useful action
//! under Windows. We let users repurpose it via a single Scancode Map entry
//! that maps source scancode 0x58 (F12) to one of a curated set of
//! destination scancodes that Windows handles natively.

use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
use crate::keyboard::modifiers::write_raw_pairs_elevated;
#[cfg(target_os = "windows")]
use crate::keyboard::scancode_map::RawScancodePair;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum F12Action {
    Default,
    Disabled,
    Calculator,
    Search,
    Mail,
    AppsMenu,
    VolumeMute,
}

impl F12Action {
    pub fn dest_scancode(&self) -> Option<&'static str> {
        match self {
            F12Action::Default    => None,
            F12Action::Disabled   => Some("0000"),
            F12Action::Calculator => Some("21E0"),
            F12Action::Search     => Some("65E0"),
            F12Action::Mail       => Some("6CE0"),
            F12Action::AppsMenu   => Some("5DE0"),
            F12Action::VolumeMute => Some("20E0"),
        }
    }

    pub fn from_dest_scancode(s: &str) -> Option<F12Action> {
        match s.to_uppercase().as_str() {
            "0000" => Some(F12Action::Disabled),
            "21E0" => Some(F12Action::Calculator),
            "65E0" => Some(F12Action::Search),
            "6CE0" => Some(F12Action::Mail),
            "5DE0" => Some(F12Action::AppsMenu),
            "20E0" => Some(F12Action::VolumeMute),
            _ => None,
        }
    }
}

const F12_SCAN_LE: &str = "5800";

fn is_f12_source(old_code: &str) -> bool {
    old_code.eq_ignore_ascii_case(F12_SCAN_LE)
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn f12_remap_get() -> Result<F12Action, String> {
    use crate::keyboard::modifiers::read_scancode_map;
    let state = read_scancode_map()?;
    for p in &state.raw_entries {
        if is_f12_source(&p.old_code) {
            if let Some(a) = F12Action::from_dest_scancode(&p.new_code) {
                return Ok(a);
            }
        }
    }
    Ok(F12Action::Default)
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn f12_remap_set(action: F12Action) -> Result<(), String> {
    use crate::keyboard::modifiers::read_scancode_map;
    let state = read_scancode_map()?;
    let mut new_pairs: Vec<RawScancodePair> = state
        .raw_entries
        .into_iter()
        .filter(|p| !is_f12_source(&p.old_code))
        .collect();
    if let Some(dest) = action.dest_scancode() {
        new_pairs.push(RawScancodePair {
            new_code: dest.to_string(),
            old_code: F12_SCAN_LE.to_string(),
        });
    }
    write_raw_pairs_elevated(&new_pairs)
}

// Non-Windows stubs:
#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn f12_remap_get() -> Result<F12Action, String> {
    Ok(F12Action::Default)
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn f12_remap_set(_action: F12Action) -> Result<(), String> {
    Err("f12_remap is only available on Windows".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dest_scancode_round_trip() {
        for a in [
            F12Action::Disabled, F12Action::Calculator, F12Action::Search,
            F12Action::Mail, F12Action::AppsMenu, F12Action::VolumeMute,
        ] {
            let s = a.dest_scancode().unwrap();
            assert_eq!(F12Action::from_dest_scancode(s), Some(a));
        }
    }

    #[test]
    fn default_has_no_dest() {
        assert!(F12Action::Default.dest_scancode().is_none());
        assert!(F12Action::from_dest_scancode("ffff").is_none());
    }

    #[test]
    fn is_f12_source_is_case_insensitive() {
        assert!(is_f12_source("5800"));
        assert!(is_f12_source("5800".to_uppercase().as_str()));
        assert!(!is_f12_source("5BE0")); // LWIN
    }
}
