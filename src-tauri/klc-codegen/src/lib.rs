//! Windows keyboard layout DLL (C source) generator.
//!
//! Takes a [`Layout`] parsed from a MagicWindows layout JSON file and emits a
//! self-contained C source file that, when compiled with `cl.exe /LD /NOENTRY`
//! and linked against `user32.lib`, produces a working keyboard layout DLL
//! exposing `KbdLayerDescriptor()`.
//!
//! # Shift-state model
//!
//! Our layouts use five columns that map directly to the KLC/DDK shift states:
//!
//! | Column     | Shift state index | Keys held      |
//! |------------|-------------------|----------------|
//! | base       | 0                 | (none)         |
//! | shift      | 1                 | Shift          |
//! | ctrl       | 2                 | Ctrl           |
//! | altgr      | 3                 | Ctrl+Alt       |
//! | altgrShift | 4                 | Shift+Ctrl+Alt |
//!
//! This matches the KLC SHIFTSTATE sequence `0 1 2 6 7`.
//!
//! This crate is consumed both by `src-tauri/build.rs` (to produce the shipped
//! DLLs) and by the tests here — a single source of truth for the codegen.

use serde::Deserialize;
use std::collections::HashMap;

// ── Input types (minimal projection of the full layout JSON) ────────────────
//
// We deliberately define our own Layout type here rather than depend on the
// main crate's richer `Layout`: build scripts cannot import from the crate
// they build, and codegen only needs a small subset of the fields. Unknown
// JSON fields are silently ignored by serde (no `deny_unknown_fields`), so
// a full MagicWindows layout JSON deserializes cleanly into this struct.

/// One key mapping row (hex-codepoint strings or "-1").
#[derive(Debug, Clone, Deserialize)]
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
#[derive(Debug, Clone, Deserialize)]
pub struct DeadKey {
    pub name: String,
    pub combinations: HashMap<String, String>,
}

/// Layout projection required by the codegen. Rich fields (description,
/// detection keys, locale_id...) are deserialized-but-ignored.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layout {
    pub id: String,
    pub name: HashMap<String, String>,
    pub locale: String,
    pub dll_name: String,
    pub keys: HashMap<String, KeyMapping>,
    #[serde(default)]
    pub dead_keys: HashMap<String, DeadKey>,
}

// ── Public entry point ──────────────────────────────────────────────────────

/// Generate a complete Windows keyboard layout DLL C source file.
///
/// The returned string is valid C source that, when compiled with
/// `cl.exe /LD /NOENTRY` and linked against `user32.lib`, produces a
/// working keyboard layout DLL.
pub fn generate_kbd_c(layout: &Layout) -> String {
    let mut out = String::with_capacity(32 * 1024);
    let has_altgr = layout_has_altgr(layout);
    let has_dead_keys = !layout.dead_keys.is_empty();

    let entries = collect_vk_entries(layout);

    emit_header(&mut out, layout);
    emit_vsc_to_vk(&mut out, layout);
    emit_e0_e1_tables(&mut out);
    emit_vk_to_wchar_tables(&mut out, &entries, has_altgr);
    emit_control_key_tables(&mut out);
    emit_dead_key_table(&mut out, layout, has_dead_keys);
    emit_key_names(&mut out);
    emit_key_names_ext(&mut out);
    emit_dead_key_names(&mut out, layout, has_dead_keys);
    emit_modifiers(&mut out, has_altgr);
    emit_kbd_tables(&mut out, has_altgr);
    emit_export(&mut out);

    out
}

// ── Internal helpers ────────────────────────────────────────────────────────

#[derive(Debug)]
struct VkEntry {
    vk: String,
    /// caps-lock attribute: "0" = no caps lock, "1" = caps-lock like shift, "4" = SGCAPS
    cap: String,
    base: CharVal,
    shift: CharVal,
    ctrl: CharVal,
    altgr: CharVal,
    altgr_shift: CharVal,
}

#[derive(Debug, Clone)]
enum CharVal {
    /// No character (`-1` in JSON).
    None,
    /// Normal character (the unicode codepoint).
    Char(u32),
    /// Dead key character (the unicode codepoint, marked with `@` suffix in JSON).
    Dead(u32),
}

impl CharVal {
    fn parse(s: &str) -> Self {
        let s = s.trim();
        if s == "-1" || s.is_empty() {
            return CharVal::None;
        }
        let (is_dead, hex) = if let Some(stripped) = s.strip_suffix('@') {
            (true, stripped)
        } else {
            (false, s)
        };
        let cp = u32::from_str_radix(hex, 16).unwrap_or(0);
        if is_dead {
            CharVal::Dead(cp)
        } else {
            CharVal::Char(cp)
        }
    }

    fn c_wch(&self) -> String {
        match self {
            CharVal::None => "WCH_NONE".to_string(),
            CharVal::Char(cp) | CharVal::Dead(cp) => format!("0x{cp:04X}"),
        }
    }

    fn is_dead(&self) -> bool {
        matches!(self, CharVal::Dead(_))
    }

    fn is_none(&self) -> bool {
        matches!(self, CharVal::None)
    }
}

