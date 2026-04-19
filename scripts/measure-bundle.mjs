#!/usr/bin/env node
/**
 * Post-build helper: record the size of every build artefact into
 * `docs/metrics.json` as an append-only history entry.
 *
 * Run AFTER `npm run tauri build` (and/or `npm run build:portable`):
 *   npm run measure
 *
 * The resulting file is committed — each release produces one new JSON entry
 * and `git log -p docs/metrics.json` gives a human-readable size timeline.
 */
import { readdirSync, existsSync, statSync, readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { execSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = dirname(here);

const pkg = JSON.parse(readFileSync(join(repoRoot, "package.json"), "utf8"));
const version = pkg.version || "dev";

const releaseDir = join(repoRoot, "src-tauri", "target", "release");
const bundleDir = join(releaseDir, "bundle");

const artefacts = {};

function recordIfPresent(key, path) {
  if (existsSync(path) && statSync(path).isFile()) {
    artefacts[key] = statSync(path).size;
  }
}

// Raw executable.
recordIfPresent("exe", join(releaseDir, "magicwindows.exe"));

// MSI and NSIS installers (filenames include the version, so glob the dirs).
for (const [key, dir] of [["msi", join(bundleDir, "msi")], ["nsis", join(bundleDir, "nsis")]]) {
  if (!existsSync(dir)) continue;
  const match = readdirSync(dir).find((f) => f.endsWith(key === "msi" ? ".msi" : ".exe"));
  if (match) artefacts[key] = statSync(join(dir, match)).size;
}

// Portable ZIP (produced by build-portable.mjs).
const portableDir = join(bundleDir, "portable");
if (existsSync(portableDir)) {
  const zip = readdirSync(portableDir).find((f) => f.endsWith(".zip"));
  if (zip) artefacts.portable_zip = statSync(join(portableDir, zip)).size;
}

// Compiled keyboard DLLs (one per layout).
const kbdDllsDir = join(repoRoot, "target", "kbd_dlls");
if (existsSync(kbdDllsDir)) {
  const dlls = readdirSync(kbdDllsDir).filter((f) => f.endsWith(".dll"));
  artefacts.kbd_dlls_total = dlls.reduce(
    (sum, f) => sum + statSync(join(kbdDllsDir, f)).size,
    0,
  );
  artefacts.kbd_dll_count = dlls.length;
}

if (Object.keys(artefacts).length === 0) {
  console.error("[measure-bundle] No build artefacts found. Did you run `npm run tauri build`?");
  process.exit(1);
}

let commit = "unknown";
try {
  commit = execSync("git rev-parse --short HEAD", { cwd: repoRoot }).toString().trim();
} catch {
  // Not a git checkout or git missing — leave as "unknown".
}

const entry = {
  version,
  commit,
  timestamp: new Date().toISOString(),
  bundles: artefacts,
};

const metricsPath = join(repoRoot, "docs", "metrics.json");
mkdirSync(dirname(metricsPath), { recursive: true });

let history = [];
if (existsSync(metricsPath)) {
  try {
    history = JSON.parse(readFileSync(metricsPath, "utf8"));
    if (!Array.isArray(history)) history = [];
  } catch {
    history = [];
  }
}
history.push(entry);

writeFileSync(metricsPath, JSON.stringify(history, null, 2) + "\n", "utf8");

const fmt = (n) => (n / 1024 / 1024).toFixed(2) + " MB";
console.log(`[measure-bundle] ${version} @ ${commit} recorded:`);
for (const [k, v] of Object.entries(artefacts)) {
  console.log(`  ${k.padEnd(16)} ${typeof v === "number" && k !== "kbd_dll_count" ? fmt(v) : v}`);
}
console.log(`[measure-bundle] Appended to ${metricsPath}`);
