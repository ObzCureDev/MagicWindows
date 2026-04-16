# Character-Based Keyboard Detection — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace position-based keyboard detection with a character-based flow that asks the user to press the physical key where a printed symbol appears, then matches `event.code` against a build-time-generated per-layout catalogue.

**Architecture:** A pure-frontend detection algorithm (`src/lib/detection.ts`) consumes a JSON catalogue auto-generated at build time from `layouts/*.json`. The Detect page presents one symbol at a time (`@`, then `ñ` if needed), captures `event.code`, and narrows candidates using minimax scoring with an explicit ABSENT bucket. A toast handles wrong-key presses; an "I don't have this key" button handles regional characters; a "Pick manually" button is always visible. Failure routes to `Select.svelte` with an inline banner.

**Tech Stack:** Svelte 5 ($state), TypeScript, Vite, Vitest (added in Task 1), Node script for catalogue generation. No Rust changes.

**Spec:** [docs/superpowers/specs/2026-04-16-character-based-detection-design.md](../specs/2026-04-16-character-based-detection-design.md)

---

## File Structure

| File | Status | Responsibility |
|------|--------|----------------|
| `package.json` | modify | Add Vitest dep; add `predev`/`prebuild`/`test` scripts |
| `vitest.config.ts` | create | Vitest config (Node env for pure-logic tests) |
| `.gitignore` | modify | Ignore generated catalogue |
| `scripts/scancode-map.mjs` | create | Static scancode → DOM `event.code` lookup (shared by generator) |
| `scripts/build-detection-catalogue.mjs` | create | Walks `layouts/*.json` → emits `detection-catalogue.generated.json` |
| `src/lib/detection-catalogue.generated.json` | generated | Char → per-layout-position table (gitignored) |
| `src/lib/types.ts` | modify | Add `DetectionChar`, `DetectionCatalogue`, `DetectionResponse`, `DetectionPhase` types |
| `src/lib/detection.ts` | create | Pure algorithm: `pickBestQuestion`, `applyResponse`, `printedChars` |
| `src/lib/detection.test.ts` | create | Vitest unit tests for the algorithm |
| `src/lib/i18n.ts` | modify | 7 new translation keys (EN + FR) |
| `src/lib/stores.svelte.ts` | modify | Add `detectionFailedMessage?: string` field |
| `src/pages/Detect.svelte` | rewrite | Character-based flow (prompt, oops toast, extended help, manual button) |
| `src/pages/Select.svelte` | modify | Render inline banner when `appState.detectionFailedMessage` is set |
| `src/app.css` | modify | Add `.mk-body--neutral` modifier (desaturated mockup during detection) |

---

## Task 1: Add Vitest for unit testing the detection algorithm

**Files:**
- Modify: `package.json`
- Create: `vitest.config.ts`

- [ ] **Step 1: Install Vitest**

Run: `npm install --save-dev vitest@^2.1.0`
Expected: `vitest` appears under `devDependencies`. No peer-dep warnings critical.

- [ ] **Step 2: Add the test script to `package.json`**

Modify the `"scripts"` block in `package.json` to add a `test` entry:

```json
"scripts": {
  "dev": "vite",
  "build": "vite build",
  "preview": "vite preview",
  "check": "svelte-check --tsconfig ./tsconfig.app.json && tsc -p tsconfig.node.json",
  "test": "vitest run",
  "test:watch": "vitest",
  "tauri": "tauri"
}
```

- [ ] **Step 3: Create `vitest.config.ts`**

```ts
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
});
```

- [ ] **Step 4: Smoke-test the runner with a placeholder test**

Create `src/lib/_vitest_smoke.test.ts` (temporary):

```ts
import { describe, it, expect } from "vitest";
describe("vitest smoke", () => {
  it("runs", () => expect(1 + 1).toBe(2));
});
```

Run: `npm test`
Expected: 1 passing test.

Then delete `src/lib/_vitest_smoke.test.ts`.

- [ ] **Step 5: Commit**

```bash
git add package.json package-lock.json vitest.config.ts
git commit -m "chore: add vitest for detection algorithm tests"
```

---

## Task 2: Build the scancode → event.code lookup table

**Files:**
- Create: `scripts/scancode-map.mjs`

- [ ] **Step 1: Create `scripts/scancode-map.mjs`**

```js
// Maps Windows PS/2 scancodes (as used in layouts/*.json) to DOM KeyboardEvent.code values.
// Coverage: every scancode that appears in any layouts/*.json key mapping.
// Source: USB HID Usage Tables + W3C UI Events KeyboardEvent code spec.

export const SCANCODE_TO_CODE = {
  // Top row
  "29": "Backquote",
  "02": "Digit1", "03": "Digit2", "04": "Digit3", "05": "Digit4",
  "06": "Digit5", "07": "Digit6", "08": "Digit7", "09": "Digit8",
  "0a": "Digit9", "0b": "Digit0",
  "0c": "Minus", "0d": "Equal",

  // Letter row 1 (QWERTY)
  "10": "KeyQ", "11": "KeyW", "12": "KeyE", "13": "KeyR", "14": "KeyT",
  "15": "KeyY", "16": "KeyU", "17": "KeyI", "18": "KeyO", "19": "KeyP",
  "1a": "BracketLeft", "1b": "BracketRight",

  // Letter row 2
  "1e": "KeyA", "1f": "KeyS", "20": "KeyD", "21": "KeyF", "22": "KeyG",
  "23": "KeyH", "24": "KeyJ", "25": "KeyK", "26": "KeyL",
  "27": "Semicolon", "28": "Quote", "2b": "Backslash",

  // Letter row 3
  "56": "IntlBackslash",
  "2c": "KeyZ", "2d": "KeyX", "2e": "KeyC", "2f": "KeyV", "30": "KeyB",
  "31": "KeyN", "32": "KeyM",
  "33": "Comma", "34": "Period", "35": "Slash",

  // Space
  "39": "Space",
};
```