fn collect_vk_entries(layout: &Layout) -> Vec<VkEntry> {
    let mut scancodes: Vec<&String> = layout.keys.keys().collect();
    scancodes.sort_by_key(|sc| u32::from_str_radix(sc, 16).unwrap_or(0));

    scancodes
        .into_iter()
        .map(|sc| {
            let km = &layout.keys[sc];
            VkEntry {
                vk: km.vk.clone(),
                cap: km.cap.clone(),
                base: CharVal::parse(&km.base),
                shift: CharVal::parse(&km.shift),
                ctrl: CharVal::parse(&km.ctrl),
                altgr: CharVal::parse(&km.altgr),
                altgr_shift: CharVal::parse(&km.altgr_shift),
            }
        })
        .collect()
}

fn layout_has_altgr(layout: &Layout) -> bool {
    layout.keys.values().any(|km| {
        let ag = CharVal::parse(&km.altgr);
        let ags = CharVal::parse(&km.altgr_shift);
        !ag.is_none() || !ags.is_none()
    })
}

fn push(out: &mut String, s: &str) {
    out.push_str(s);
    out.push('\n');
}

// ── Section emitters ────────────────────────────────────────────────────────

fn emit_header(out: &mut String, layout: &Layout) {
    let name = layout.name.get("en").map_or(layout.id.as_str(), |s| s.as_str());
    push(out, "/*");
    push(out, &format!(" * Keyboard layout: {name}"));
    push(out, &format!(" * Locale:          {}", layout.locale));
    push(out, " * Generated by MagicWindows klc-codegen.  DO NOT EDIT.");
    push(out, " */");
    push(out, "");
    push(out, "#define KBD_TYPE 4");
    push(out, "");
    // Include order matters: do NOT include windows.h because it pulls in
    // the full Win32 headers which cause name collisions with kbd.h
    // (e.g. the VK_F struct typedef vs winuser function-param type).
    push(out, "#include <windef.h>");
    push(out, "#include <winuser.h>");
    push(out, "#include <kbd.h>");
    push(out, "");
    // ALLOC_SECTION_LDATA is a DDK/WDK macro for section placement.
    // For user-mode keyboard layout DLLs it can be defined as empty.
    push(out, "#ifndef ALLOC_SECTION_LDATA");
    push(out, "#define ALLOC_SECTION_LDATA");
    push(out, "#endif");
    push(out, "");
}

/// Emit the `ausVK[]` table: scancode -> Virtual Key.
fn emit_vsc_to_vk(out: &mut String, layout: &Layout) {
    let mut table: Vec<String> = BASE_VSC_TABLE.iter().map(|s| s.to_string()).collect();

    for (sc_str, km) in &layout.keys {
        let sc = match u32::from_str_radix(sc_str, 16) {
            Ok(v) if (v as usize) < table.len() => v as usize,
            _ => continue,
        };
        table[sc] = vk_name_to_c(&km.vk);
    }

    push(out, "static USHORT ausVK[] = {");
    for (i, vk) in table.iter().enumerate() {
        push(out, &format!("    /* {i:02X} */ {vk},"));
    }
    push(out, "};");
    push(out, "");
}

fn emit_e0_e1_tables(out: &mut String) {
    push(out, "static VSC_VK aE0VscToVk[] = {");
    for &(vsc, vk) in E0_TABLE {
        push(out, &format!("    {{ 0x{vsc:02X}, {vk} }},"));
    }
    push(out, "    { 0,    0 }  /* Terminator */");
    push(out, "};");
    push(out, "");
    push(out, "static VSC_VK aE1VscToVk[] = {");
    push(out, "    { 0x1D, VK_PAUSE },");
    push(out, "    { 0,    0 }  /* Terminator */");
    push(out, "};");
    push(out, "");
}

fn emit_vk_to_wchar_tables(out: &mut String, entries: &[VkEntry], has_altgr: bool) {
    if has_altgr {
        emit_wchar_table5(out, entries);
    } else {
        emit_wchar_table3(out, entries);
    }
}

/// 5-column table: base | shift | ctrl | altgr | altgr+shift
fn emit_wchar_table5(out: &mut String, entries: &[VkEntry]) {
    push(out, "static VK_TO_WCHARS5 aVkToWch5[] = {");
    push(out, "//   VkKey    Attr    Base      Shift     Ctrl      AltGr     AltGrSh");

    for entry in entries {
        let attr = caps_attr(&entry.cap);
        let vk_c = vk_name_to_c(&entry.vk);
        let any_dead = entry.base.is_dead()
            || entry.shift.is_dead()
            || entry.altgr.is_dead()
            || entry.altgr_shift.is_dead();

        push(
            out,
            &format!(
                "    {{ {:<20} 0x{attr:02X}, {{ {}, {}, {}, {}, {} }} }},",
                format!("{vk_c},"),
                entry.base.c_wch(),
                entry.shift.c_wch(),
                entry.ctrl.c_wch(),
                entry.altgr.c_wch(),
                entry.altgr_shift.c_wch(),
            ),
        );

        if any_dead {
            let d = |cv: &CharVal| -> &'static str {
                if cv.is_dead() { "WCH_DEAD" } else { "WCH_NONE" }
            };
            push(
                out,
                &format!(
                    "    {{ (BYTE)-1,              0x00, {{ {}, {}, WCH_NONE, {}, {} }} }},",
                    d(&entry.base),
                    d(&entry.shift),
                    d(&entry.altgr),
                    d(&entry.altgr_shift),
                ),
            );
        }
    }
    push(out, "    { 0,                    0x00, { WCH_NONE, WCH_NONE, WCH_NONE, WCH_NONE, WCH_NONE } }");
    push(out, "};");
    push(out, "");
}

