/// MagicWindows build script.
///
/// On Windows this script:
///   1. Reads every `layouts/*.json` file (skipping `schema.json`).
///   2. Generates a C source file for each layout using the same logic as
///      `kbd_c.rs` (duplicated here so the build script is self-contained –
///      build scripts cannot import from the crate they build).
///   3. Compiles each C file into a keyboard layout DLL using `cl.exe` and
///      `link.exe` from the MSVC toolchain that Rust itself requires.
///   4. Copies the compiled DLLs to `OUT_DIR/kbd_dlls/` where Tauri can
///      pick them up as bundled resources.
///
/// On non-Windows targets the DLL compilation step is skipped (the DLLs
/// can only be installed on Windows anyway).
use std::{
    env,
    fs,
    path::{Path, PathBuf},
};

// ── serde types (duplicated from mod.rs because build scripts are standalone)

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeyMapping {
    vk: String,
    cap: String,
    base: String,
    shift: String,
    #[serde(default)]
    ctrl: String,
    #[serde(default)]
    altgr: String,
    #[serde(default, rename = "altgrShift")]
    altgr_shift: String,
}

#[derive(serde::Deserialize)]
struct DeadKey {
    name: String,
    combinations: std::collections::HashMap<String, String>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Layout {
    id: String,
    name: std::collections::HashMap<String, String>,
    locale: String,
    dll_name: String,
    #[serde(default)]
    dead_keys: std::collections::HashMap<String, DeadKey>,
    keys: std::collections::HashMap<String, KeyMapping>,
}

fn main() {
    // Tell Cargo to re-run this script if any layout JSON changes.
    println!("cargo:rerun-if-changed=../layouts");
    println!("cargo:rerun-if-changed=build.rs");

    // Compile keyboard DLLs BEFORE tauri_build::build() so that the
    // kbd_dlls/ directory is populated when tauri-build validates the
    // resource glob in tauri.conf.json.
    #[cfg(target_os = "windows")]
    if let Err(e) = compile_keyboard_dlls() {
        // Emit a warning rather than a hard error so that `cargo check` and
        // `cargo test` still work even without a complete MSVC environment.
        println!("cargo:warning=Keyboard DLL compilation skipped: {e}");
    }

    tauri_build::build();
}

#[cfg(target_os = "windows")]
fn compile_keyboard_dlls() -> Result<(), String> {
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").map_err(|_| "CARGO_MANIFEST_DIR not set")?,
    );
    let layouts_dir = manifest_dir.parent()
        .ok_or("cannot get parent of manifest dir")?
        .join("layouts");
    let out_dir = PathBuf::from(
        env::var("OUT_DIR").map_err(|_| "OUT_DIR not set")?,
    );

    // Intermediate directory for .c/.obj files (inside OUT_DIR, never committed).
    let c_build_dir = out_dir.join("kbd_c_build");
    fs::create_dir_all(&c_build_dir)
        .map_err(|e| format!("create kbd_c_build dir: {e}"))?;

    // Final DLL destination: src-tauri/kbd_dlls/  (committed alongside the
    // source so that Tauri can glob them as resources in tauri.conf.json).
    let dll_dest_dir = manifest_dir.join("kbd_dlls");
    fs::create_dir_all(&dll_dest_dir)
        .map_err(|e| format!("create kbd_dlls dir: {e}"))?;

    // Locate the MSVC toolchain.
    let msvc = MsvcPaths::find()?;

    // Process each layout JSON.
    let entries = fs::read_dir(&layouts_dir)
        .map_err(|e| format!("read layouts dir: {e}"))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("dir entry: {e}"))?;
        let path = entry.path();
        if path.extension().map(|e| e != "json").unwrap_or(true) {
            continue;
        }
        if path.file_name().map(|n| n == "schema.json").unwrap_or(false) {
            continue;
        }

        let json = fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        let layout: Layout = serde_json::from_str(&json)
            .map_err(|e| format!("parse {}: {e}", path.display()))?;

        let dll_path = dll_dest_dir.join(format!("{}.dll", layout.dll_name));

        // Skip recompilation if the DLL is already up-to-date.
        // (A proper incremental check could compare mtimes, but for now we
        // always recompile when the build script runs – Cargo reruns this
        // script whenever the layouts/ dir changes.)

        let c_src = generate_kbd_c_build(&layout);
        let c_path = c_build_dir.join(format!("{}.c", layout.dll_name));
        fs::write(&c_path, &c_src)
            .map_err(|e| format!("write {}: {e}", c_path.display()))?;

        compile_dll(&msvc, &c_path, &dll_path, &c_build_dir)
            .map_err(|e| {
                println!(
                    "cargo:warning=Failed to compile DLL for {}: {e}",
                    layout.dll_name
                );
                e
            })?;

        println!("cargo:warning=Compiled keyboard DLL: {}", dll_path.display());
    }

    Ok(())
}

