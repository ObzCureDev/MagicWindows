#!/usr/bin/env node
// Builds the per-layout download ZIPs, uploads them to R2 via wrangler,
// and writes web/public/manifest.json.
//
// Usage:
//   node scripts/build-web-release.mjs            # full release
//   node scripts/build-web-release.mjs --dry-run  # build ZIPs locally, skip R2 upload + manifest write

import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync, createWriteStream } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import archiver from "archiver";

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(__dirname, "..");
const LAYOUTS_DIR = join(REPO_ROOT, "layouts");
const DLL_OUT_DIR = join(REPO_ROOT, "target", "kbd_dlls");
const PAYLOAD_DIR = join(REPO_ROOT, "scripts", "web");
const STAGING_DIR = join(REPO_ROOT, "target", "web-release");
const MANIFEST_PATH = join(REPO_ROOT, "web", "public", "manifest.json");
const SRC_TAURI = join(REPO_ROOT, "src-tauri");

const R2_BUCKET = "magicwindows-downloads";
const R2_PUBLIC_BASE = `https://${R2_BUCKET}.r2.dev`;
const EPOCH = new Date(0);

// ── Pure helpers (unit-tested in build-web-release.test.mjs) ────────────────

export function sha256(buf) {
  return createHash("sha256").update(buf).digest("hex");
}

export function manifestEntry({ url, size, sha256: hash }) {
  return { url, size, sha256: hash };
}

export function layoutMetadata(layout) {
  if (!layout?.dllName || typeof layout.dllName !== "string") {
    throw new Error(`layout missing dllName: ${JSON.stringify(layout)}`);
  }
  if (!layout?.name?.en || typeof layout.name.en !== "string") {
    throw new Error(`layout missing name.en`);
  }
  if (!/^[0-9a-fA-F]{8}$/.test(layout.localeId ?? "")) {
    throw new Error(`layout localeId must be 8 hex chars, got '${layout.localeId}'`);
  }
  return {
    dllName: layout.dllName,
    displayName: layout.name.en,
    localeId: layout.localeId,
  };
}

// ── Orchestration ───────────────────────────────────────────────────────────

function loadLayouts() {
  return readdirSync(LAYOUTS_DIR)
    .filter((f) => f.startsWith("apple-") && f.endsWith(".json"))
    .map((f) => ({
      id: f.replace(/\.json$/, ""),
      data: JSON.parse(readFileSync(join(LAYOUTS_DIR, f), "utf8")),
    }));
}

function ensureDllsBuilt(layouts) {
  console.log("→ Triggering cargo build (compiles layouts/*.json into kbd_dlls/*.dll via build.rs)...");
  execFileSync("cargo", ["build", "--manifest-path", join(SRC_TAURI, "Cargo.toml")], {
    stdio: "inherit",
  });
  if (!existsSync(DLL_OUT_DIR)) {
    throw new Error(`DLL output dir missing after cargo build: ${DLL_OUT_DIR}. Are you on Windows with MSVC?`);
  }
  const missing = [];
  for (const layout of layouts) {
    const dllName = layout.data.dllName;
    if (!dllName) continue; // layoutMetadata will throw later with a clearer error
    const dllPath = join(DLL_OUT_DIR, `${dllName}.dll`);
    if (!existsSync(dllPath)) {
      missing.push(`${layout.id} → ${dllName}.dll`);
    }
  }
  if (missing.length > 0) {
    throw new Error(
      `cargo build did not produce DLLs for ${missing.length} layout(s):\n  ` +
      missing.join("\n  ") +
      `\nCheck src-tauri/build.rs warnings or run from src-tauri/ to see compiler errors.`
    );
  }
}