/// 3-column table: base | shift | ctrl (no AltGr).
fn emit_wchar_table3(out: &mut String, entries: &[VkEntry]) {
    push(out, "static VK_TO_WCHARS3 aVkToWch3[] = {");
    push(out, "//   VkKey    Attr    Base      Shift     Ctrl");

    for entry in entries {
        let attr = caps_attr(&entry.cap);
        let vk_c = vk_name_to_c(&entry.vk);
        let any_dead = entry.base.is_dead() || entry.shift.is_dead();

        push(
            out,
            &format!(
                "    {{ {:<20} 0x{attr:02X}, {{ {}, {}, {} }} }},",
                format!("{vk_c},"),
                entry.base.c_wch(),
                entry.shift.c_wch(),
                entry.ctrl.c_wch(),
            ),
        );

        if any_dead {
            let d = |cv: &CharVal| -> &'static str {
                if cv.is_dead() { "WCH_DEAD" } else { "WCH_NONE" }
            };
            push(
                out,
                &format!(
                    "    {{ (BYTE)-1,              0x00, {{ {}, {}, WCH_NONE }} }},",
                    d(&entry.base),
                    d(&entry.shift),
                ),
            );
        }
    }
    push(out, "    { 0,                    0x00, { WCH_NONE, WCH_NONE, WCH_NONE } }");
    push(out, "};");
    push(out, "");
}

/// Emit the fixed `VK_TO_WCHARS1` and `VK_TO_WCHARS2` tables covering the
/// control-character keys (ENTER, BACKSPACE, TAB, ESCAPE, CANCEL) and the
/// numeric keypad.
///
/// Without these tables, `ToUnicodeEx` returns 0 for those VKs and Chromium-
/// based apps (VSCode, Chrome, Electron) fail to produce newlines or handle
/// backspace correctly — classic Win32 apps are unaffected because
/// `TranslateMessage` hard-codes `'\r'` for VK_RETURN on the Windows side and
/// masks the gap. This mirrors what Microsoft's reference layouts do.
fn emit_control_key_tables(out: &mut String) {
    push(out, "static VK_TO_WCHARS1 aVkToWch1[] = {");
    push(out, "    { VK_NUMPAD0,  0x00, { L'0'    } },");
    push(out, "    { VK_NUMPAD1,  0x00, { L'1'    } },");
    push(out, "    { VK_NUMPAD2,  0x00, { L'2'    } },");
    push(out, "    { VK_NUMPAD3,  0x00, { L'3'    } },");
    push(out, "    { VK_NUMPAD4,  0x00, { L'4'    } },");
    push(out, "    { VK_NUMPAD5,  0x00, { L'5'    } },");
    push(out, "    { VK_NUMPAD6,  0x00, { L'6'    } },");
    push(out, "    { VK_NUMPAD7,  0x00, { L'7'    } },");
    push(out, "    { VK_NUMPAD8,  0x00, { L'8'    } },");
    push(out, "    { VK_NUMPAD9,  0x00, { L'9'    } },");
    push(out, "    { VK_TAB,      0x00, { 0x0009 } },");
    push(out, "    { VK_ADD,      0x00, { L'+'    } },");
    push(out, "    { VK_DIVIDE,   0x00, { L'/'    } },");
    push(out, "    { VK_MULTIPLY, 0x00, { L'*'    } },");
    push(out, "    { VK_SUBTRACT, 0x00, { L'-'    } },");
    push(out, "    { 0,           0x00, { 0      } }");
    push(out, "};");
    push(out, "");

    push(out, "static VK_TO_WCHARS2 aVkToWch2[] = {");
    push(out, "    { VK_BACK,    0x00, { 0x0008, 0x007F } },");
    push(out, "    { VK_ESCAPE,  0x00, { 0x001B, 0x001B } },");
    // Shift+Enter: emit CR (same as Enter alone) instead of LF.
    // Returning a printable wchar for Shift+Enter makes Chromium/Electron
    // (Antigravity, VSCode, Slack…) treat it as a text-input event and swallow
    // the keybinding — Shift+Enter then no longer triggers terminal newline.
    // Microsoft's stock layouts leave the second slot empty for the same reason.
    push(out, "    { VK_RETURN,  0x00, { 0x000D, 0x000D } },");
    push(out, "    { VK_CANCEL,  0x00, { 0x0003, 0x0003 } },");
    push(out, "    { 0,          0x00, { 0,      0      } }");
    push(out, "};");
    push(out, "");
}

