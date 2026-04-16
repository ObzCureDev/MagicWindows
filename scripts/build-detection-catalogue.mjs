#!/usr/bin/env node
// Reads layouts/apple-*.json, extracts every (codepoint, scancode) pair from base/shift/altgr/altgrShift layers,
// translates scancode -> DOM event.code, filters out dead keys (entries ending with '@'),
// and emits a per-character map of layout -> position. Only includes characters that distinguish
// at least 2 layouts (or are present in 1 layout — useful as ABSENT-bucket discriminators).

import { readFileSync, writeFileSync, readdirSync } from "fs";
import { join, resolve, dirname } from "path";
import { fileURLToPath } from "url";
import { SCANCODE_TO_CODE } from "./scancode-map.mjs";

const __dirname = dirname(fileURLToPath(import.meta.url));
const LAYOUTS_DIR = resolve(__dirname, "..", "layouts");
const OUT_PATH   = resolve(__dirname, "..", "src", "lib", "detection-catalogue.generated.json");

function loadLayouts() {
  return readdirSync(LAYOUTS_DIR)
    .filter(f => f.startsWith("apple-") && f.endsWith(".json"))
    .map(f => JSON.parse(readFileSync(join(LAYOUTS_DIR, f), "utf8")));
}

function isDeadKey(hex) {
  return typeof hex === "string" && hex.endsWith("@");
}

function normalizeHex(hex) {
  return isDeadKey(hex) ? hex.slice(0, -1) : hex;
}

// Returns Map<codepointHex, eventCode> — the FIRST scancode that prints this codepoint on this layout
function charsPrintedOn(layout) {
  const result = new Map();
  for (const [scancode, mapping] of Object.entries(layout.keys ?? {})) {
    const eventCode = SCANCODE_TO_CODE[scancode];
    if (!eventCode) continue;
    for (const layer of ["base", "shift", "altgr", "altgrShift"]) {
      const raw = mapping[layer];
      if (!raw || raw === "-1") continue;
      if (isDeadKey(raw)) continue;
      const cp = normalizeHex(raw).toLowerCase();
      if (cp === "0000") continue;
      if (!result.has(cp)) result.set(cp, eventCode);
    }
  }
  return result;
}

function build() {
  const layouts = loadLayouts();
  // codepoint -> { layoutId -> eventCode }
  const byChar = new Map();
  for (const layout of layouts) {
    const charMap = charsPrintedOn(layout);
    for (const [cp, eventCode] of charMap) {
      if (!byChar.has(cp)) byChar.set(cp, {});
      byChar.get(cp)[layout.id] = eventCode;
    }
  }

  // Keep only characters useful for discrimination:
  //   - present on at least one layout AND
  //   - either: distinguishes 2+ layouts by position, OR splits present-vs-absent across the full set
  const totalLayouts = layouts.length;
  const useful = [];
  for (const [cp, positions] of byChar) {
    const presentCount = Object.keys(positions).length;
    const distinctPositions = new Set(Object.values(positions)).size;
    const splitsPositions = distinctPositions >= 2;
    const splitsPresence  = presentCount > 0 && presentCount < totalLayouts;
    if (splitsPositions || splitsPresence) {
      useful.push({
        char: String.fromCodePoint(parseInt(cp, 16)),
        codepoint: cp,
        positions,
      });
    }
  }

  // Sort: best discriminators first (characters that split into the most balanced buckets across all layouts)
  useful.sort((a, b) => {
    const score = (entry) => {
      const buckets = new Map();
      buckets.set("ABSENT", totalLayouts - Object.keys(entry.positions).length);
      for (const code of Object.values(entry.positions)) {
        buckets.set(code, (buckets.get(code) ?? 0) + 1);
      }
      return Math.max(...buckets.values()); // smaller worst-bucket = better
    };
    return score(a) - score(b);
  });

  const out = { generatedAt: new Date().toISOString(), characters: useful };
  writeFileSync(OUT_PATH, JSON.stringify(out, null, 2) + "\n", "utf8");
  console.log(`Detection catalogue: ${useful.length} characters across ${layouts.length} layouts -> ${OUT_PATH}`);
}

build();
