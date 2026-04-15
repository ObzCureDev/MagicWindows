use super::Layout;

/// Generate a complete .klc (Microsoft Keyboard Layout Creator) file from a
/// `Layout` definition.  The returned string is ready to be written to disk.
pub fn generate_klc(layout: &Layout) -> String {
    let mut out = String::with_capacity(8 * 1024);
    let layout_name = layout.name.get("en").map(|s| s.as_str()).unwrap_or(&layout.id);

    // ── Header ──────────────────────────────────────────────────────────
    push_line(&mut out, &format!("KBD\t{}\t\"{}\"", layout.dll_name, layout_name));
    push_line(&mut out, "");
    push_line(&mut out, "COPYRIGHT\t\"(c) MagicWindows Contributors\"");
    push_line(&mut out, "COMPANY\t\"MagicWindows\"");
    push_line(&mut out, "");
    push_line(&mut out, &format!("LOCALENAME\t\"{}\"", layout.locale));
    push_line(&mut out, &format!("LOCALEID\t\"{}\"", layout.locale_id));
    push_line(&mut out, "");
    push_line(&mut out, "VERSION\t1.0");
    push_line(&mut out, "");

    // ── SHIFTSTATE ──────────────────────────────────────────────────────
    push_line(&mut out, "SHIFTSTATE");
    push_line(&mut out, "");
    push_line(&mut out, "0\t//Column 4");
    push_line(&mut out, "1\t//Column 5 : Shft");
    push_line(&mut out, "2\t//Column 6 :       Ctrl");
    push_line(&mut out, "6\t//Column 7 :       Ctrl Alt  (= AltGr)");
    push_line(&mut out, "7\t//Column 8 : Shft  Ctrl Alt");
    push_line(&mut out, "");

    // ── LAYOUT ──────────────────────────────────────────────────────────
    push_line(&mut out, "LAYOUT\t\t;an extra '@' sign follows a dead key value");
    push_line(&mut out, "");
    push_line(
        &mut out,
        "//SC\tVK_\t\tCap\t0\t1\t2\t6\t7",
    );
    push_line(
        &mut out,
        "//--\t----\t\t----\t----\t----\t----\t----\t----",
    );
    push_line(&mut out, "");

    // Sort keys by scancode (hex string -> numeric for stable order).
    let mut scancodes: Vec<&String> = layout.keys.keys().collect();
    scancodes.sort_by_key(|sc| u32::from_str_radix(sc, 16).unwrap_or(0));

    for sc in &scancodes {
        let km = &layout.keys[*sc];
        let line = format!(
            "{}\t{}\t\t{}\t{}\t{}\t{}\t{}\t{}",
            sc, km.vk, km.cap, km.base, km.shift, km.ctrl, km.altgr, km.altgr_shift,
        );
        push_line(&mut out, &line);
    }

    push_line(&mut out, "");

    // ── DEADKEY sections ────────────────────────────────────────────────
    // Sort dead keys for reproducible output.
    let mut dk_codes: Vec<&String> = layout.dead_keys.keys().collect();
    dk_codes.sort();

    for dk_code in &dk_codes {
        let dk = &layout.dead_keys[*dk_code];
        push_line(&mut out, &format!("DEADKEY\t{}", dk_code));
        push_line(&mut out, "");

        let mut combos: Vec<(&String, &String)> = dk.combinations.iter().collect();
        combos.sort_by_key(|(k, _)| *k);

        for (base_char, result_char) in combos {
            // Try to add a helpful comment with the Unicode character.
            let comment = codepoint_to_char(result_char)
                .map(|c| format!("\t// {}", c))
                .unwrap_or_default();
            push_line(
                &mut out,
                &format!("{}\t{}{}", base_char, result_char, comment),
            );
        }
        push_line(&mut out, "");
    }

    // ── KEYNAME ─────────────────────────────────────────────────────────
    push_line(&mut out, "KEYNAME");
    push_line(&mut out, "");
    for (sc, name) in KEYNAMES {
        push_line(&mut out, &format!("{}\t{}", sc, name));
    }
    push_line(&mut out, "");

    // ── KEYNAME_EXT ─────────────────────────────────────────────────────
    push_line(&mut out, "KEYNAME_EXT");
    push_line(&mut out, "");
    for (sc, name) in KEYNAMES_EXT {
        push_line(&mut out, &format!("{}\t{}", sc, name));
    }
    push_line(&mut out, "");

    // ── KEYNAME_DEAD ────────────────────────────────────────────────────
    push_line(&mut out, "KEYNAME_DEAD");
    push_line(&mut out, "");
    for dk_code in &dk_codes {
        let dk = &layout.dead_keys[*dk_code];
        push_line(&mut out, &format!("{}\t\"{}\"", dk_code, dk.name));
    }
    push_line(&mut out, "");

    // ── DESCRIPTIONS ────────────────────────────────────────────────────
    push_line(&mut out, "DESCRIPTIONS");
    push_line(&mut out, "");
    push_line(&mut out, &format!("0409\t{}", layout_name));
    push_line(&mut out, "");

    // ── LANGUAGENAMES ───────────────────────────────────────────────────
    push_line(&mut out, "LANGUAGENAMES");
    push_line(&mut out, "");
    push_line(&mut out, &format!("0409\t{}", locale_display_name(&layout.locale)));
    push_line(&mut out, "");

    // ── End ─────────────────────────────────────────────────────────────
    push_line(&mut out, "ENDKBD");

    out
}