fn caps_attr(cap: &str) -> u8 {
    match cap.trim() {
        "1" => 0x01, // CAPLOK
        "4" => 0x02, // SGCAPS
        _ => 0x00,
    }
}

fn emit_dead_key_table(out: &mut String, layout: &Layout, has_dead_keys: bool) {
    if !has_dead_keys {
        push(out, "static DEADKEY aDeadKey[] = { { 0, 0, 0 } };");
        push(out, "");
        return;
    }

    push(out, "static DEADKEY aDeadKey[] = {");

    let mut dk_codes: Vec<&String> = layout.dead_keys.keys().collect();
    dk_codes.sort();

    for dk_code in &dk_codes {
        let dk = &layout.dead_keys[*dk_code];
        let accent_cp = u32::from_str_radix(dk_code, 16).unwrap_or(0);

        let mut combos: Vec<(&String, &String)> = dk.combinations.iter().collect();
        combos.sort_by_key(|(k, _)| *k);

        for (base_str, result_str) in &combos {
            let base_cp = u32::from_str_radix(base_str, 16).unwrap_or(0);
            let result_cp = u32::from_str_radix(result_str, 16).unwrap_or(0);
            push(
                out,
                &format!(
                    "    DEADTRANS( 0x{base_cp:04X}, 0x{accent_cp:04X}, 0x{result_cp:04X}, 0x0000 ),"
                ),
            );
        }
    }

    push(out, "    { 0, 0, 0 }");
    push(out, "};");
    push(out, "");
}

fn emit_key_names(out: &mut String) {
    push(out, "static ALLOC_SECTION_LDATA VSC_LPWSTR aKeyNames[] = {");
    for &(vsc, name) in KEY_NAMES {
        push(out, &format!("    {{ 0x{vsc:02x}, L\"{name}\" }},"));
    }
    push(out, "    { 0,    NULL }");
    push(out, "};");
    push(out, "");
}

fn emit_key_names_ext(out: &mut String) {
    push(out, "static ALLOC_SECTION_LDATA VSC_LPWSTR aKeyNamesExt[] = {");
    for &(vsc, name) in KEY_NAMES_EXT {
        push(out, &format!("    {{ 0x{vsc:02x}, L\"{name}\" }},"));
    }
    push(out, "    { 0,    NULL }");
    push(out, "};");
    push(out, "");
}

fn emit_dead_key_names(out: &mut String, layout: &Layout, has_dead_keys: bool) {
    push(out, "static DEADKEY_LPWSTR aKeyNamesDead[] = {");
    if has_dead_keys {
        let mut dk_codes: Vec<&String> = layout.dead_keys.keys().collect();
        dk_codes.sort();
        for dk_code in &dk_codes {
            let dk = &layout.dead_keys[*dk_code];
            push(out, &format!("    L\"{}\",", dk.name));
        }
    }
    push(out, "    NULL");
    push(out, "};");
    push(out, "");
}

fn emit_modifiers(out: &mut String, has_altgr: bool) {
    push(out, "static VK_TO_BIT aVkToBits[] = {");
    push(out, "    { VK_SHIFT,   KBDSHIFT },");
    push(out, "    { VK_CONTROL, KBDCTRL  },");
    push(out, "    { VK_MENU,    KBDALT   },");
    push(out, "    { 0,          0        }");
    push(out, "};");
    push(out, "");

    if has_altgr {
        push(out, "static MODIFIERS CharModifiers = {");
        push(out, "    &aVkToBits[0],");
        push(out, "    7,");
        push(out, "    {");
        push(out, "    //  Shift  Ctrl   Alt");
        push(out, "        0,         // 000 = base");
        push(out, "        1,         // 001 = Shift");
        push(out, "        2,         // 010 = Ctrl");
        push(out, "        SHFT_INVALID, // 011 = Shift+Ctrl");
        push(out, "        SHFT_INVALID, // 100 = Alt");
        push(out, "        SHFT_INVALID, // 101 = Shift+Alt");
        push(out, "        3,         // 110 = Ctrl+Alt (AltGr)");
        push(out, "        4          // 111 = Shift+Ctrl+Alt (Shift+AltGr)");
        push(out, "    }");
        push(out, "};");
    } else {
        push(out, "static MODIFIERS CharModifiers = {");
        push(out, "    &aVkToBits[0],");
        push(out, "    7,");
        push(out, "    {");
        push(out, "    //  Shift  Ctrl   Alt");
        push(out, "        0,         // 000 = base");
        push(out, "        1,         // 001 = Shift");
        push(out, "        2,         // 010 = Ctrl");
        push(out, "        SHFT_INVALID, // 011 = Shift+Ctrl");
        push(out, "        SHFT_INVALID, // 100 = Alt");
        push(out, "        SHFT_INVALID, // 101 = Shift+Alt");
        push(out, "        SHFT_INVALID, // 110 = Ctrl+Alt");
        push(out, "        SHFT_INVALID  // 111 = Shift+Ctrl+Alt");
        push(out, "    }");
        push(out, "};");
    }
    push(out, "");
}