/// Compile a single keyboard layout C file into a DLL.
#[cfg(target_os = "windows")]
fn compile_dll(
    msvc: &MsvcPaths,
    c_path: &Path,
    dll_path: &Path,
    obj_dir: &Path,
) -> Result<(), String> {
    use std::process::Command;

    // cl.exe compiles and (with /LD) links in one step, but we split into
    // two steps for better error isolation.

    let obj_path = obj_dir.join(
        c_path.file_stem().map(|s| format!("{}.obj", s.to_string_lossy()))
            .unwrap_or_else(|| "kbd.obj".into()),
    );

    // ── Step 1: Compile C → OBJ ─────────────────────────────────────────
    let mut cl = Command::new(&msvc.cl);
    cl.env("INCLUDE", &msvc.include_dirs)
      .env("LIB", &msvc.lib_dirs)
      .env("PATH", &msvc.path);

    // Determine target architecture for the preprocessor.
    // The keyboard DLL is always compiled for the host x64 architecture.
    // winnt.h requires one of _X86_, _AMD64_, _ARM_, or _ARM64_ to be defined.
    cl.arg("/nologo")
      .arg("/W3")
      .arg("/Zl")          // Omit default library name from .obj
      .arg("/c")           // Compile only (no link)
      .arg("/GS-")         // Disable buffer security checks (no CRT)
      .arg("/D_AMD64_=1")  // Tell headers we're targeting x64
      .arg("/DWIN32")
      .arg("/D_WINDOWS")
      .arg(format!("/Fo{}", obj_path.display()))
      .arg(c_path);

    let output = cl.output()
        .map_err(|e| format!("spawn cl.exe: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "cl.exe failed compiling {}:\nstdout: {stdout}\nstderr: {stderr}",
            c_path.display()
        ));
    }

    // ── Step 2: Link OBJ → DLL ──────────────────────────────────────────
    let dll_stem = dll_path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "kbd".into());
    let def_path = obj_dir.join(format!("{dll_stem}.def"));
    // Write a minimal .def file so the export is clean without needing
    // __declspec(dllexport) on older toolchain versions.
    fs::write(
        &def_path,
        format!("LIBRARY {dll_stem}\nEXPORTS\n    KbdLayerDescriptor\n"),
    )
    .map_err(|e| format!("write def file: {e}"))?;

    let mut link = Command::new(&msvc.link);
    link.env("LIB", &msvc.lib_dirs)
        .env("PATH", &msvc.path);

    link.arg("/nologo")
        .arg("/DLL")
        .arg("/NOENTRY")    // No entry point (no CRT)
        .arg("/NODEFAULTLIB")
        .arg(format!("/DEF:{}", def_path.display()))
        .arg(format!("/OUT:{}", dll_path.display()))
        .arg(&obj_path)
        .arg("user32.lib");

    let link_output = link.output()
        .map_err(|e| format!("spawn link.exe: {e}"))?;
    if !link_output.status.success() {
        let stderr = String::from_utf8_lossy(&link_output.stderr);
        let stdout = String::from_utf8_lossy(&link_output.stdout);
        return Err(format!(
            "link.exe failed linking {}:\nstdout: {stdout}\nstderr: {stderr}",
            dll_path.display()
        ));
    }

    Ok(())
}

/// Paths to the MSVC compiler and linker, plus the include/lib search paths.
#[cfg(target_os = "windows")]
struct MsvcPaths {
    cl: PathBuf,
    link: PathBuf,
    /// Semicolon-separated include directories (INCLUDE env var for cl.exe).
    include_dirs: String,
    /// Semicolon-separated library directories (LIB env var for link.exe).
    lib_dirs: String,
    /// PATH that contains the MSVC bin dir (so cl.exe can find c1.dll etc.).
    path: String,
}

#[cfg(target_os = "windows")]
impl MsvcPaths {
    /// Discover the MSVC toolchain.
    ///
    /// Rust on Windows is always built against an MSVC toolchain, so `cl.exe`
    /// and `link.exe` must exist somewhere.  We probe the common VS 2022 and
    /// VS 2019 install locations and fall back to searching via `vswhere.exe`.
    fn find() -> Result<Self, String> {
        // Try vswhere first (most reliable method).
        if let Ok(paths) = Self::find_via_vswhere() {
            return Ok(paths);
        }
        // Fall back to well-known paths.
        Self::find_via_known_paths()
    }