fn push_line(out: &mut String, line: &str) {
    out.push_str(line);
    out.push('\n');
}

/// Try to convert a 4-hex-digit codepoint string to a char.
fn codepoint_to_char(hex: &str) -> Option<char> {
    u32::from_str_radix(hex, 16)
        .ok()
        .and_then(char::from_u32)
}

/// Map a BCP-47 locale tag to a human-readable language name.
fn locale_display_name(locale: &str) -> &'static str {
    match locale {
        "fr-FR" => "French (France)",
        "fr-CA" => "French (Canada)",
        "fr-BE" => "French (Belgium)",
        "fr-CH" => "French (Switzerland)",
        "de-DE" => "German (Germany)",
        "de-CH" => "German (Switzerland)",
        "de-AT" => "German (Austria)",
        "en-US" => "English (United States)",
        "en-GB" => "English (United Kingdom)",
        "es-ES" => "Spanish (Spain)",
        "it-IT" => "Italian (Italy)",
        "pt-BR" => "Portuguese (Brazil)",
        "pt-PT" => "Portuguese (Portugal)",
        "nl-NL" => "Dutch (Netherlands)",
        "sv-SE" => "Swedish (Sweden)",
        "da-DK" => "Danish (Denmark)",
        "nb-NO" => "Norwegian Bokmal (Norway)",
        "fi-FI" => "Finnish (Finland)",
        "pl-PL" => "Polish (Poland)",
        "ja-JP" => "Japanese (Japan)",
        _ => "English (United States)",
    }
}

// ── Standard KLC key-name tables ────────────────────────────────────────

static KEYNAMES: &[(&str, &str)] = &[
    ("01", "Esc"),
    ("0e", "Backspace"),
    ("0f", "Tab"),
    ("1c", "Enter"),
    ("1d", "Ctrl"),
    ("2a", "Shift"),
    ("36", "\"Right Shift\""),
    ("37", "\"Num *\""),
    ("38", "Alt"),
    ("39", "Space"),
    ("3a", "\"Caps Lock\""),
    ("3b", "F1"),
    ("3c", "F2"),
    ("3d", "F3"),
    ("3e", "F4"),
    ("3f", "F5"),
    ("40", "F6"),
    ("41", "F7"),
    ("42", "F8"),
    ("43", "F9"),
    ("44", "F10"),
    ("45", "Pause"),
    ("46", "\"Scroll Lock\""),
    ("47", "\"Num 7\""),
    ("48", "\"Num 8\""),
    ("49", "\"Num 9\""),
    ("4a", "\"Num -\""),
    ("4b", "\"Num 4\""),
    ("4c", "\"Num 5\""),
    ("4d", "\"Num 6\""),
    ("4e", "\"Num +\""),
    ("4f", "\"Num 1\""),
    ("50", "\"Num 2\""),
    ("51", "\"Num 3\""),
    ("52", "\"Num 0\""),
    ("53", "\"Num .\""),
    ("57", "F11"),
    ("58", "F12"),
];

static KEYNAMES_EXT: &[(&str, &str)] = &[
    ("1c", "\"Num Enter\""),
    ("1d", "\"Right Ctrl\""),
    ("35", "\"Num /\""),
    ("37", "\"Prnt Scrn\""),
    ("38", "\"Right Alt\""),
    ("45", "\"Num Lock\""),
    ("46", "Break"),
    ("47", "Home"),
    ("48", "Up"),
    ("49", "\"Page Up\""),
    ("4b", "Left"),
    ("4d", "Right"),
    ("4f", "End"),
    ("50", "Down"),
    ("51", "\"Page Down\""),
    ("52", "Insert"),
    ("53", "Delete"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keyboard::Layout;

    #[test]
    fn generates_non_empty_klc() {
        let json = include_str!("../../../layouts/apple-fr-azerty.json");
        let layout: Layout = serde_json::from_str(json).expect("parse layout JSON");
        let klc = generate_klc(&layout);

        assert!(klc.contains("KBD\tkbdaplfr"));
        assert!(klc.contains("LOCALENAME\t\"fr-FR\""));
        assert!(klc.contains("SHIFTSTATE"));
        assert!(klc.contains("LAYOUT"));
        assert!(klc.contains("DEADKEY\t005e"));
        assert!(klc.contains("DEADKEY\t00a8"));
        assert!(klc.contains("DEADKEY\t0060"));
        assert!(klc.contains("DEADKEY\t007e"));
        assert!(klc.contains("KEYNAME"));
        assert!(klc.contains("KEYNAME_EXT"));
        assert!(klc.contains("KEYNAME_DEAD"));
        assert!(klc.contains("ENDKBD"));
    }
}