- [ ] **Step 2: Verify every scancode used in `layouts/*.json` has a mapping**

Run this one-liner to print any uncovered scancodes:

```bash
node -e "
import('./scripts/scancode-map.mjs').then(({SCANCODE_TO_CODE}) => {
  import('fs').then(fs => {
    import('path').then(path => {
      const dir = 'layouts';
      const files = fs.readdirSync(dir).filter(f => f.startsWith('apple-') && f.endsWith('.json'));
      const used = new Set();
      for (const f of files) {
        const json = JSON.parse(fs.readFileSync(path.join(dir, f), 'utf8'));
        for (const sc of Object.keys(json.keys ?? {})) used.add(sc);
      }
      const missing = [...used].filter(sc => !SCANCODE_TO_CODE[sc]);
      console.log(missing.length === 0 ? 'OK: all scancodes mapped' : 'MISSING: ' + missing.join(', '));
    });
  });
});
"
```

Expected: `OK: all scancodes mapped`. If any are missing, add them to the map (consult `https://www.w3.org/TR/uievents-code/`) and rerun.

- [ ] **Step 3: Commit**

```bash
git add scripts/scancode-map.mjs
git commit -m "feat: add scancode to DOM event.code lookup table"
```

---

## Task 3: Build the catalogue generator script

**Files:**
- Create: `scripts/build-detection-catalogue.mjs`

- [ ] **Step 1: Write the generator**

```js
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
```

- [ ] **Step 2: Run the generator manually**

Run: `node scripts/build-detection-catalogue.mjs`

Expected stdout (similar):
```
Detection catalogue: 30+ characters across 6 layouts -> .../src/lib/detection-catalogue.generated.json
```

- [ ] **Step 3: Inspect the output for the two key characters**

Run: `node -e "const c = require('./src/lib/detection-catalogue.generated.json'); console.log(JSON.stringify(c.characters.find(x=>x.char==='@'), null, 2)); console.log(JSON.stringify(c.characters.find(x=>x.char==='ñ'), null, 2));"`

Expected:
```json
{
  "char": "@",
  "codepoint": "0040",
  "positions": {
    "apple-us-qwerty": "Digit2",
    "apple-uk-qwerty": "Quote",
    "apple-fr-azerty": "Backquote",
    "apple-de-qwertz": "KeyL",
    "apple-es-qwerty": "Digit2",
    "apple-it-qwerty": "Semicolon"
  }
}
{
  "char": "ñ",
  "codepoint": "00f1",
  "positions": {
    "apple-es-qwerty": "Semicolon"
  }
}
```

If positions don't match, debug the generator before continuing. The algorithm will be wrong otherwise.

- [ ] **Step 4: Commit (script only, NOT the generated JSON)**

```bash
git add scripts/build-detection-catalogue.mjs
git commit -m "feat: catalogue generator extracts char positions from layouts"
```

---

## Task 4: Wire the generator into the build pipeline

**Files:**
- Modify: `package.json`
- Modify: `.gitignore`

- [ ] **Step 1: Add the `predev`/`prebuild` scripts to `package.json`**

Update the `"scripts"` block:

```json
"scripts": {
  "dev": "vite",
  "predev": "node scripts/build-detection-catalogue.mjs",
  "build": "vite build",
  "prebuild": "node scripts/build-detection-catalogue.mjs",
  "preview": "vite preview",
  "check": "svelte-check --tsconfig ./tsconfig.app.json && tsc -p tsconfig.node.json",
  "test": "vitest run",
  "test:watch": "vitest",
  "tauri": "tauri"
}
```

(`predev` and `prebuild` are npm lifecycle hooks; they run automatically before `dev` and `build`.)

- [ ] **Step 2: Add the generated file to `.gitignore`**

Append to `.gitignore`:

```
# Detection catalogue is generated by scripts/build-detection-catalogue.mjs
src/lib/detection-catalogue.generated.json
```

- [ ] **Step 3: Verify the hook fires**

Run: `npm run dev -- --help` (the `--help` is so vite exits immediately; `predev` still runs).
Expected: stdout contains `Detection catalogue: ... characters across 6 layouts ...`.

If `predev` doesn't fire, check that the file paths in the script are correct.

- [ ] **Step 4: Commit**

```bash
git add package.json .gitignore
git commit -m "chore: generate detection catalogue on dev/build"
```

---

## Task 5: Add types and stub the detection module

**Files:**
- Modify: `src/lib/types.ts`
- Create: `src/lib/detection.ts`

- [ ] **Step 1: Add types to `src/lib/types.ts`**

Append at the end of the file:

```ts
// ── Character-based detection (see docs/superpowers/specs/2026-04-16-character-based-detection-design.md)

export interface DetectionCharEntry {
  char: string;
  codepoint: string;
  /** Map from layoutId to the DOM event.code where this char is printed on that layout. */
  positions: Record<string, string>;
}

export interface DetectionCatalogue {
  generatedAt: string;
  characters: DetectionCharEntry[];
}

export type DetectionResponse =
  | { kind: "key_pressed"; eventCode: string }
  | { kind: "no_such_key" };

export type DetectionPhase =
  | { kind: "asking"; char: DetectionCharEntry; candidates: string[] }
  | { kind: "detected"; layoutId: string }
  | { kind: "failed" };
```

- [ ] **Step 2: Create `src/lib/detection.ts` with the function signatures (no implementation yet)**

```ts
import type { DetectionCatalogue, DetectionCharEntry, DetectionResponse } from "./types";

/**
 * Returns the set of layoutIds that DO have this char printed on a key.
 */
export function layoutsWithChar(entry: DetectionCharEntry, candidates: string[]): string[] {
  throw new Error("not implemented");
}

/**
 * Picks the catalogue entry that produces the smallest worst-case bucket.
 * Scoring: partition `candidates` by entry.positions[layoutId] (or "ABSENT" if missing).
 * The best entry minimizes the size of its largest bucket.
 * Returns null if no entry distinguishes any two candidates.
 */
export function pickBestQuestion(
  catalogue: DetectionCatalogue,
  candidates: string[],
): DetectionCharEntry | null {
  throw new Error("not implemented");
}

/**
 * Applies a user response to the candidate set and returns the narrowed set.
 * - "key_pressed" with a known eventCode: keep layouts whose entry.positions[id] === eventCode
 * - "no_such_key": keep layouts where entry.positions[id] is undefined (char absent)
 * - "key_pressed" with an unknown eventCode: returns candidates unchanged (caller should treat as "wrong key")
 */
export function applyResponse(
  entry: DetectionCharEntry,
  candidates: string[],
  response: DetectionResponse,
): string[] {
  throw new Error("not implemented");
}

/**
 * True when the user's keypress matches an expected position for at least one candidate.
 */
export function isExpectedPress(
  entry: DetectionCharEntry,
  candidates: string[],
  eventCode: string,
): boolean {
  throw new Error("not implemented");
}
```

- [ ] **Step 3: Verify TypeScript still compiles**

Run: `npm run check`
Expected: 0 errors, 0 warnings.

- [ ] **Step 4: Commit**

```bash
git add src/lib/types.ts src/lib/detection.ts
git commit -m "feat: types and stubs for character-based detection"
```

---

## Task 6: Implement `layoutsWithChar` (TDD)

**Files:**
- Create: `src/lib/detection.test.ts`
- Modify: `src/lib/detection.ts`

- [ ] **Step 1: Write the failing test**

Create `src/lib/detection.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { layoutsWithChar } from "./detection";
import type { DetectionCharEntry } from "./types";

const AT: DetectionCharEntry = {
  char: "@",
  codepoint: "0040",
  positions: {
    "apple-us-qwerty": "Digit2",
    "apple-uk-qwerty": "Quote",
    "apple-fr-azerty": "Backquote",
    "apple-de-qwertz": "KeyL",
    "apple-es-qwerty": "Digit2",
    "apple-it-qwerty": "Semicolon",
  },
};

const NTILDE: DetectionCharEntry = {
  char: "ñ",
  codepoint: "00f1",
  positions: { "apple-es-qwerty": "Semicolon" },
};

describe("layoutsWithChar", () => {
  it("returns all candidates that have the char printed", () => {
    const result = layoutsWithChar(AT, ["apple-us-qwerty", "apple-fr-azerty"]);
    expect(result).toEqual(["apple-us-qwerty", "apple-fr-azerty"]);
  });

  it("excludes candidates that do not have the char", () => {
    const result = layoutsWithChar(NTILDE, ["apple-us-qwerty", "apple-es-qwerty"]);
    expect(result).toEqual(["apple-es-qwerty"]);
  });

  it("returns empty array when no candidate has the char", () => {
    const result = layoutsWithChar(NTILDE, ["apple-us-qwerty", "apple-fr-azerty"]);
    expect(result).toEqual([]);
  });
});
```

- [ ] **Step 2: Run the test and verify it fails**

Run: `npm test`
Expected: FAIL with `Error: not implemented`.

- [ ] **Step 3: Implement `layoutsWithChar`**

Replace the body of `layoutsWithChar` in `src/lib/detection.ts`:

```ts
export function layoutsWithChar(entry: DetectionCharEntry, candidates: string[]): string[] {
  return candidates.filter((id) => entry.positions[id] !== undefined);
}
```

- [ ] **Step 4: Run the test and verify it passes**

Run: `npm test`
Expected: 3 passing tests.

- [ ] **Step 5: Commit**

```bash
git add src/lib/detection.ts src/lib/detection.test.ts
git commit -m "feat: layoutsWithChar — filter candidates by char presence"
```

---

## Task 7: Implement `applyResponse` and `isExpectedPress` (TDD)

**Files:**
- Modify: `src/lib/detection.test.ts`
- Modify: `src/lib/detection.ts`

- [ ] **Step 1: Add failing tests**