    fn find_via_vswhere() -> Result<Self, String> {
        use std::process::Command;

        let vswhere_candidates = [
            r"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe",
            r"C:\Program Files\Microsoft Visual Studio\Installer\vswhere.exe",
        ];

        let vswhere = vswhere_candidates
            .iter()
            .find(|p| Path::new(p).exists())
            .ok_or("vswhere.exe not found")?;

        let out = Command::new(vswhere)
            .args([
                "-latest",
                "-requires",
                "Microsoft.VisualCpp.Tools.HostX64.TargetX64",
                "-property",
                "installationPath",
            ])
            .output()
            .map_err(|e| format!("vswhere: {e}"))?;

        if !out.status.success() {
            return Err("vswhere returned non-zero".into());
        }

        let install_root = String::from_utf8_lossy(&out.stdout)
            .trim()
            .to_string();
        if install_root.is_empty() {
            return Err("vswhere returned empty path".into());
        }

        Self::from_vs_install_root(&install_root)
    }

    fn find_via_known_paths() -> Result<Self, String> {
        let candidates: &[&str] = &[
            r"C:\Program Files\Microsoft Visual Studio\2022\Community",
            r"C:\Program Files\Microsoft Visual Studio\2022\Professional",
            r"C:\Program Files\Microsoft Visual Studio\2022\Enterprise",
            r"C:\Program Files\Microsoft Visual Studio\2022\BuildTools",
            r"C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools",
            r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Community",
            r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Professional",
            r"C:\Program Files (x86)\Microsoft Visual Studio\2019\Enterprise",
        ];

        for root in candidates {
            if Path::new(root).exists() {
                if let Ok(paths) = Self::from_vs_install_root(root) {
                    return Ok(paths);
                }
            }
        }

        Err("No MSVC installation found".into())
    }

    fn from_vs_install_root(root: &str) -> Result<Self, String> {
        // Find the MSVC version directory.
        let vc_tools = PathBuf::from(root).join("VC").join("Tools").join("MSVC");
        if !vc_tools.exists() {
            return Err(format!("{} does not contain VC/Tools/MSVC", root));
        }

        // Pick the latest version directory.
        let mut versions: Vec<PathBuf> = fs::read_dir(&vc_tools)
            .map_err(|e| format!("read {}: {e}", vc_tools.display()))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        versions.sort();
        let msvc_ver = versions
            .last()
            .ok_or_else(|| format!("no MSVC version in {}", vc_tools.display()))?
            .clone();

        // Choose the host/target bin directory.
        // For x64 Rust targets we want Hostx64/x64.
        let bin_dir = msvc_ver.join("bin").join("Hostx64").join("x64");
        if !bin_dir.exists() {
            return Err(format!("{} does not exist", bin_dir.display()));
        }

        let cl = bin_dir.join("cl.exe");
        let link = bin_dir.join("link.exe");
        if !cl.exists() {
            return Err(format!("{} not found", cl.display()));
        }
        if !link.exists() {
            return Err(format!("{} not found", link.display()));
        }

        // Build the INCLUDE path: MSVC headers + Windows SDK headers.
        let msvc_include = msvc_ver.join("include");
        let (sdk_include, sdk_lib) = find_windows_sdk()?;

        let include_dirs = format!(
            "{};{};{};{}",
            msvc_include.display(),
            sdk_include.join("um").display(),
            sdk_include.join("shared").display(),
            sdk_include.join("ucrt").display(),
        );

        // Build the LIB path: MSVC libs + Windows SDK libs (x64).
        let msvc_lib = msvc_ver.join("lib").join("x64");
        let lib_dirs = format!(
            "{};{};{}",
            msvc_lib.display(),
            sdk_lib.join("um").join("x64").display(),
            sdk_lib.join("ucrt").join("x64").display(),
        );

        // PATH must include the bin dir so cl.exe can find its DLLs (c1.dll etc.).
        let system_path = env::var("PATH").unwrap_or_default();
        let path = format!("{};{system_path}", bin_dir.display());

        Ok(Self {
            cl,
            link,
            include_dirs,
            lib_dirs,
            path,
        })
    }
}

/// Find the latest Windows SDK include and lib directories.
#[cfg(target_os = "windows")]
fn find_windows_sdk() -> Result<(PathBuf, PathBuf), String> {
    let sdk_root = PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10");
    if !sdk_root.exists() {
        return Err("Windows Kits 10 not found at default location".into());
    }

    let include_root = sdk_root.join("Include");
    let lib_root = sdk_root.join("Lib");

    // Pick the latest versioned directory.
    let latest_include = latest_versioned_dir(&include_root)?;
    let latest_lib = latest_versioned_dir(&lib_root)?;

    Ok((latest_include, latest_lib))
}