fn emit_kbd_tables(out: &mut String, has_altgr: bool) {
    let locale_flags = if has_altgr {
        "(MAKELONG(KLLF_ALTGR, KBD_VERSION))"
    } else {
        "(MAKELONG(0, KBD_VERSION))"
    };

    let wchar_n = if has_altgr { "5" } else { "3" };

    push(out, "static VK_TO_WCHAR_TABLE aVkToWcharTable[] = {");
    push(
        out,
        &format!(
            "    {{ (PVK_TO_WCHARS1)aVkToWch{wchar_n}, {wchar_n}, sizeof(aVkToWch{wchar_n}[0]) }},"
        ),
    );
    push(out, "    { (PVK_TO_WCHARS1)aVkToWch1, 1, sizeof(aVkToWch1[0]) },");
    push(out, "    { (PVK_TO_WCHARS1)aVkToWch2, 2, sizeof(aVkToWch2[0]) },");
    push(out, "    { NULL, 0, 0 }");
    push(out, "};");
    push(out, "");

    push(out, "static KBDTABLES KbdTables = {");
    push(out, "    &CharModifiers,");
    push(out, "    aVkToWcharTable,");
    push(out, "    aDeadKey,");
    push(out, "    (PVSC_LPWSTR)aKeyNames,");
    push(out, "    (PVSC_LPWSTR)aKeyNamesExt,");
    push(out, "    (WCHAR * KBD_LONG_POINTER * KBD_LONG_POINTER)aKeyNamesDead,");
    push(out, "    (USHORT * KBD_LONG_POINTER)ausVK,");
    push(out, "    sizeof(ausVK) / sizeof(ausVK[0]),");
    push(out, "    (PVSC_VK)aE0VscToVk,");
    push(out, "    (PVSC_VK)aE1VscToVk,");
    push(out, &format!("    {locale_flags},"));
    push(out, "    0,");
    push(out, "    0,");
    push(out, "    NULL,");
    push(out, "    0,");
    push(out, "    0");
    push(out, "};");
    push(out, "");
}

fn emit_export(out: &mut String) {
    push(out, "__declspec(dllexport) PKBDTABLES KbdLayerDescriptor(void)");
    push(out, "{");
    push(out, "    return &KbdTables;");
    push(out, "}");
    push(out, "");
}

/// Map a JSON VK name to a C expression. Alphanumeric VKs use raw hex
/// literals because those constants are not defined in winuser.h (they equal
/// the ASCII code); using the symbolic names would conflict with the VK_F
/// struct typedef in kbd.h.
fn vk_name_to_c(vk: &str) -> String {
    match vk {
        "VK_0" => "0x30".into(),
        "VK_1" => "0x31".into(),
        "VK_2" => "0x32".into(),
        "VK_3" => "0x33".into(),
        "VK_4" => "0x34".into(),
        "VK_5" => "0x35".into(),
        "VK_6" => "0x36".into(),
        "VK_7" => "0x37".into(),
        "VK_8" => "0x38".into(),
        "VK_9" => "0x39".into(),
        "VK_A" => "0x41".into(),
        "VK_B" => "0x42".into(),
        "VK_C" => "0x43".into(),
        "VK_D" => "0x44".into(),
        "VK_E" => "0x45".into(),
        "VK_F" => "0x46".into(),
        "VK_G" => "0x47".into(),
        "VK_H" => "0x48".into(),
        "VK_I" => "0x49".into(),
        "VK_J" => "0x4A".into(),
        "VK_K" => "0x4B".into(),
        "VK_L" => "0x4C".into(),
        "VK_M" => "0x4D".into(),
        "VK_N" => "0x4E".into(),
        "VK_O" => "0x4F".into(),
        "VK_P" => "0x50".into(),
        "VK_Q" => "0x51".into(),
        "VK_R" => "0x52".into(),
        "VK_S" => "0x53".into(),
        "VK_T" => "0x54".into(),
        "VK_U" => "0x55".into(),
        "VK_V" => "0x56".into(),
        "VK_W" => "0x57".into(),
        "VK_X" => "0x58".into(),
        "VK_Y" => "0x59".into(),
        "VK_Z" => "0x5A".into(),
        "VK_SPACE"      => "VK_SPACE".into(),
        "VK_OEM_1"      => "VK_OEM_1".into(),
        "VK_OEM_2"      => "VK_OEM_2".into(),
        "VK_OEM_3"      => "VK_OEM_3".into(),
        "VK_OEM_4"      => "VK_OEM_4".into(),
        "VK_OEM_5"      => "VK_OEM_5".into(),
        "VK_OEM_6"      => "VK_OEM_6".into(),
        "VK_OEM_7"      => "VK_OEM_7".into(),
        "VK_OEM_8"      => "VK_OEM_8".into(),
        "VK_OEM_102"    => "VK_OEM_102".into(),
        "VK_OEM_COMMA"  => "VK_OEM_COMMA".into(),
        "VK_OEM_PERIOD" => "VK_OEM_PERIOD".into(),
        "VK_OEM_MINUS"  => "VK_OEM_MINUS".into(),
        "VK_OEM_PLUS"   => "VK_OEM_PLUS".into(),
        _ => "0".into(),
    }
}