async function packageOne(layoutEntry, version) {
  const meta = layoutMetadata(layoutEntry.data);
  const dllPath = join(DLL_OUT_DIR, `${meta.dllName}.dll`);
  if (!existsSync(dllPath)) {
    throw new Error(`DLL not produced: ${dllPath}`);
  }

  const zipName = `magicwindows-${layoutEntry.id}-${version}.zip`;
  const zipPath = join(STAGING_DIR, zipName);

  const layoutJson = JSON.stringify(meta, null, 2);

  await new Promise((resolveZip, rejectZip) => {
    const out = createWriteStream(zipPath);
    // statConcurrency: 1 keeps entry order stable across runs (default 4 races
    // file stats, producing different central-directory ordering each run).
    const ar = archiver("zip", { zlib: { level: 9 }, statConcurrency: 1 });
    out.on("close", resolveZip);
    ar.on("error", rejectZip);
    ar.pipe(out);
    ar.file(dllPath, { name: `${meta.dllName}.dll`, date: EPOCH });
    ar.file(join(PAYLOAD_DIR, "Install-Layout.cmd"),   { name: "Install-Layout.cmd",   date: EPOCH });
    ar.file(join(PAYLOAD_DIR, "Install-Layout.ps1"),   { name: "Install-Layout.ps1",   date: EPOCH });
    ar.file(join(PAYLOAD_DIR, "Uninstall-Layout.cmd"), { name: "Uninstall-Layout.cmd", date: EPOCH });
    ar.file(join(PAYLOAD_DIR, "Uninstall-Layout.ps1"), { name: "Uninstall-Layout.ps1", date: EPOCH });
    ar.file(join(PAYLOAD_DIR, "README.txt"),           { name: "README.txt",           date: EPOCH });
    ar.append(layoutJson,                              { name: "layout.json",          date: EPOCH });
    ar.finalize();
  });

  const buf = readFileSync(zipPath);
  return {
    layoutId: layoutEntry.id,
    zipPath,
    zipName,
    size: buf.length,
    sha256: sha256(buf),
  };
}

function uploadToR2(zipPath, zipName) {
  console.log(`  ↑ uploading ${zipName} to R2 ...`);
  execFileSync("npx", ["wrangler", "r2", "object", "put",
    `${R2_BUCKET}/${zipName}`,
    "--file", zipPath,
  ], { stdio: "inherit" });
}

async function main() {
  const dryRun = process.argv.includes("--dry-run");
  const pkg = JSON.parse(readFileSync(join(REPO_ROOT, "package.json"), "utf8"));
  const version = pkg.version;
  console.log(`Building MagicWindows Web release for version ${version}${dryRun ? " (DRY RUN)" : ""}\n`);

  if (existsSync(STAGING_DIR)) rmSync(STAGING_DIR, { recursive: true, force: true });
  mkdirSync(STAGING_DIR, { recursive: true });

  const layouts = loadLayouts();
  ensureDllsBuilt(layouts);
  console.log(`Found ${layouts.length} layouts\n`);

  const downloads = {};
  for (const layout of layouts) {
    console.log(`→ Packaging ${layout.id}...`);
    const result = await packageOne(layout, version);
    if (!dryRun) {
      uploadToR2(result.zipPath, result.zipName);
    }
    const url = `${R2_PUBLIC_BASE}/${result.zipName}`;
    downloads[layout.id] = manifestEntry({ url, size: result.size, sha256: result.sha256 });
    console.log(`  ${result.size} bytes, sha256 ${result.sha256.slice(0, 16)}…`);
  }

  const manifest = {
    version,
    generatedAt: new Date().toISOString(),
    downloads,
  };

  if (dryRun) {
    console.log("\nDRY RUN — manifest preview:");
    console.log(JSON.stringify(manifest, null, 2));
  } else {
    writeFileSync(MANIFEST_PATH, JSON.stringify(manifest, null, 2) + "\n", "utf8");
    console.log(`\nWrote ${MANIFEST_PATH}`);
    console.log("Commit and push to deploy.");
  }
}

// Only run main() when invoked directly, not when imported by tests.
const invokedDirectly = fileURLToPath(import.meta.url) === resolve(process.argv[1] ?? "");
if (invokedDirectly) {
  main().catch((err) => {
    console.error("\nFAILED:", err.message);
    process.exit(1);
  });
}