/// Return the highest-version sub-directory of `dir` (e.g. `10.0.22621.0`).
#[cfg(target_os = "windows")]
fn latest_versioned_dir(dir: &Path) -> Result<PathBuf, String> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("read {}: {e}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    entries.sort();
    entries
        .last()
        .cloned()
        .ok_or_else(|| format!("no versioned dirs in {}", dir.display()))
}

// ── C source generator (self-contained copy for use in build.rs) ────────────
//
// This is intentionally a simplified, self-contained duplicate of the logic in
// `src/keyboard/kbd_c.rs`.  Build scripts cannot import from the crate they
// build, so we cannot share code at compile time.

fn generate_kbd_c_build(layout: &Layout) -> String {
    let mut out = String::with_capacity(32 * 1024);
    let has_altgr = build_layout_has_altgr(layout);
    let has_dead_keys = !layout.dead_keys.is_empty();
    let entries = build_collect_entries(layout);

    build_emit_header(&mut out, layout);
    build_emit_vsc_to_vk(&mut out, layout);
    build_emit_e0_e1(&mut out);
    build_emit_wchar_tables(&mut out, &entries, has_altgr);
    build_emit_dead_keys(&mut out, layout, has_dead_keys);
    build_emit_key_names(&mut out);
    build_emit_key_names_ext(&mut out);
    build_emit_dead_key_names(&mut out, layout, has_dead_keys);
    build_emit_modifiers(&mut out, has_altgr);
    build_emit_kbd_tables(&mut out, has_altgr);
    build_emit_export(&mut out);
    out
}

fn push_ln(out: &mut String, s: &str) {
    out.push_str(s);
    out.push('\n');
}

// --- CharVal duplicate for build.rs ---

#[derive(Clone)]
enum CharVal {
    None,
    Char(u32),
    Dead(u32),
}

impl CharVal {
    fn parse(s: &str) -> Self {
        let s = s.trim();
        if s == "-1" || s.is_empty() {
            return CharVal::None;
        }
        let (dead, hex) = if s.ends_with('@') {
            (true, &s[..s.len() - 1])
        } else {
            (false, s)
        };
        let cp = u32::from_str_radix(hex, 16).unwrap_or(0);
        if dead { CharVal::Dead(cp) } else { CharVal::Char(cp) }
    }
    fn c_wch(&self) -> String {
        match self {
            CharVal::None => "WCH_NONE".into(),
            CharVal::Char(cp) | CharVal::Dead(cp) => format!("0x{cp:04X}"),
        }
    }
    fn is_dead(&self) -> bool { matches!(self, CharVal::Dead(_)) }
    fn is_none(&self) -> bool { matches!(self, CharVal::None) }
}

struct Entry {
    vk: String,
    cap: String,
    base: CharVal,
    shift: CharVal,
    ctrl: CharVal,
    altgr: CharVal,
    altgr_shift: CharVal,
}

fn build_collect_entries(layout: &Layout) -> Vec<Entry> {
    let mut scancodes: Vec<&String> = layout.keys.keys().collect();
    scancodes.sort_by_key(|sc| u32::from_str_radix(sc, 16).unwrap_or(0));
    scancodes.iter().map(|sc| {
        let km = &layout.keys[*sc];
        Entry {
            vk: km.vk.clone(),
            cap: km.cap.clone(),
            base:       CharVal::parse(&km.base),
            shift:      CharVal::parse(&km.shift),
            ctrl:       CharVal::parse(&km.ctrl),
            altgr:      CharVal::parse(&km.altgr),
            altgr_shift:CharVal::parse(&km.altgr_shift),
        }
    }).collect()
}

fn build_layout_has_altgr(layout: &Layout) -> bool {
    layout.keys.values().any(|km| {
        let ag  = CharVal::parse(&km.altgr);
        let ags = CharVal::parse(&km.altgr_shift);
        !ag.is_none() || !ags.is_none()
    })
}

fn caps_attr_build(cap: &str) -> u8 {
    match cap.trim() {
        "1" => 0x01,
        "4" => 0x02,
        _ => 0x00,
    }
}