// ── Static tables ───────────────────────────────────────────────────────────

/// Standard VSC→VK table for a 101/104-key US-type keyboard, indexed by
/// scancode byte 0x00–0x7F.
static BASE_VSC_TABLE: &[&str] = &[
    /* 00 */ "0",
    /* 01 */ "VK_ESCAPE",
    /* 02 */ "0x31",  // VK_1
    /* 03 */ "0x32",  // VK_2
    /* 04 */ "0x33",  // VK_3
    /* 05 */ "0x34",  // VK_4
    /* 06 */ "0x35",  // VK_5
    /* 07 */ "0x36",  // VK_6
    /* 08 */ "0x37",  // VK_7
    /* 09 */ "0x38",  // VK_8
    /* 0A */ "0x39",  // VK_9
    /* 0B */ "0x30",  // VK_0
    /* 0C */ "VK_OEM_MINUS",
    /* 0D */ "VK_OEM_PLUS",
    /* 0E */ "VK_BACK",
    /* 0F */ "VK_TAB",
    /* 10 */ "0x51",  // VK_Q
    /* 11 */ "0x57",  // VK_W
    /* 12 */ "0x45",  // VK_E
    /* 13 */ "0x52",  // VK_R
    /* 14 */ "0x54",  // VK_T
    /* 15 */ "0x59",  // VK_Y
    /* 16 */ "0x55",  // VK_U
    /* 17 */ "0x49",  // VK_I
    /* 18 */ "0x4F",  // VK_O
    /* 19 */ "0x50",  // VK_P
    /* 1A */ "VK_OEM_4",
    /* 1B */ "VK_OEM_6",
    /* 1C */ "VK_RETURN",
    /* 1D */ "VK_LCONTROL",
    /* 1E */ "0x41",  // VK_A
    /* 1F */ "0x53",  // VK_S
    /* 20 */ "0x44",  // VK_D
    /* 21 */ "0x46",  // VK_F
    /* 22 */ "0x47",  // VK_G
    /* 23 */ "0x48",  // VK_H
    /* 24 */ "0x4A",  // VK_J
    /* 25 */ "0x4B",  // VK_K
    /* 26 */ "0x4C",  // VK_L
    /* 27 */ "VK_OEM_1",
    /* 28 */ "VK_OEM_7",
    /* 29 */ "VK_OEM_3",
    /* 2A */ "VK_LSHIFT",
    /* 2B */ "VK_OEM_5",
    /* 2C */ "0x5A",  // VK_Z
    /* 2D */ "0x58",  // VK_X
    /* 2E */ "0x43",  // VK_C
    /* 2F */ "0x56",  // VK_V
    /* 30 */ "0x42",  // VK_B
    /* 31 */ "0x4E",  // VK_N
    /* 32 */ "0x4D",  // VK_M
    /* 33 */ "VK_OEM_COMMA",
    /* 34 */ "VK_OEM_PERIOD",
    /* 35 */ "VK_OEM_2",
    /* 36 */ "VK_RSHIFT",
    /* 37 */ "VK_MULTIPLY",
    /* 38 */ "VK_LMENU",
    /* 39 */ "VK_SPACE",
    /* 3A */ "VK_CAPITAL",
    /* 3B */ "VK_F1",
    /* 3C */ "VK_F2",
    /* 3D */ "VK_F3",
    /* 3E */ "VK_F4",
    /* 3F */ "VK_F5",
    /* 40 */ "VK_F6",
    /* 41 */ "VK_F7",
    /* 42 */ "VK_F8",
    /* 43 */ "VK_F9",
    /* 44 */ "VK_F10",
    /* 45 */ "VK_NUMLOCK | KBDEXT",
    /* 46 */ "VK_SCROLL",
    /* 47 */ "VK_NUMPAD7 | KBDNUMPAD | KBDSPECIAL",
    /* 48 */ "VK_NUMPAD8 | KBDNUMPAD | KBDSPECIAL",
    /* 49 */ "VK_NUMPAD9 | KBDNUMPAD | KBDSPECIAL",
    /* 4A */ "VK_SUBTRACT",
    /* 4B */ "VK_NUMPAD4 | KBDNUMPAD | KBDSPECIAL",
    /* 4C */ "VK_NUMPAD5 | KBDNUMPAD | KBDSPECIAL",
    /* 4D */ "VK_NUMPAD6 | KBDNUMPAD | KBDSPECIAL",
    /* 4E */ "VK_ADD",
    /* 4F */ "VK_NUMPAD1 | KBDNUMPAD | KBDSPECIAL",
    /* 50 */ "VK_NUMPAD2 | KBDNUMPAD | KBDSPECIAL",
    /* 51 */ "VK_NUMPAD3 | KBDNUMPAD | KBDSPECIAL",
    /* 52 */ "VK_NUMPAD0 | KBDNUMPAD | KBDSPECIAL",
    /* 53 */ "VK_DECIMAL | KBDNUMPAD | KBDSPECIAL",
    /* 54 */ "0",
    /* 55 */ "0",
    /* 56 */ "VK_OEM_102",
    /* 57 */ "VK_F11",
    /* 58 */ "VK_F12",
    /* 59 */ "0",
    /* 5A */ "0",
    /* 5B */ "0",
    /* 5C */ "0",
    /* 5D */ "0",
    /* 5E */ "0",
    /* 5F */ "0",
    /* 60 */ "0",
    /* 61 */ "0",
    /* 62 */ "0",
    /* 63 */ "0",
    /* 64 */ "VK_F13",
    /* 65 */ "VK_F14",
    /* 66 */ "VK_F15",
    /* 67 */ "VK_F16",
    /* 68 */ "VK_F17",
    /* 69 */ "VK_F18",
    /* 6A */ "VK_F19",
    /* 6B */ "VK_F20",
    /* 6C */ "VK_F21",
    /* 6D */ "VK_F22",
    /* 6E */ "VK_F23",
    /* 6F */ "0",
    /* 70 */ "0",
    /* 71 */ "0",
    /* 72 */ "0",
    /* 73 */ "0",
    /* 74 */ "0",
    /* 75 */ "0",
    /* 76 */ "VK_F24",
    /* 77 */ "0",
    /* 78 */ "0",
    /* 79 */ "0",
    /* 7A */ "0",
    /* 7B */ "0",
    /* 7C */ "0",
    /* 7D */ "0",
    /* 7E */ "0",
    /* 7F */ "0",
];