Append to `src/lib/detection.test.ts`:

```ts
import { applyResponse, isExpectedPress } from "./detection";

describe("applyResponse", () => {
  const all = ["apple-us-qwerty", "apple-uk-qwerty", "apple-fr-azerty", "apple-de-qwertz", "apple-es-qwerty", "apple-it-qwerty"];

  it("narrows by event.code (Digit2 -> US + ES)", () => {
    const result = applyResponse(AT, all, { kind: "key_pressed", eventCode: "Digit2" });
    expect(result.sort()).toEqual(["apple-es-qwerty", "apple-us-qwerty"]);
  });

  it("narrows by event.code (Backquote -> FR alone)", () => {
    const result = applyResponse(AT, all, { kind: "key_pressed", eventCode: "Backquote" });
    expect(result).toEqual(["apple-fr-azerty"]);
  });

  it("no_such_key removes layouts that have the char (US vs ES with ñ)", () => {
    const result = applyResponse(NTILDE, ["apple-us-qwerty", "apple-es-qwerty"], { kind: "no_such_key" });
    expect(result).toEqual(["apple-us-qwerty"]);
  });

  it("returns candidates unchanged when event.code is unknown for this question", () => {
    const result = applyResponse(AT, all, { kind: "key_pressed", eventCode: "KeyZ" });
    expect(result).toEqual(all);
  });
});

describe("isExpectedPress", () => {
  const all = ["apple-us-qwerty", "apple-fr-azerty"];

  it("true when eventCode matches a position for at least one candidate", () => {
    expect(isExpectedPress(AT, all, "Digit2")).toBe(true);
    expect(isExpectedPress(AT, all, "Backquote")).toBe(true);
  });

  it("false when eventCode does not match any candidate position", () => {
    expect(isExpectedPress(AT, all, "KeyZ")).toBe(false);
  });
});
```

- [ ] **Step 2: Run and verify tests fail**

Run: `npm test`
Expected: 6 failing tests (4 new + 2 new), 3 passing.

- [ ] **Step 3: Implement both functions**

Replace the bodies of `applyResponse` and `isExpectedPress` in `src/lib/detection.ts`:

```ts
export function applyResponse(
  entry: DetectionCharEntry,
  candidates: string[],
  response: DetectionResponse,
): string[] {
  if (response.kind === "no_such_key") {
    return candidates.filter((id) => entry.positions[id] === undefined);
  }
  // key_pressed
  const expectedCodes = new Set(Object.values(entry.positions));
  if (!expectedCodes.has(response.eventCode)) {
    return candidates; // unknown for this question — caller treats as "wrong key"
  }
  return candidates.filter((id) => entry.positions[id] === response.eventCode);
}

export function isExpectedPress(
  entry: DetectionCharEntry,
  candidates: string[],
  eventCode: string,
): boolean {
  return candidates.some((id) => entry.positions[id] === eventCode);
}
```

- [ ] **Step 4: Run tests and verify all pass**

Run: `npm test`
Expected: 9 passing tests.

- [ ] **Step 5: Commit**

```bash
git add src/lib/detection.ts src/lib/detection.test.ts
git commit -m "feat: applyResponse and isExpectedPress — core narrowing logic"
```

---

## Task 8: Implement `pickBestQuestion` (TDD)

**Files:**
- Modify: `src/lib/detection.test.ts`
- Modify: `src/lib/detection.ts`

- [ ] **Step 1: Add failing tests**

Append to `src/lib/detection.test.ts`:

```ts
import { pickBestQuestion } from "./detection";
import type { DetectionCatalogue } from "./types";

const CATALOGUE: DetectionCatalogue = {
  generatedAt: "2026-04-16T00:00:00Z",
  characters: [AT, NTILDE],
};

describe("pickBestQuestion", () => {
  it("returns the entry that minimizes the worst-case bucket", () => {
    // For all 6 layouts, '@' splits into {US,ES}=2, {UK}=1, {FR}=1, {DE}=1, {IT}=1, ABSENT=0 → max bucket 2
    // 'ñ' splits into {ES}=1, ABSENT=5 → max bucket 5
    // '@' wins.
    const all = ["apple-us-qwerty", "apple-uk-qwerty", "apple-fr-azerty", "apple-de-qwertz", "apple-es-qwerty", "apple-it-qwerty"];
    const result = pickBestQuestion(CATALOGUE, all);
    expect(result?.char).toBe("@");
  });

  it("picks ñ when narrowing {US, ES}", () => {
    // For {US, ES}: '@' splits as Digit2=2 (no narrowing) → max 2
    //              'ñ' splits as Semicolon=1, ABSENT=1 → max 1 → wins
    const result = pickBestQuestion(CATALOGUE, ["apple-us-qwerty", "apple-es-qwerty"]);
    expect(result?.char).toBe("ñ");
  });

  it("returns null when no entry distinguishes any candidates", () => {
    // Single candidate left — no question helps
    const result = pickBestQuestion(CATALOGUE, ["apple-us-qwerty"]);
    expect(result).toBeNull();
  });
});
```

- [ ] **Step 2: Run and verify tests fail**

Run: `npm test`
Expected: 3 new failing tests.

- [ ] **Step 3: Implement `pickBestQuestion`**

Replace the body of `pickBestQuestion` in `src/lib/detection.ts`:

```ts
export function pickBestQuestion(
  catalogue: DetectionCatalogue,
  candidates: string[],
): DetectionCharEntry | null {
  if (candidates.length <= 1) return null;

  const score = (entry: DetectionCharEntry): number => {
    // Bucket size by event.code, plus an ABSENT bucket
    const buckets = new Map<string, number>();
    let absent = 0;
    for (const id of candidates) {
      const pos = entry.positions[id];
      if (pos === undefined) {
        absent += 1;
      } else {
        buckets.set(pos, (buckets.get(pos) ?? 0) + 1);
      }
    }
    const maxPositionBucket = buckets.size === 0 ? 0 : Math.max(...buckets.values());
    return Math.max(maxPositionBucket, absent);
  };

  let best: DetectionCharEntry | null = null;
  let bestScore = candidates.length + 1;
  for (const entry of catalogue.characters) {
    const s = score(entry);
    if (s < candidates.length && s < bestScore) {
      bestScore = s;
      best = entry;
    }
  }
  return best;
}
```

- [ ] **Step 4: Run tests and verify all pass**

Run: `npm test`
Expected: 12 passing tests.

- [ ] **Step 5: Commit**

```bash
git add src/lib/detection.ts src/lib/detection.test.ts
git commit -m "feat: pickBestQuestion — minimax scoring with ABSENT bucket"
```

---

## Task 9: Add i18n strings and the failure-message field

**Files:**
- Modify: `src/lib/i18n.ts`
- Modify: `src/lib/stores.svelte.ts`

- [ ] **Step 1: Add the new keys to the `detect` section of both `en` and `fr` blocks in `src/lib/i18n.ts`**

In the `en.detect` block, replace the existing block content with (preserving keys not listed):

```ts
detect: {
  title: "Keyboard Detection",
  instruction: "Find a symbol on your keyboard and press the key it's printed on.",
  charPrompt: "Press the key where you see this symbol",
  charHint: "Find this symbol on your physical keyboard, then press that key. The symbol does not need to appear on screen.",
  noKey: "I don't have this key",
  manual: "Pick manually",
  wrongKey: "Oops, that might be the wrong key. Try again.",
  wrongKeyHelp: "Just press the physical key where you see this symbol. The symbol does not need to appear on screen.",
  failedBanner: "Detection failed. Pick your keyboard manually.",
  back: "Back",
  // Legacy keys kept for now in case other pages reference them; remove in a follow-up cleanup
  pressKey: "Press this key:",
  progress: "Key {current} of {total}",
  analyzing: "Analyzing your keyboard...",
  detected: "Detected layout: {name}",
  notDetected: "Could not identify your keyboard layout.",
  tryAgain: "Try again",
  continue: "Continue",
  fallback: "Or choose manually instead",
},
```

In the `fr.detect` block, mirror with French strings:

```ts
detect: {
  title: "Détection du clavier",
  instruction: "Repérez un symbole sur votre clavier et appuyez sur la touche où il est imprimé.",
  charPrompt: "Appuyez sur la touche où vous voyez ce symbole",
  charHint: "Repérez ce symbole sur votre clavier physique, puis appuyez sur cette touche. Le symbole n'a pas besoin de s'afficher à l'écran.",
  noKey: "Je n'ai pas cette touche",
  manual: "Choisir manuellement",
  wrongKey: "Oups, ce n'est peut-être pas la bonne touche. Réessayez.",
  wrongKeyHelp: "Appuyez simplement sur la touche physique où vous voyez ce symbole. Il n'est pas nécessaire que ce symbole s'affiche à l'écran.",
  failedBanner: "Détection impossible. Choisissez votre clavier manuellement.",
  back: "Retour",
  pressKey: "Appuyez sur cette touche :",
  progress: "Touche {current} sur {total}",
  analyzing: "Analyse de votre clavier...",
  detected: "Clavier détecté : {name}",
  notDetected: "Impossible d'identifier votre clavier.",
  tryAgain: "Réessayer",
  continue: "Continuer",
  fallback: "Ou choisissez manuellement",
},
```

- [ ] **Step 2: Add the failure-message field to `AppState` in `src/lib/stores.svelte.ts`**

Add one line to the `AppState` class:

```ts
class AppState {
  page = $state<Page>("welcome");
  lang = $state<Lang>(detectLang());
  selectedLayoutId = $state<string | null>(null);
  layouts = $state<LayoutMeta[]>([]);
  theme = $state<Theme>("system");
  error = $state<string | null>(null);
  installing = $state(false);
  detectionFailedMessage = $state<string | null>(null);
}
```

- [ ] **Step 3: Verify TypeScript still compiles**

Run: `npm run check`
Expected: 0 errors, 0 warnings.

- [ ] **Step 4: Commit**

```bash
git add src/lib/i18n.ts src/lib/stores.svelte.ts
git commit -m "feat: i18n strings and detection-failed state for character flow"
```

---

## Task 10: Rewrite `Detect.svelte` with the character flow

**Files:**
- Modify: `src/pages/Detect.svelte`