/// Map a JSON VK name to a C expression (hex literal for alphanumeric keys,
/// symbolic name for OEM/special keys defined in winuser.h).
fn vk_to_c_build(vk: &str) -> &'static str {
    match vk {
        // Digits (ASCII 0x30-0x39, NOT in winuser.h)
        "VK_0" => "0x30", "VK_1" => "0x31", "VK_2" => "0x32",
        "VK_3" => "0x33", "VK_4" => "0x34", "VK_5" => "0x35",
        "VK_6" => "0x36", "VK_7" => "0x37", "VK_8" => "0x38",
        "VK_9" => "0x39",
        // Letters (ASCII 0x41-0x5A, NOT in winuser.h; VK_F conflicts with kbd.h struct)
        "VK_A" => "0x41", "VK_B" => "0x42", "VK_C" => "0x43",
        "VK_D" => "0x44", "VK_E" => "0x45", "VK_F" => "0x46",
        "VK_G" => "0x47", "VK_H" => "0x48", "VK_I" => "0x49",
        "VK_J" => "0x4A", "VK_K" => "0x4B", "VK_L" => "0x4C",
        "VK_M" => "0x4D", "VK_N" => "0x4E", "VK_O" => "0x4F",
        "VK_P" => "0x50", "VK_Q" => "0x51", "VK_R" => "0x52",
        "VK_S" => "0x53", "VK_T" => "0x54", "VK_U" => "0x55",
        "VK_V" => "0x56", "VK_W" => "0x57", "VK_X" => "0x58",
        "VK_Y" => "0x59", "VK_Z" => "0x5A",
        // OEM and special keys (defined in winuser.h)
        "VK_SPACE"      => "VK_SPACE",
        "VK_OEM_1"      => "VK_OEM_1",
        "VK_OEM_2"      => "VK_OEM_2",
        "VK_OEM_3"      => "VK_OEM_3",
        "VK_OEM_4"      => "VK_OEM_4",
        "VK_OEM_5"      => "VK_OEM_5",
        "VK_OEM_6"      => "VK_OEM_6",
        "VK_OEM_7"      => "VK_OEM_7",
        "VK_OEM_8"      => "VK_OEM_8",
        "VK_OEM_102"    => "VK_OEM_102",
        "VK_OEM_COMMA"  => "VK_OEM_COMMA",
        "VK_OEM_PERIOD" => "VK_OEM_PERIOD",
        "VK_OEM_MINUS"  => "VK_OEM_MINUS",
        "VK_OEM_PLUS"   => "VK_OEM_PLUS",
        _ => "0",
    }
}

