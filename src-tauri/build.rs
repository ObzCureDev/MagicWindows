//! MagicWindows build script.
//!
//! On Windows this script:
//!   1. Reads every `layouts/*.json` file (skipping `schema.json`).
//!   2. Generates a C source file for each layout using `klc-codegen`
//!      (shared with the main crate's tests — single source of truth).
//!   3. Compiles each C file into a keyboard layout DLL using `cl.exe` and
//!      `link.exe` from the MSVC toolchain that Rust itself requires.
//!   4. Copies the compiled DLLs to `target/kbd_dlls/` where Tauri can
//!      pick them up as bundled resources.
//!
//! On non-Windows targets the DLL compilation step is skipped (the DLLs
//! can only be installed on Windows anyway).

fn main() {
    println!("cargo:rerun-if-changed=../layouts");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=klc-codegen/src/lib.rs");

    // Compile keyboard DLLs BEFORE tauri_build::build() so that the
    // kbd_dlls/ directory is populated when tauri-build validates the
    // resource glob in tauri.conf.json.
    #[cfg(target_os = "windows")]
    if let Err(e) = windows::compile_keyboard_dlls() {
        // Emit a warning rather than a hard error so that `cargo check` and
        // `cargo test` still work even without a complete MSVC environment.
        println!("cargo:warning=Keyboard DLL compilation skipped: {e}");
    }

    tauri_build::build();
}

#[cfg(target_os = "windows")]
mod windows {
    use std::{env, fs, path::{Path, PathBuf}, process::Command};

    /// Target architectures we build keyboard layout DLLs for.
    /// Windows on ARM cannot load x64 keyboard DLLs (Win32K is not emulated),
    /// so a native ARM64 build is required to support Surface Pro X / Snapdragon
    /// machines. The desktop app itself stays x64 for now; ARM64 DLLs are
    /// shipped in the web download ZIPs.
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub(crate) enum Arch {
        X64,
        Arm64,
    }