- [ ] **Step 1: Replace the entire `<script lang="ts">` block**

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import {
    pickBestQuestion,
    applyResponse,
    isExpectedPress,
    layoutsWithChar,
  } from "../lib/detection";
  import catalogueJson from "../lib/detection-catalogue.generated.json";
  import type { DetectionCatalogue, DetectionCharEntry } from "../lib/types";

  const catalogue = catalogueJson as DetectionCatalogue;
  const MAX_QUESTIONS = 3;
  const MAX_WRONG_PER_QUESTION = 3;
  const MODIFIER_CODES = new Set([
    "ShiftLeft", "ShiftRight",
    "AltLeft", "AltRight",
    "ControlLeft", "ControlRight",
    "MetaLeft", "MetaRight",
    "CapsLock", "Fn", "FnLock",
  ]);

  let candidates = $state<string[]>(appState.layouts.map((l) => l.id));
  let questionsAsked = $state(0);
  let wrongPresses = $state(0);
  let currentChar = $state<DetectionCharEntry | null>(null);
  let detectedId = $state<string | null>(null);
  let failed = $state(false);
  let showWrongToast = $state(false);
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  function nextQuestion() {
    if (candidates.length === 1) {
      finishSuccess(candidates[0]);
      return;
    }
    if (questionsAsked >= MAX_QUESTIONS) {
      finishFailure();
      return;
    }
    const q = pickBestQuestion(catalogue, candidates);
    if (!q) {
      finishFailure();
      return;
    }
    currentChar = q;
    wrongPresses = 0;
  }

  function finishSuccess(id: string) {
    detectedId = id;
    appState.selectedLayoutId = id;
  }

  function finishFailure() {
    failed = true;
    appState.detectionFailedMessage = t(appState.lang, "detect.failedBanner");
    appState.page = "select";
  }

  function flashWrongToast() {
    showWrongToast = true;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      showWrongToast = false;
      toastTimer = null;
    }, 2500);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!currentChar || detectedId || failed) return;
    if (event.repeat) return;
    if (MODIFIER_CODES.has(event.code)) return;
    if (event.code === "Escape") {
      appState.page = "welcome";
      return;
    }
    event.preventDefault();

    if (isExpectedPress(currentChar, candidates, event.code)) {
      const next = applyResponse(currentChar, candidates, { kind: "key_pressed", eventCode: event.code });
      candidates = next;
      questionsAsked += 1;
      currentChar = null;
      nextQuestion();
    } else {
      wrongPresses += 1;
      flashWrongToast();
    }
  }

  function clickNoKey() {
    if (!currentChar) return;
    const next = applyResponse(currentChar, candidates, { kind: "no_such_key" });
    candidates = next;
    questionsAsked += 1;
    currentChar = null;
    nextQuestion();
  }

  function pickManually() {
    appState.page = "select";
  }

  function goBack() {
    appState.page = "welcome";
  }

  function detectedName(id: string): string {
    const layout = appState.layouts.find((l) => l.id === id);
    return layout?.name[appState.lang] ?? id;
  }

  function goPreview() {
    appState.page = "preview";
  }

  // Helpers for the conditional "I don't have this key" button
  let showNoKeyButton = $derived(
    !!currentChar &&
    layoutsWithChar(currentChar, candidates).length < candidates.length
  );

  let progressPct = $derived((questionsAsked / MAX_QUESTIONS) * 100);

  onMount(() => {
    nextQuestion();
    window.addEventListener("keydown", handleKeydown);
    return () => {
      window.removeEventListener("keydown", handleKeydown);
      if (toastTimer) clearTimeout(toastTimer);
    };
  });