/// Base VSC table using hex literals for alphanumeric keys (avoids kbd.h conflicts).
static BUILD_BASE_VSC: &[&str] = &[
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

fn build_emit_header(out: &mut String, layout: &Layout) {
    let name = layout.name.get("en").map_or(layout.id.as_str(), |s| s.as_str());
    push_ln(out, "/*");
    push_ln(out, &format!(" * Keyboard layout: {name}"));
    push_ln(out, &format!(" * Locale:          {}", layout.locale));
    push_ln(out, " * Generated by MagicWindows.  DO NOT EDIT.");
    push_ln(out, " */");
    push_ln(out, "#define KBD_TYPE 4");
    // Include order matters: windef.h before winuser.h; do NOT include
    // windows.h because it pulls in the full Win32 headers which cause
    // name collisions with kbd.h (e.g. VK_F struct vs function-param type).
    push_ln(out, "#include <windef.h>");
    push_ln(out, "#include <winuser.h>");
    push_ln(out, "#include <kbd.h>");
    push_ln(out, "");
    // ALLOC_SECTION_LDATA is a DDK/WDK macro for section placement.
    // For user-mode keyboard layout DLLs it can be defined as empty.
    push_ln(out, "#ifndef ALLOC_SECTION_LDATA");
    push_ln(out, "#define ALLOC_SECTION_LDATA");
    push_ln(out, "#endif");
    push_ln(out, "");
}

fn build_emit_vsc_to_vk(out: &mut String, layout: &Layout) {
    let mut table: Vec<&str> = BUILD_BASE_VSC.to_vec();
    // Pad to 0x80 entries if needed.
    while table.len() < 0x80 {
        table.push("0");
    }
    for (sc_str, km) in &layout.keys {
        let sc = match u32::from_str_radix(sc_str, 16) {
            Ok(v) if (v as usize) < table.len() => v as usize,
            _ => continue,
        };
        table[sc] = vk_to_c_build(&km.vk);
    }
    push_ln(out, "static USHORT ausVK[] = {");
    for (i, vk) in table.iter().enumerate() {
        push_ln(out, &format!("    /* {i:02X} */ {vk},"));
    }
    push_ln(out, "};");
    push_ln(out, "");
}

static BUILD_E0: &[(u8, &str)] = &[
    (0x10,"VK_MEDIA_PREV_TRACK"),
    (0x19,"VK_MEDIA_NEXT_TRACK"),
    (0x1C,"VK_RETURN  | KBDEXT"),
    (0x1D,"VK_RCONTROL"),
    (0x20,"VK_VOLUME_MUTE"),
    (0x21,"VK_LAUNCH_APP2"),
    (0x22,"VK_MEDIA_PLAY_PAUSE"),
    (0x24,"VK_MEDIA_STOP"),
    (0x2E,"VK_VOLUME_DOWN"),
    (0x30,"VK_VOLUME_UP"),
    (0x32,"VK_BROWSER_HOME"),
    (0x35,"VK_DIVIDE   | KBDEXT"),
    (0x37,"VK_SNAPSHOT"),
    (0x38,"VK_RMENU"),
    (0x47,"VK_HOME     | KBDEXT"),
    (0x48,"VK_UP       | KBDEXT"),
    (0x49,"VK_PRIOR    | KBDEXT"),
    (0x4B,"VK_LEFT     | KBDEXT"),
    (0x4D,"VK_RIGHT    | KBDEXT"),
    (0x4F,"VK_END      | KBDEXT"),
    (0x50,"VK_DOWN     | KBDEXT"),
    (0x51,"VK_NEXT     | KBDEXT"),
    (0x52,"VK_INSERT   | KBDEXT"),
    (0x53,"VK_DELETE   | KBDEXT"),
    (0x5B,"VK_LWIN     | KBDEXT"),
    (0x5C,"VK_RWIN     | KBDEXT"),
    (0x5D,"VK_APPS     | KBDEXT"),
    (0x5F,"VK_SLEEP"),
    (0x65,"VK_BROWSER_SEARCH"),
    (0x66,"VK_BROWSER_FAVORITES"),
    (0x67,"VK_BROWSER_REFRESH"),
    (0x68,"VK_BROWSER_STOP"),
    (0x69,"VK_BROWSER_FORWARD"),
    (0x6A,"VK_BROWSER_BACK"),
    (0x6B,"VK_LAUNCH_APP1"),
    (0x6C,"VK_LAUNCH_MAIL"),
    (0x6D,"VK_LAUNCH_MEDIA_SELECT"),
];

fn build_emit_e0_e1(out: &mut String) {
    push_ln(out, "static VSC_VK aE0VscToVk[] = {");
    for &(vsc, vk) in BUILD_E0 {
        push_ln(out, &format!("    {{ 0x{vsc:02X}, {vk} }},"));
    }
    push_ln(out, "    { 0, 0 }");
    push_ln(out, "};");
    push_ln(out, "");
    push_ln(out, "static VSC_VK aE1VscToVk[] = {");
    push_ln(out, "    { 0x1D, VK_PAUSE },");
    push_ln(out, "    { 0, 0 }");
    push_ln(out, "};");
    push_ln(out, "");
}

fn build_emit_wchar_tables(out: &mut String, entries: &[Entry], has_altgr: bool) {
    let n = if has_altgr { "5" } else { "3" };
    push_ln(out, &format!("static VK_TO_WCHARS{n} aVkToWch{n}[] = {{"));
    for e in entries {
        let attr = caps_attr_build(&e.cap);
        let vk_c = vk_to_c_build(&e.vk);
        let any_dead = e.base.is_dead() || e.shift.is_dead()
            || e.altgr.is_dead() || e.altgr_shift.is_dead();

        if has_altgr {
            push_ln(out, &format!(
                "    {{ {:<20} 0x{attr:02X}, {{ {}, {}, {}, {}, {} }} }},",
                format!("{vk_c},"),
                e.base.c_wch(), e.shift.c_wch(), e.ctrl.c_wch(),
                e.altgr.c_wch(), e.altgr_shift.c_wch(),
            ));
            if any_dead {
                let d = |cv: &CharVal| if cv.is_dead() { "WCH_DEAD" } else { "WCH_NONE" };
                push_ln(out, &format!(
                    "    {{ (BYTE)-1,              0x00, {{ {}, {}, WCH_NONE, {}, {} }} }},",
                    d(&e.base), d(&e.shift), d(&e.altgr), d(&e.altgr_shift),
                ));
            }
        } else {
            push_ln(out, &format!(
                "    {{ {:<20} 0x{attr:02X}, {{ {}, {}, {} }} }},",
                format!("{vk_c},"),
                e.base.c_wch(), e.shift.c_wch(), e.ctrl.c_wch(),
            ));
            if any_dead {
                let d = |cv: &CharVal| if cv.is_dead() { "WCH_DEAD" } else { "WCH_NONE" };
                push_ln(out, &format!(
                    "    {{ (BYTE)-1,              0x00, {{ {}, {}, WCH_NONE }} }},",
                    d(&e.base), d(&e.shift),
                ));
            }
        }
    }
    if has_altgr {
        push_ln(out, "    { 0, 0x00, { WCH_NONE, WCH_NONE, WCH_NONE, WCH_NONE, WCH_NONE } }");
    } else {
        push_ln(out, "    { 0, 0x00, { WCH_NONE, WCH_NONE, WCH_NONE } }");
    }
    push_ln(out, "};");
    push_ln(out, "");
}

fn build_emit_dead_keys(out: &mut String, layout: &Layout, has_dead_keys: bool) {
    if !has_dead_keys {
        push_ln(out, "static DEADKEY aDeadKey[] = { { 0, 0, 0 } };");
        push_ln(out, "");
        return;
    }
    push_ln(out, "static DEADKEY aDeadKey[] = {");
    let mut dk_codes: Vec<&String> = layout.dead_keys.keys().collect();
    dk_codes.sort();
    for dk_code in &dk_codes {
        let dk = &layout.dead_keys[*dk_code];
        let accent = u32::from_str_radix(dk_code, 16).unwrap_or(0);
        let mut combos: Vec<(&String, &String)> = dk.combinations.iter().collect();
        combos.sort_by_key(|(k, _)| *k);
        for (base_str, result_str) in &combos {
            let base = u32::from_str_radix(base_str, 16).unwrap_or(0);
            let result = u32::from_str_radix(result_str, 16).unwrap_or(0);
            push_ln(out, &format!(
                "    DEADTRANS( 0x{base:04X}, 0x{accent:04X}, 0x{result:04X}, 0x0000 ),"
            ));
        }
    }
    push_ln(out, "    { 0, 0, 0 }");
    push_ln(out, "};");
    push_ln(out, "");
}

static BUILD_KEY_NAMES: &[(u8, &str)] = &[
    (0x01,"Esc"),(0x0e,"Backspace"),(0x0f,"Tab"),(0x1c,"Enter"),
    (0x1d,"Ctrl"),(0x2a,"Shift"),(0x36,"Right Shift"),(0x37,"Num *"),
    (0x38,"Alt"),(0x39,"Space"),(0x3a,"Caps Lock"),
    (0x3b,"F1"),(0x3c,"F2"),(0x3d,"F3"),(0x3e,"F4"),(0x3f,"F5"),
    (0x40,"F6"),(0x41,"F7"),(0x42,"F8"),(0x43,"F9"),(0x44,"F10"),
    (0x45,"Pause"),(0x46,"Scroll Lock"),
    (0x47,"Num 7"),(0x48,"Num 8"),(0x49,"Num 9"),(0x4a,"Num -"),
    (0x4b,"Num 4"),(0x4c,"Num 5"),(0x4d,"Num 6"),(0x4e,"Num +"),
    (0x4f,"Num 1"),(0x50,"Num 2"),(0x51,"Num 3"),
    (0x52,"Num 0"),(0x53,"Num ."),(0x57,"F11"),(0x58,"F12"),
];

static BUILD_KEY_NAMES_EXT: &[(u8, &str)] = &[
    (0x1c,"Num Enter"),(0x1d,"Right Ctrl"),(0x35,"Num /"),
    (0x37,"Prnt Scrn"),(0x38,"Right Alt"),(0x45,"Num Lock"),(0x46,"Break"),
    (0x47,"Home"),(0x48,"Up"),(0x49,"Page Up"),
    (0x4b,"Left"),(0x4d,"Right"),(0x4f,"End"),
    (0x50,"Down"),(0x51,"Page Down"),(0x52,"Insert"),(0x53,"Delete"),
];

fn build_emit_key_names(out: &mut String) {
    push_ln(out, "static ALLOC_SECTION_LDATA VSC_LPWSTR aKeyNames[] = {");
    for &(vsc, name) in BUILD_KEY_NAMES {
        push_ln(out, &format!("    {{ 0x{vsc:02x}, L\"{name}\" }},"));
    }
    push_ln(out, "    { 0, NULL }");
    push_ln(out, "};");
    push_ln(out, "");
}

fn build_emit_key_names_ext(out: &mut String) {
    push_ln(out, "static ALLOC_SECTION_LDATA VSC_LPWSTR aKeyNamesExt[] = {");
    for &(vsc, name) in BUILD_KEY_NAMES_EXT {
        push_ln(out, &format!("    {{ 0x{vsc:02x}, L\"{name}\" }},"));
    }
    push_ln(out, "    { 0, NULL }");
    push_ln(out, "};");
    push_ln(out, "");
}

fn build_emit_dead_key_names(out: &mut String, layout: &Layout, has_dead_keys: bool) {
    push_ln(out, "static DEADKEY_LPWSTR aKeyNamesDead[] = {");
    if has_dead_keys {
        let mut dk_codes: Vec<&String> = layout.dead_keys.keys().collect();
        dk_codes.sort();
        for dk_code in &dk_codes {
            push_ln(out, &format!("    L\"{}\",", layout.dead_keys[*dk_code].name));
        }
    }
    push_ln(out, "    NULL");
    push_ln(out, "};");
    push_ln(out, "");
}

fn build_emit_modifiers(out: &mut String, has_altgr: bool) {
    push_ln(out, "static VK_TO_BIT aVkToBits[] = {");
    push_ln(out, "    { VK_SHIFT,   KBDSHIFT },");
    push_ln(out, "    { VK_CONTROL, KBDCTRL  },");
    push_ln(out, "    { VK_MENU,    KBDALT   },");
    push_ln(out, "    { 0,          0        }");
    push_ln(out, "};");
    push_ln(out, "");
    push_ln(out, "static MODIFIERS CharModifiers = {");
    push_ln(out, "    &aVkToBits[0],");
    push_ln(out, "    7,");
    push_ln(out, "    {");
    push_ln(out, "        0,");            // 000 base
    push_ln(out, "        1,");            // 001 shift
    push_ln(out, "        2,");            // 010 ctrl
    push_ln(out, "        SHFT_INVALID,"); // 011 shift+ctrl
    push_ln(out, "        SHFT_INVALID,"); // 100 alt
    push_ln(out, "        SHFT_INVALID,"); // 101 shift+alt
    if has_altgr {
        push_ln(out, "        3,");        // 110 ctrl+alt = altgr
        push_ln(out, "        4");         // 111 shift+ctrl+alt = shift+altgr
    } else {
        push_ln(out, "        SHFT_INVALID,"); // 110 ctrl+alt
        push_ln(out, "        SHFT_INVALID"); // 111 shift+ctrl+alt
    }
    push_ln(out, "    }");
    push_ln(out, "};");
    push_ln(out, "");
}

fn build_emit_kbd_tables(out: &mut String, has_altgr: bool) {
    let locale_flags = if has_altgr {
        "(MAKELONG(KLLF_ALTGR, KBD_VERSION))"
    } else {
        "(MAKELONG(0, KBD_VERSION))"
    };
    let n = if has_altgr { "5" } else { "3" };

    push_ln(out, "static VK_TO_WCHAR_TABLE aVkToWcharTable[] = {");
    push_ln(out, &format!(
        "    {{ (PVK_TO_WCHARS1)aVkToWch{n}, {n}, sizeof(aVkToWch{n}[0]) }},"
    ));
    push_ln(out, "    { NULL, 0, 0 }");
    push_ln(out, "};");
    push_ln(out, "");
    // Use explicit casts to silence KBD_LONG_POINTER type warnings on x64.
    push_ln(out, "static KBDTABLES KbdTables = {");
    push_ln(out, "    &CharModifiers,");
    push_ln(out, "    aVkToWcharTable,");
    push_ln(out, "    aDeadKey,");
    push_ln(out, "    (PVSC_LPWSTR)aKeyNames,");
    push_ln(out, "    (PVSC_LPWSTR)aKeyNamesExt,");
    push_ln(out, "    (WCHAR * KBD_LONG_POINTER * KBD_LONG_POINTER)aKeyNamesDead,");
    push_ln(out, "    (USHORT * KBD_LONG_POINTER)ausVK,");
    push_ln(out, "    sizeof(ausVK) / sizeof(ausVK[0]),");
    push_ln(out, "    (PVSC_VK)aE0VscToVk,");
    push_ln(out, "    (PVSC_VK)aE1VscToVk,");
    push_ln(out, &format!("    {locale_flags},"));
    push_ln(out, "    0,");
    push_ln(out, "    0,");
    push_ln(out, "    NULL,");
    push_ln(out, "    0,");
    push_ln(out, "    0");
    push_ln(out, "};");
    push_ln(out, "");
}

fn build_emit_export(out: &mut String) {
    push_ln(out, "__declspec(dllexport) PKBDTABLES KbdLayerDescriptor(void)");
    push_ln(out, "{");
    push_ln(out, "    return &KbdTables;");
    push_ln(out, "}");
    push_ln(out, "");
}