static E0_TABLE: &[(u8, &str)] = &[
    (0x10, "VK_MEDIA_PREV_TRACK"),
    (0x19, "VK_MEDIA_NEXT_TRACK"),
    (0x1C, "VK_RETURN  | KBDEXT"),
    (0x1D, "VK_RCONTROL"),
    (0x20, "VK_VOLUME_MUTE"),
    (0x21, "VK_LAUNCH_APP2"),
    (0x22, "VK_MEDIA_PLAY_PAUSE"),
    (0x24, "VK_MEDIA_STOP"),
    (0x2E, "VK_VOLUME_DOWN"),
    (0x30, "VK_VOLUME_UP"),
    (0x32, "VK_BROWSER_HOME"),
    (0x35, "VK_DIVIDE   | KBDEXT"),
    (0x37, "VK_SNAPSHOT"),
    (0x38, "VK_RMENU"),
    (0x47, "VK_HOME     | KBDEXT"),
    (0x48, "VK_UP       | KBDEXT"),
    (0x49, "VK_PRIOR    | KBDEXT"),
    (0x4B, "VK_LEFT     | KBDEXT"),
    (0x4D, "VK_RIGHT    | KBDEXT"),
    (0x4F, "VK_END      | KBDEXT"),
    (0x50, "VK_DOWN     | KBDEXT"),
    (0x51, "VK_NEXT     | KBDEXT"),
    (0x52, "VK_INSERT   | KBDEXT"),
    (0x53, "VK_DELETE   | KBDEXT"),
    (0x5B, "VK_LWIN     | KBDEXT"),
    (0x5C, "VK_RWIN     | KBDEXT"),
    (0x5D, "VK_APPS     | KBDEXT"),
    (0x5F, "VK_SLEEP"),
    (0x65, "VK_BROWSER_SEARCH"),
    (0x66, "VK_BROWSER_FAVORITES"),
    (0x67, "VK_BROWSER_REFRESH"),
    (0x68, "VK_BROWSER_STOP"),
    (0x69, "VK_BROWSER_FORWARD"),
    (0x6A, "VK_BROWSER_BACK"),
    (0x6B, "VK_LAUNCH_APP1"),
    (0x6C, "VK_LAUNCH_MAIL"),
    (0x6D, "VK_LAUNCH_MEDIA_SELECT"),
];

static KEY_NAMES: &[(u8, &str)] = &[
    (0x01, "Esc"),
    (0x0e, "Backspace"),
    (0x0f, "Tab"),
    (0x1c, "Enter"),
    (0x1d, "Ctrl"),
    (0x2a, "Shift"),
    (0x36, "Right Shift"),
    (0x37, "Num *"),
    (0x38, "Alt"),
    (0x39, "Space"),
    (0x3a, "Caps Lock"),
    (0x3b, "F1"),
    (0x3c, "F2"),
    (0x3d, "F3"),
    (0x3e, "F4"),
    (0x3f, "F5"),
    (0x40, "F6"),
    (0x41, "F7"),
    (0x42, "F8"),
    (0x43, "F9"),
    (0x44, "F10"),
    (0x45, "Pause"),
    (0x46, "Scroll Lock"),
    (0x47, "Num 7"),
    (0x48, "Num 8"),
    (0x49, "Num 9"),
    (0x4a, "Num -"),
    (0x4b, "Num 4"),
    (0x4c, "Num 5"),
    (0x4d, "Num 6"),
    (0x4e, "Num +"),
    (0x4f, "Num 1"),
    (0x50, "Num 2"),
    (0x51, "Num 3"),
    (0x52, "Num 0"),
    (0x53, "Num ."),
    (0x57, "F11"),
    (0x58, "F12"),
];

