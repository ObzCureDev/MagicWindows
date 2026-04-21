#!/usr/bin/env node
/**
 * Post-build helper: zip the release exe + its bundled resources into a
 * portable archive. The output ZIP is self-contained — the user unzips it,
 * runs `magicwindows.exe`, and the app finds `kbd_dlls/` and `layouts/`
 * via resource_dir() (same directory as the exe).
 *
 * Run AFTER `npm run tauri build` (or chain via `npm run build:portable`).
 */
import { readdirSync, mkdirSync, existsSync, createWriteStream, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import archiver from "archiver";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = dirname(here);

// Allow CI matrix builds to point at a target-specific release directory,
// e.g. src-tauri/target/x86_64-pc-windows-msvc/release. Locally (no env var
// set) we fall back to the default host-target release dir.
const targetTriple = process.env.TAURI_TARGET_TRIPLE;
const releaseDir = targetTriple
  ? join(repoRoot, "src-tauri", "target", targetTriple, "release")
  : join(repoRoot, "src-tauri", "target", "release");
const exePath = join(releaseDir, "magicwindows.exe");
const kbdDllsDir = join(repoRoot, "target", "kbd_dlls");
const layoutsDir = join(repoRoot, "layouts");

const bundleDir = join(releaseDir, "bundle", "portable");
mkdirSync(bundleDir, { recursive: true });

const pkg = JSON.parse(readFileSync(join(repoRoot, "package.json"), "utf8"));
const version = pkg.version || "dev";

// When built per-target in CI, include an arch suffix so x64 and arm64
// portables don't overwrite each other when uploaded to the same release.
const archLabels = { x86_64: "x64", aarch64: "arm64" };
const archRaw = targetTriple ? targetTriple.split("-")[0] : null;
const archSuffix = archRaw ? `_${archLabels[archRaw] || archRaw}` : "";
const outZip = join(bundleDir, `MagicWindows_${version}${archSuffix}_portable.zip`);

for (const p of [exePath, kbdDllsDir, layoutsDir]) {
  if (!existsSync(p)) {
    console.error(`[build-portable] Missing required input: ${p}`);
    process.exit(1);
  }
}

console.log(`[build-portable] Creating ${outZip}`);

const output = createWriteStream(outZip);
const archive = archiver("zip", { zlib: { level: 9 } });

output.on("close", () => {
  const mb = (archive.pointer() / 1024 / 1024).toFixed(2);
  console.log(`[build-portable] Done: ${outZip} (${mb} MB)`);
});
archive.on("error", (err) => {
  throw err;
});
archive.pipe(output);

// Main exe at the root of the archive.
archive.file(exePath, { name: "magicwindows.exe" });

// kbd_dlls/ (only .dll files, skip .exp/.lib).
for (const file of readdirSync(kbdDllsDir)) {
  if (file.endsWith(".dll")) {
    archive.file(join(kbdDllsDir, file), { name: `kbd_dlls/${file}` });
  }
}

// layouts/ (all .json files — schema is harmless, keep it).
for (const file of readdirSync(layoutsDir)) {
  if (file.endsWith(".json")) {
    archive.file(join(layoutsDir, file), { name: `layouts/${file}` });
  }
}

const readme = `MagicWindows portable \u2014 version ${version}

Usage:
  1. Keep magicwindows.exe next to the kbd_dlls/ and layouts/ folders.
  2. Double-click magicwindows.exe to run.
  3. The app will ask for administrator rights (UAC) to install the
     selected keyboard layout into the Windows system directory.

To uninstall a layout afterwards, re-launch the exe and open the
Settings page (top bar, gear icon).

Requires Microsoft Edge WebView2 (present by default on Windows 10/11).
`;
archive.append(readme, { name: "README.txt" });

await archive.finalize();