</script>
```

- [ ] **Step 2: Replace the entire markup section (everything between `</script>` and end of file)**

```svelte
<div class="page">
  <div class="page__header">
    <h1 class="page__title">{t(appState.lang, "detect.title")}</h1>
  </div>

  <div class="page__content">
    {#if detectedId}
      <div class="status status--success">
        {t(appState.lang, "detect.detected", { name: detectedName(detectedId) })}
      </div>
      <div class="page__actions">
        <button class="btn btn-primary" onclick={goPreview}>
          {t(appState.lang, "detect.continue")}
        </button>
      </div>
    {:else if currentChar}
      <div
        class="progress-bar"
        role="progressbar"
        aria-valuenow={Math.round(progressPct)}
        aria-valuemin={0}
        aria-valuemax={100}
      >
        <div class="progress-bar__fill" style="width: {progressPct}%"></div>
      </div>

      {#if showWrongToast}
        <div class="status status--error" role="alert">
          {t(appState.lang, "detect.wrongKey")}
        </div>
      {/if}

      {#if wrongPresses >= MAX_WRONG_PER_QUESTION}
        <div class="status status--info" role="status">
          {t(appState.lang, "detect.wrongKeyHelp")}
        </div>
      {/if}

      <div class="detect-prompt">
        <p class="detect-prompt__text">{t(appState.lang, "detect.charPrompt")}</p>
        <div class="detect-prompt__char">{currentChar.char}</div>
        <p class="text-secondary">{t(appState.lang, "detect.charHint")}</p>
      </div>

      <div class="page__actions">
        {#if showNoKeyButton}
          <button class="btn btn-secondary" onclick={clickNoKey}>
            {t(appState.lang, "detect.noKey")}
          </button>
        {/if}
        <button class="btn btn-secondary" onclick={pickManually}>
          {t(appState.lang, "detect.manual")}
        </button>
        <button class="btn btn-secondary" onclick={goBack}>
          {t(appState.lang, "detect.back")}
        </button>
      </div>
    {:else}
      <div class="spinner"></div>
    {/if}
  </div>
</div>

<style>
  .detect-prompt {
    text-align: center;
    margin: 2rem auto;
    max-width: 480px;
  }
  .detect-prompt__text {
    font-size: 1rem;
    color: var(--color-text-secondary);
    margin-bottom: 1.5rem;
  }
  .detect-prompt__char {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 120px;
    height: 120px;
    margin: 0 auto 1.5rem;
    font-size: 64px;
    font-weight: 400;
    color: var(--color-text);
    background: linear-gradient(180deg, #ffffff 0%, #f0f0f4 100%);
    border-radius: 18px;
    border: 1px solid rgba(0,0,0,0.08);
    box-shadow:
      0 0.5px 0 rgba(255,255,255,0.95) inset,
      0 4px 14px rgba(0,0,0,0.10);
  }
  :root[data-theme="dark"] .detect-prompt__char,
  :root:not([data-theme="light"]) .detect-prompt__char {
    background: linear-gradient(180deg, #5a5a5e 0%, #4a4a4c 100%);
    border-color: rgba(0,0,0,0.45);
    color: #f5f5f7;
  }
</style>
```

- [ ] **Step 3: Verify TypeScript still compiles**

Run: `npm run check`
Expected: 0 errors, 0 warnings.

- [ ] **Step 4: Smoke-test in the dev server**

Run: `npm run tauri dev` in one terminal.

Manual test in the app:
1. Click "Auto-detect my keyboard" on Welcome.
2. The Detect page shows a large "@" prompt.
3. Press the `@` key on your physical keyboard (with whatever modifier you need to find it).
4. Either: detection completes (single layout reached), OR a second prompt for "ñ" appears.
5. Press "ñ" if visible, or click "Je n'ai pas cette touche" if not.
6. Final layout name is shown; "Continue" advances to Preview.
7. Press a wildly wrong key (e.g. spacebar when asked for `@`): the "Oops" toast appears.
8. After 3 wrong presses: the extended help banner appears.
9. The "Choisir manuellement" button is always visible during the prompt and routes to Select.

Note any UI issues but don't block on cosmetics — fix them in Task 12.

- [ ] **Step 5: Commit**

```bash
git add src/pages/Detect.svelte
git commit -m "feat: rewrite Detect.svelte with character-based flow"
```

---

## Task 11: Render the failure banner in `Select.svelte`

**Files:**
- Modify: `src/pages/Select.svelte`

- [ ] **Step 1: Read the current header section to find the insertion point**

Run: `head -75 src/pages/Select.svelte`

Locate the line `<div class="page__content">` (around line 65 per earlier inspection).

- [ ] **Step 2: Insert the banner just inside `page__content`**

After the `<div class="page__content">` opening tag, add:

```svelte
{#if appState.detectionFailedMessage}
  <div class="status status--info" role="status">
    {appState.detectionFailedMessage}
  </div>
{/if}
```

- [ ] **Step 3: Clear the message when leaving Select**

In the `<script>` block of `Select.svelte`, ensure the message is cleared once the user has seen it. Add this line at the end of the existing script block (or wherever existing handlers live; if the script imports `onMount`, add a cleanup; otherwise add a `$effect` block):

```ts
import { onMount } from "svelte";
onMount(() => {
  return () => {
    appState.detectionFailedMessage = null;
  };
});
```

(If `onMount` is already imported and used, only add the inner cleanup return inside the existing call. Don't double-import.)

- [ ] **Step 4: Verify TypeScript still compiles**

Run: `npm run check`
Expected: 0 errors, 0 warnings.

- [ ] **Step 5: Smoke-test the banner**

In the running dev session:
1. Start detection.
2. Press wrong keys 3 times for each of 3 questions (force `MAX_QUESTIONS` exhaustion). Or simpler: dispatch a fake failure by editing `nextQuestion` temporarily — but easier is to just fail naturally.
3. After failure, Select page renders with the banner at the top reading *"Détection impossible. Choisissez votre clavier manuellement."*
4. Picking a layout and going back to Welcome then re-entering Select shows no banner.

- [ ] **Step 6: Commit**

```bash
git add src/pages/Select.svelte
git commit -m "feat: inline detection-failed banner on Select page"
```

---

## Task 12: Add `mk-body--neutral` modifier and apply it during detection

**Files:**
- Modify: `src/app.css`
- Modify: `src/pages/Detect.svelte`

- [ ] **Step 1: Add the modifier to `src/app.css`**

After the existing `.mk-body { ... }` block (around line 462), add:

```css
.mk-body--neutral {
  filter: saturate(0.6);
  opacity: 0.75;
}
```

- [ ] **Step 2: Render a neutral mockup in `Detect.svelte`**

The current Detect.svelte (post-rewrite from Task 10) does not include a keyboard mockup. Per spec: "keep [the mockup] visible (neutral, no highlight)" during the prompt. Add a minimal mockup using `KeyboardVisual` is overkill (it requires a Layout); instead reuse the inline `mk-*` markup but stripped of any active-key logic.

In `src/pages/Detect.svelte`, add this just AFTER the `</div>` that closes `page__actions` and BEFORE the closing `</div>` of `page__content`, **only inside the `{:else if currentChar}` branch**:

```svelte
<div class="mk-body mk-body--neutral" aria-hidden="true">
  <div class="mk-row mk-row--fn">
    <div class="mk-key mk-key--fn-esc"><span class="mk-lbl-sm">esc</span></div>
    {#each Array(12) as _, i}
      <div class="mk-key mk-key--fn"><span class="mk-lbl-fn">F{i + 1}</span></div>
    {/each}
    <div class="mk-touchid"></div>
  </div>
  <div class="mk-row">
    {#each Array(13) as _}<div class="mk-key"></div>{/each}
    <div class="mk-key mk-key--delete"><span class="mk-lbl-sm">delete</span></div>
  </div>
  <div class="mk-row">
    <div class="mk-key mk-key--tab"><span class="mk-lbl-sm">tab</span></div>
    {#each Array(12) as _}<div class="mk-key"></div>{/each}
    <div class="mk-key mk-key--enter-top"></div>
  </div>
  <div class="mk-row">
    <div class="mk-key mk-key--caps"><span class="mk-lbl-sm">caps lock</span></div>
    {#each Array(12) as _}<div class="mk-key"></div>{/each}
    <div class="mk-key mk-key--enter-bot"><span class="mk-lbl-sm">return</span></div>
  </div>
  <div class="mk-row">
    <div class="mk-key mk-key--lshift"><span class="mk-lbl-sm">shift</span></div>
    {#each Array(11) as _}<div class="mk-key"></div>{/each}
    <div class="mk-key mk-key--rshift"><span class="mk-lbl-sm">shift</span></div>
  </div>
  <div class="mk-row mk-row--bottom">
    <div class="mk-key mk-key--mod1"><span class="mk-lbl-xs">fn</span></div>
    <div class="mk-key mk-key--mod1"><span class="mk-lbl-xs">control</span></div>
    <div class="mk-key mk-key--mod1"><span class="mk-lbl-xs">option</span></div>
    <div class="mk-key mk-key--cmd"><span class="mk-lbl-xs">command</span></div>
    <div class="mk-key mk-key--space"></div>
    <div class="mk-key mk-key--cmd"><span class="mk-lbl-xs">command</span></div>
    <div class="mk-key mk-key--mod1"><span class="mk-lbl-xs">option</span></div>
    <div class="mk-arrows">
      <div class="mk-arrow mk-arrow--l"><span class="mk-arrow-glyph">◀</span></div>
      <div class="mk-arrow-stack">
        <div class="mk-arrow mk-arrow--h"><span class="mk-arrow-glyph">▲</span></div>
        <div class="mk-arrow mk-arrow--h"><span class="mk-arrow-glyph">▼</span></div>
      </div>
      <div class="mk-arrow mk-arrow--l"><span class="mk-arrow-glyph">▶</span></div>
    </div>
  </div>
</div>
```

- [ ] **Step 3: Verify TypeScript still compiles**

Run: `npm run check`
Expected: 0 errors, 0 warnings.

- [ ] **Step 4: Smoke-test the neutral mockup**

In the dev session, the Detect page now shows the keyboard mockup below the actions, slightly desaturated and dimmed.

- [ ] **Step 5: Commit**

```bash
git add src/app.css src/pages/Detect.svelte
git commit -m "feat: neutral keyboard mockup on Detect page during prompt"
```

---

## Task 13: Final integration verification

**Files:** none (verification only)

- [ ] **Step 1: Clean rebuild**

Run: `rm -f src/lib/detection-catalogue.generated.json && npm run build`
Expected: build completes without errors. The catalogue is regenerated fresh.

- [ ] **Step 2: Run full test suite**

Run: `npm test`
Expected: 12 passing, 0 failing.

- [ ] **Step 3: Run type check**

Run: `npm run check`
Expected: 0 errors, 0 warnings.

- [ ] **Step 4: End-to-end manual test in dev**

Run: `npm run tauri dev`

Walk through these flows:

| Scenario | Expected outcome |
|----------|------------------|
| Welcome → Auto-detect → press `@` (US position Digit2) → press `ñ` no-key button | Detected as `apple-us-qwerty` |
| Welcome → Auto-detect → press `@` (Backquote) | Detected as `apple-fr-azerty` immediately |
| Welcome → Auto-detect → press `@` (Quote) | Detected as `apple-uk-qwerty` immediately |
| Welcome → Auto-detect → press `KeyZ` (wildly wrong) | Toast appears; no candidate change |
| Welcome → Auto-detect → press `KeyZ` 3× | Extended-help banner appears |
| Welcome → Auto-detect → click "Choisir manuellement" | Goes to Select with no failure banner |
| Welcome → Auto-detect → exhaust questions wrongly | Routes to Select with the failure banner |

- [ ] **Step 5: Final commit (if any cleanup files were touched)**

```bash
git status
# If clean, no commit needed.
# If anything changed during smoke-testing, commit it.
```

---

## Summary

After completing all tasks, the result is:

- A pure TypeScript detection algorithm with full unit test coverage (12 tests).
- A build-time catalogue generator that keeps `layouts/*.json` as the single source of truth.
- A new Detect.svelte that asks 1-2 questions max, handles wrong keys without penalizing the user's question budget, and routes failures gracefully to manual selection.
- The "Choisir manuellement" escape hatch is always one click away.