static KEY_NAMES_EXT: &[(u8, &str)] = &[
    (0x1c, "Num Enter"),
    (0x1d, "Right Ctrl"),
    (0x35, "Num /"),
    (0x37, "Prnt Scrn"),
    (0x38, "Right Alt"),
    (0x45, "Num Lock"),
    (0x46, "Break"),
    (0x47, "Home"),
    (0x48, "Up"),
    (0x49, "Page Up"),
    (0x4b, "Left"),
    (0x4d, "Right"),
    (0x4f, "End"),
    (0x50, "Down"),
    (0x51, "Page Down"),
    (0x52, "Insert"),
    (0x53, "Delete"),
];

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_layout(json: &str) -> Layout {
        serde_json::from_str(json).expect("parse layout JSON")
    }

    fn check_c_source(c: &str, dll_name: &str) {
        assert!(c.contains("KbdLayerDescriptor"), "{dll_name}: missing KbdLayerDescriptor export");
        assert!(c.contains("KBDTABLES"), "{dll_name}: missing KBDTABLES");
        assert!(c.contains("ausVK"), "{dll_name}: missing ausVK");
        assert!(c.contains("aVkToWch"), "{dll_name}: missing aVkToWch");
        assert!(c.contains("CharModifiers"), "{dll_name}: missing CharModifiers");
        assert!(c.contains("#include <kbd.h>"), "{dll_name}: missing kbd.h include");
        // Control-key tables are required for Chromium/Electron compatibility:
        // without VK_RETURN / VK_BACK / VK_TAB in a VK_TO_WCHARS table,
        // ToUnicodeEx returns 0 and Chromium-based apps fail to handle these keys.
        assert!(c.contains("aVkToWch1"), "{dll_name}: missing aVkToWch1 (control keys + numpad)");
        assert!(c.contains("aVkToWch2"), "{dll_name}: missing aVkToWch2 (RETURN/BACK/ESC/CANCEL)");
        assert!(c.contains("VK_RETURN"), "{dll_name}: VK_RETURN must be mapped");
        assert!(c.contains("VK_BACK"), "{dll_name}: VK_BACK must be mapped");
        assert!(c.contains("VK_TAB"), "{dll_name}: VK_TAB must be mapped");
    }

    #[test]
    fn generates_c_us_qwerty() {
        let json = include_str!("../../../layouts/apple-us-qwerty.json");
        let layout = parse_layout(json);
        let c = generate_kbd_c(&layout);
        check_c_source(&c, "kbdaplus");
        assert!(c.contains("aVkToWch5"), "kbdaplus: expected 5-column table");
        assert!(c.contains("KLLF_ALTGR"), "kbdaplus: expected KLLF_ALTGR flag");
        assert!(c.contains("aDeadKey"), "kbdaplus: expected dead key table");
        assert!(c.contains("DEADTRANS"), "kbdaplus: expected DEADTRANS entries");
    }

    #[test]
    fn generates_c_fr_azerty() {
        let json = include_str!("../../../layouts/apple-fr-azerty.json");
        let layout = parse_layout(json);
        let c = generate_kbd_c(&layout);
        check_c_source(&c, "kbdaplfr");
        assert!(c.contains("aVkToWch5"), "kbdaplfr: expected 5-column table (has AltGr)");
    }

    #[test]
    fn generates_c_de_qwertz() {
        let json = include_str!("../../../layouts/apple-de-qwertz.json");
        let layout = parse_layout(json);
        let c = generate_kbd_c(&layout);
        check_c_source(&c, "kbdaplde");
    }

    #[test]
    fn generates_c_es_qwerty() {
        let json = include_str!("../../../layouts/apple-es-qwerty.json");
        let layout = parse_layout(json);
        let c = generate_kbd_c(&layout);
        check_c_source(&c, "kbdaples");
    }

    #[test]
    fn generates_c_uk_qwerty() {
        let json = include_str!("../../../layouts/apple-uk-qwerty.json");
        let layout = parse_layout(json);
        let c = generate_kbd_c(&layout);
        check_c_source(&c, "kbdapluk");
    }

    #[test]
    fn generates_c_it_qwerty() {
        let json = include_str!("../../../layouts/apple-it-qwerty.json");
        let layout = parse_layout(json);
        let c = generate_kbd_c(&layout);
        check_c_source(&c, "kbdaplit");
    }

    #[test]
    fn dead_key_table_empty_when_no_dead_keys() {
        let mut json_val: serde_json::Value =
            serde_json::from_str(include_str!("../../../layouts/apple-us-qwerty.json")).unwrap();
        json_val["deadKeys"] = serde_json::json!({});
        for key_obj in json_val["keys"].as_object_mut().unwrap().values_mut() {
            for field in &["base", "shift", "altgr", "altgrShift"] {
                if let Some(v) = key_obj.get(*field).and_then(|v| v.as_str()) {
                    if v.ends_with('@') {
                        let new_val = v.trim_end_matches('@').to_string();
                        key_obj[field] = serde_json::Value::String(new_val);
                    }
                }
            }
        }
        let layout: Layout = serde_json::from_value(json_val).unwrap();
        let c = generate_kbd_c(&layout);
        assert!(!c.contains("DEADTRANS"), "no dead keys expected");
    }
}