    impl Arch {
        fn subdir(self) -> &'static str {
            match self { Self::X64 => "x64", Self::Arm64 => "arm64" }
        }
        fn host_target(self) -> (&'static str, &'static str) {
            // (host, target) under VC/Tools/MSVC/<ver>/bin/Host<host>/<target>/
            match self {
                Self::X64   => ("x64", "x64"),
                Self::Arm64 => ("x64", "arm64"),
            }
        }
        fn lib_dir(self) -> &'static str {
            match self { Self::X64 => "x64", Self::Arm64 => "arm64" }
        }
        fn arch_define(self) -> &'static str {
            match self { Self::X64 => "/D_AMD64_=1", Self::Arm64 => "/D_ARM64_=1" }
        }
        fn vswhere_required(self) -> &'static str {
            match self {
                Self::X64   => "Microsoft.VisualCpp.Tools.HostX64.TargetX64",
                Self::Arm64 => "Microsoft.VisualCpp.Tools.HostX64.TargetARM64",
            }
        }
    }

    pub fn compile_keyboard_dlls() -> Result<(), String> {
        let manifest_dir = PathBuf::from(
            env::var("CARGO_MANIFEST_DIR").map_err(|_| "CARGO_MANIFEST_DIR not set")?,
        );
        let layouts_dir = manifest_dir
            .parent()
            .ok_or("cannot get parent of manifest dir")?
            .join("layouts");
        let out_dir = PathBuf::from(env::var("OUT_DIR").map_err(|_| "OUT_DIR not set")?);

        // Intermediate .c/.obj files stay inside OUT_DIR (never committed).
        let c_build_dir = out_dir.join("kbd_c_build");
        fs::create_dir_all(&c_build_dir).map_err(|e| format!("create kbd_c_build dir: {e}"))?;

        // Final DLL destination: target/kbd_dlls/<arch>/ (outside src-tauri so
        // the Tauri dev watcher doesn't see them changing and loop constantly;
        // tauri.conf.json bundles only the x64 DLLs from
        // ../target/kbd_dlls/x64/*.dll for the desktop installer).
        let dll_base_dir = manifest_dir
            .parent()
            .ok_or("cannot get parent of manifest dir")?
            .join("target")
            .join("kbd_dlls");

        // Load all layouts once, so we can iterate per-architecture.
        let mut layouts: Vec<(PathBuf, klc_codegen::Layout)> = Vec::new();
        let entries = fs::read_dir(&layouts_dir).map_err(|e| format!("read layouts dir: {e}"))?;
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
            let layout: klc_codegen::Layout = serde_json::from_str(&json)
                .map_err(|e| format!("parse {}: {e}", path.display()))?;
            layouts.push((path, layout));
        }

        for arch in [Arch::X64, Arch::Arm64] {
            let msvc = match MsvcPaths::find(arch) {
                Ok(m) => m,
                Err(e) => {
                    if arch == Arch::Arm64 {
                        // ARM64 toolchain is optional. Warn and skip — the build
                        // succeeds without ARM64 DLLs and the desktop app keeps working.
                        println!(
                            "cargo:warning=ARM64 toolchain not found; skipping ARM64 DLLs ({e})"
                        );
                        continue;
                    }
                    return Err(e);
                }
            };

            let dll_dest_dir = dll_base_dir.join(arch.subdir());
            fs::create_dir_all(&dll_dest_dir)
                .map_err(|e| format!("create {} dir: {e}", dll_dest_dir.display()))?;

            for (layout_path, layout) in &layouts {
                let dll_path = dll_dest_dir.join(format!("{}.dll", layout.dll_name));

                let c_src = klc_codegen::generate_kbd_c(layout);
                let c_path = c_build_dir.join(format!("{}-{}.c", layout.dll_name, arch.subdir()));
                fs::write(&c_path, &c_src)
                    .map_err(|e| format!("write {}: {e}", c_path.display()))?;

                compile_dll(&msvc, arch, &c_path, &dll_path, &c_build_dir).map_err(|e| {
                    println!(
                        "cargo:warning=Failed to compile {} DLL for {} ({}): {e}",
                        arch.subdir(),
                        layout.dll_name,
                        layout_path.display(),
                    );
                    e
                })?;

                println!(
                    "cargo:warning=Compiled keyboard DLL: {}",
                    dll_path.display()
                );
            }
        }

        Ok(())
    }

    fn compile_dll(
        msvc: &MsvcPaths,
        arch: Arch,
        c_path: &Path,
        dll_path: &Path,
        obj_dir: &Path,
    ) -> Result<(), String> {
        let obj_path = obj_dir.join(
            c_path
                .file_stem()
                .map(|s| format!("{}.obj", s.to_string_lossy()))
                .unwrap_or_else(|| "kbd.obj".into()),
        );

        // ── Step 1: Compile C → OBJ ─────────────────────────────────────────
        // winnt.h requires one of _X86_, _AMD64_, _ARM_, or _ARM64_ to be
        // defined to pick the architecture-specific type definitions.
        let mut cl = Command::new(&msvc.cl);
        cl.env("INCLUDE", &msvc.include_dirs)
            .env("LIB", &msvc.lib_dirs)
            .env("PATH", &msvc.path);
        cl.arg("/nologo")
            .arg("/W3")
            .arg("/Zl") // Omit default library name from .obj
            .arg("/c") // Compile only (no link)
            .arg("/GS-") // Disable buffer security checks (no CRT)
            .arg(arch.arch_define())
            .arg("/DWIN32")
            .arg("/D_WINDOWS")
            .arg(format!("/Fo{}", obj_path.display()))
            .arg(c_path);

        let output = cl.output().map_err(|e| format!("spawn cl.exe: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "cl.exe failed compiling {}:\nstdout: {}\nstderr: {}",
                c_path.display(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            ));
        }

        // ── Step 2: Link OBJ → DLL ──────────────────────────────────────────
        // Write a minimal .def so the export is clean without needing
        // __declspec(dllexport) on older toolchain versions.
        let dll_stem = dll_path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "kbd".into());
        let def_path = obj_dir.join(format!("{dll_stem}.def"));
        fs::write(
            &def_path,
            format!("LIBRARY {dll_stem}\nEXPORTS\n    KbdLayerDescriptor\n"),
        )
        .map_err(|e| format!("write def file: {e}"))?;

        let mut link = Command::new(&msvc.link);
        link.env("LIB", &msvc.lib_dirs).env("PATH", &msvc.path);
        link.arg("/nologo")
            .arg("/DLL")
            .arg("/NOENTRY")
            .arg("/NODEFAULTLIB")
            .arg(format!("/DEF:{}", def_path.display()))
            .arg(format!("/OUT:{}", dll_path.display()))
            .arg(&obj_path)
            .arg("user32.lib");

        let link_output = link.output().map_err(|e| format!("spawn link.exe: {e}"))?;
        if !link_output.status.success() {
            return Err(format!(
                "link.exe failed linking {}:\nstdout: {}\nstderr: {}",
                dll_path.display(),
                String::from_utf8_lossy(&link_output.stdout),
                String::from_utf8_lossy(&link_output.stderr),
            ));
        }

        Ok(())
    }

    /// Paths to the MSVC compiler and linker, plus the include/lib search paths.
    pub(crate) struct MsvcPaths {
        pub cl: PathBuf,
        pub link: PathBuf,
        /// Semicolon-separated include directories (INCLUDE env var for cl.exe).
        pub include_dirs: String,
        /// Semicolon-separated library directories (LIB env var for link.exe).
        pub lib_dirs: String,
        /// PATH that contains the MSVC bin dir (so cl.exe can find c1.dll etc.).
        pub path: String,
    }

    impl MsvcPaths {
        /// Discover the MSVC toolchain for `arch`.
        ///
        /// Probes `vswhere.exe` first (latest VS install with the right
        /// component requirement), then falls back to the common VS 2022 and
        /// VS 2019 install locations. Returns Err when the architecture's
        /// toolchain is not installed (e.g. ARM64 cross-compiler missing).
        pub fn find(arch: Arch) -> Result<Self, String> {
            if let Ok(paths) = Self::find_via_vswhere(arch) {
                return Ok(paths);
            }
            Self::find_via_known_paths(arch)
        }

        fn find_via_vswhere(arch: Arch) -> Result<Self, String> {
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
                    arch.vswhere_required(),
                    "-property",
                    "installationPath",
                ])
                .output()
                .map_err(|e| format!("vswhere: {e}"))?;

            if !out.status.success() {
                return Err("vswhere returned non-zero".into());
            }

            let install_root = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if install_root.is_empty() {
                return Err(format!(
                    "vswhere found no install with {}",
                    arch.vswhere_required()
                ));
            }

            Self::from_vs_install_root(&install_root, arch)
        }

        fn find_via_known_paths(arch: Arch) -> Result<Self, String> {
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
                    if let Ok(paths) = Self::from_vs_install_root(root, arch) {
                        return Ok(paths);
                    }
                }
            }

            Err(format!("No MSVC {} toolchain found", arch.subdir()))
        }

        fn from_vs_install_root(root: &str, arch: Arch) -> Result<Self, String> {
            let vc_tools = PathBuf::from(root).join("VC").join("Tools").join("MSVC");
            if !vc_tools.exists() {
                return Err(format!("{} does not contain VC/Tools/MSVC", root));
            }

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

            // bin/Host<host>/<target>/cl.exe — host is always x64 on our build
            // machines; target depends on the arch we want DLLs for.
            let (host, target) = arch.host_target();
            let bin_dir = msvc_ver
                .join("bin")
                .join(format!("Host{host}"))
                .join(target);
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

            let msvc_include = msvc_ver.join("include");
            let (sdk_include, sdk_lib) = find_windows_sdk()?;

            let include_dirs = format!(
                "{};{};{};{}",
                msvc_include.display(),
                sdk_include.join("um").display(),
                sdk_include.join("shared").display(),
                sdk_include.join("ucrt").display(),
            );

            let msvc_lib = msvc_ver.join("lib").join(arch.lib_dir());
            let lib_dirs = format!(
                "{};{};{}",
                msvc_lib.display(),
                sdk_lib.join("um").join(arch.lib_dir()).display(),
                sdk_lib.join("ucrt").join(arch.lib_dir()).display(),
            );

            // PATH must include the host x64 bin dir so cl.exe can find its
            // DLLs (c1.dll etc.) — these live next to the host x64 cl, not
            // the cross-compile target dir.
            let host_x64_bin = msvc_ver.join("bin").join(format!("Host{host}")).join(host);
            let system_path = env::var("PATH").unwrap_or_default();
            let path = format!(
                "{};{};{system_path}",
                bin_dir.display(),
                host_x64_bin.display()
            );

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
    fn find_windows_sdk() -> Result<(PathBuf, PathBuf), String> {
        let sdk_root = PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10");
        if !sdk_root.exists() {
            return Err("Windows Kits 10 not found at default location".into());
        }

        let include_root = sdk_root.join("Include");
        let lib_root = sdk_root.join("Lib");

        let latest_include = latest_versioned_dir(&include_root)?;
        let latest_lib = latest_versioned_dir(&lib_root)?;

        Ok((latest_include, latest_lib))
    }

    /// Return the highest-version sub-directory of `dir` (e.g. `10.0.22621.0`).
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
}
