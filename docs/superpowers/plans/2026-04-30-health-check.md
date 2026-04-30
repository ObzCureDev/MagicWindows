# Health Check Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Health Check page that lets the user verify every key on the installed layout produces the expected character, with visual feedback on `KeyboardVisual` and a one-click control-key assertion that catches Chromium-breaking regressions like the Shift+Enter bug fixed in `a8f8b7f`.

**Architecture:**
- New Svelte page `HealthCheck.svelte` reachable from Settings (auto-show post-install deferred to v2).
- Free-form typing: user types whatever they want; each keystroke is compared to the layout's expected codepoint for that `(scancode, modifierState)` tuple.
- `KeyboardVisual` reused with a per-key status overlay (untested / passed / failed).
- New Rust Tauri command `health_check_control_keys(klid)` loads the **target** layout via `LoadKeyboardLayoutW(klid, KLF_NOTELLSHELL)` and runs a `ToUnicodeEx` probe against that exact HKL for Enter, Shift+Enter, Tab, Backspace, Esc — asserts each one produces no `wchar` output (regression test for the bug we just fixed). Tests the *installed MagicWindows layout*, not the active foreground layout.
- Settings provides the target (`klid`, `layoutId`) via `appState.healthCheckTarget` so the page knows both which probe to run and which JSON spec to compare keystrokes against.
- Combined report (session + control-key probe + app version) exportable as JSON, suitable for pasting into bug reports the same way `collect_diagnostics` works today.

**Tech Stack:** Svelte 5 runes, Tauri v2, Rust (windows-rs or P/Invoke via PowerShell consistent with `diagnostics.rs`), Vitest for frontend logic.

**Out of MVP scope:**
- Auto-show after install (only entry point in MVP = Settings).
- Dead-key compose flow validation (marked "not testable in MVP" in the UI; full coverage later).
- Per-layout regression CI matrix (separate plan once feature stabilises).

---

## File Structure

| File | Status | Responsibility |
|---|---|---|
| `src/lib/types.ts` | modify | Add `HealthCheckResult`, `HealthCheckSession`, `KeyStatus`, `HealthCheckTarget`; extend `Page` with `"healthCheck"` |
| `src/lib/stores.svelte.ts` | modify | Add `healthCheckTarget: HealthCheckTarget \| null` to `AppState` |
| `src/lib/healthCheck.ts` | **create** | Pure comparison logic: expected char given `(layout, scancode, modifiers)`. Vitest-friendly. |
| `src/lib/healthCheck.test.ts` | **create** | Vitest tests for the comparison logic |
| `src/pages/HealthCheck.svelte` | **create** | The page itself: input capture, status overlay, summary |
| `src/components/KeyboardVisual.svelte` | modify | Accept optional `keyStatus: Record<string, KeyStatus>` prop and overlay colour rings |
| `src/pages/Settings.svelte` | modify | Add per-installed-layout "Run health check" button that sets `healthCheckTarget` and navigates |
| `src/App.svelte` | modify | Route `"healthCheck"` to `HealthCheck.svelte` (no props — page reads `healthCheckTarget`) |
| `src/lib/i18n.ts` | modify | Add EN/FR keys for the new page |
| `src-tauri/src/keyboard/health_check.rs` | **create** | Win32 control-key probe via `LoadKeyboardLayoutW` + `ToUnicodeEx` against the target HKL |
| `src-tauri/src/keyboard/mod.rs` | modify | Wire the new module + re-export the command |
| `src-tauri/src/lib.rs` | modify | Register `health_check_control_keys` Tauri command |

Total: **9 files modified, 5 created** (counting `README.md` in Task 7.3 and `src/lib/scancode.ts` extracted in Task 3.3).

---

## Task 1: Frontend types + comparison helper (TDD foundation)

**Files:**
- Modify: `src/lib/types.ts`
- Create: `src/lib/healthCheck.ts`
- Test: `src/lib/healthCheck.test.ts`

- [ ] **Step 1.1: Add types**

In `src/lib/types.ts`, append:

```ts
// ── Health Check (post-install verification)

export type KeyStatus = "untested" | "passed" | "failed";

export type ModState = {
  shift: boolean;
  altgr: boolean;   // event.getModifierState('AltGraph')
  capsLock: boolean;
};

export interface HealthCheckResult {
  /** Lowercase 2-hex scancode, e.g. "1e" for A on US/QWERTY position */
  scancode: string;
  modifiers: ModState;
  /** 4-hex-char Unicode codepoint expected from the layout JSON, e.g. "0040" */
  expectedCodepoint: string;
  /** What the keystroke actually produced (single char, possibly empty) */
  receivedChar: string;
  status: KeyStatus;
  /** ms since session start, for ordering */
  at: number;
}

export interface HealthCheckSession {
  layoutId: string;
  /** Installed KLID the probe ran against (e.g. "a000040c") */
  klid: string;
  startedAt: string;       // ISO timestamp
  results: HealthCheckResult[];
}

/**
 * Set by Settings when launching the health check.
 * The page reads this from appState — it is required (page redirects to
 * Settings if null).
 */
export interface HealthCheckTarget {
  /** Layout JSON id, e.g. "apple-fr-azerty" */
  layoutId: string;
  /** Installed KLID, e.g. "a000040c" — comes from InstalledLayoutInfo, NOT layout.localeId */
  klid: string;
}
```

**Naming note.** A `ModifierState` interface already exists in `types.ts` (the
mac-style modifier toggles state), so the health-check type is named
`ModState` to avoid a collision. Keep that name throughout the feature.

Also update `Page`:

```ts
export type Page = "welcome" | "detect" | "select" | "preview" | "install" | "test" | "done" | "about" | "modifiers" | "settings" | "healthCheck";
```

In `src/lib/stores.svelte.ts`, add the field to the `AppState` class:

```ts
healthCheckTarget = $state<HealthCheckTarget | null>(null);
```

(Import `HealthCheckTarget` at the top.)

- [ ] **Step 1.2: Write failing test**

Create `src/lib/healthCheck.test.ts`:

```ts
import { describe, it, expect } from "vitest";
import { expectedCodepointFor, compareKeystroke } from "./healthCheck";
import type { Layout, ModState } from "./types";

const fixture: Layout = {
  id: "test", name: { en: "T" }, locale: "fr", localeId: "0000040c",
  dllName: "kbdtest", description: { en: "" }, detectionKeys: [],
  deadKeys: {},
  keys: {
    // 'a' / 'A' — alphabetic, cap: "1" so Caps Lock affects it
    "10": { vk: "VK_Q", cap: "1", base: "0061", shift: "0041", ctrl: "-1", altgr: "-1", altgrShift: "-1" },
    // 2 / @ on AltGr — digit row, cap: "0" so Caps Lock does NOT affect it
    "03": { vk: "VK_2", cap: "0", base: "0026", shift: "0032", ctrl: "-1", altgr: "0040", altgrShift: "-1" },
    // Dead key marker on AltGr — must be reported as untestable in MVP
    "1a": { vk: "VK_OEM_4", cap: "0", base: "005e@", shift: "00a8@", ctrl: "-1", altgr: "-1", altgrShift: "-1" },
  } as any,
};

const noMods: ModState = { shift: false, altgr: false, capsLock: false };

describe("expectedCodepointFor", () => {
  it("returns base codepoint with no modifiers", () => {
    expect(expectedCodepointFor(fixture, "10", noMods)).toBe("0061"); // a
  });
  it("returns shift codepoint when shift held", () => {
    expect(expectedCodepointFor(fixture, "10", { ...noMods, shift: true })).toBe("0041"); // A
  });
  it("returns altgr codepoint when AltGraph held", () => {
    expect(expectedCodepointFor(fixture, "03", { ...noMods, altgr: true })).toBe("0040"); // @
  });
  it("inverts base/shift for cap: '1' keys when CapsLock is on", () => {
    // Caps Lock on, no Shift → must yield uppercase 'A' for an alphabetic key
    expect(expectedCodepointFor(fixture, "10", { ...noMods, capsLock: true })).toBe("0041");
  });
  it("CapsLock + Shift on cap: '1' keys yields the base char (mutual cancel)", () => {
    expect(expectedCodepointFor(fixture, "10", { shift: true, altgr: false, capsLock: true })).toBe("0061");
  });
  it("CapsLock has no effect on cap: '0' keys", () => {
    // Digit '2' must stay '2' regardless of CapsLock
    expect(expectedCodepointFor(fixture, "03", { ...noMods, capsLock: true })).toBe("0026");
  });
  it("returns null for unmapped scancode", () => {
    expect(expectedCodepointFor(fixture, "ff", noMods)).toBeNull();
  });
  it("returns null when slot is -1 (no character produced)", () => {
    expect(expectedCodepointFor(fixture, "10", { ...noMods, altgr: true })).toBeNull();
  });
  it("returns null for dead-key slots (suffix '@' is not testable in MVP)", () => {
    expect(expectedCodepointFor(fixture, "1a", noMods)).toBeNull();
    expect(expectedCodepointFor(fixture, "1a", { ...noMods, shift: true })).toBeNull();
  });
});

describe("compareKeystroke", () => {
  it("passes when received char matches expected codepoint", () => {
    expect(compareKeystroke("0040", "@")).toBe("passed");
  });
  it("fails when received char differs", () => {
    expect(compareKeystroke("0040", "a")).toBe("failed");
  });
  it("fails when received is empty but expected is set", () => {
    expect(compareKeystroke("0040", "")).toBe("failed");
  });
});
```

- [ ] **Step 1.3: Run test — verify it fails**

Run: `npm test`
Expected: FAIL with `Cannot find module './healthCheck'`.

- [ ] **Step 1.4: Implement `healthCheck.ts`**

Create `src/lib/healthCheck.ts`:

```ts
import type { Layout, ModState, KeyStatus } from "./types";

const NO_CHAR = "-1";

/**
 * Look up the codepoint the layout JSON says this key should produce
 * given the current modifier state. Returns null if the combination is
 * untestable in MVP, i.e.:
 *  - the scancode is not in the layout
 *  - the slot for this modifier combo is "-1" (no character produced)
 *  - the slot is a dead-key marker (suffix "@") — dead-key compose flow
 *    is out of MVP scope. Pressing such a key produces e.key="Dead" or a
 *    composing buffer, not a comparable single char.
 *
 * CapsLock is honoured for keys flagged `cap: "1"` in the layout JSON
 * (alphabetic in most layouts). For those keys, the effective shift is
 * `shift XOR capsLock`. For `cap: "0"` keys (digits, punctuation),
 * CapsLock has no effect. `cap: "SGCap"` is treated as `"0"` for MVP —
 * SGCAPS is rare and would need its own logic.
 */
export function expectedCodepointFor(
  layout: Layout,
  scancode: string,
  mods: ModState,
): string | null {
  const key = layout.keys[scancode.toLowerCase()];
  if (!key) return null;

  const capsAffectsKey = key.cap === "1";
  const effectiveShift = capsAffectsKey ? (mods.shift !== mods.capsLock) : mods.shift;

  let raw: string;
  if (mods.altgr && effectiveShift) raw = key.altgrShift;
  else if (mods.altgr) raw = key.altgr;
  else if (effectiveShift) raw = key.shift;
  else raw = key.base;

  if (!raw || raw === NO_CHAR) return null;
  // Dead-key markers are untestable in MVP — surface as null so the UI
  // can label them "not testable" rather than mark a false failure.
  if (raw.endsWith("@")) return null;
  return raw;
}

/** "0040" + "@" → "passed". "0040" + "a" → "failed". */
export function compareKeystroke(expectedCodepoint: string, received: string): KeyStatus {
  if (received.length === 0) return "failed";
  const expected = String.fromCodePoint(parseInt(expectedCodepoint, 16));
  return received === expected ? "passed" : "failed";
}
```

- [ ] **Step 1.5: Run test — verify it passes**

Run: `npm test`
Expected: PASS — all 12 cases (9 in `expectedCodepointFor`, 3 in `compareKeystroke`).

- [ ] **Step 1.6: Commit**

```bash
git add src/lib/types.ts src/lib/healthCheck.ts src/lib/healthCheck.test.ts
git commit -m "feat(health-check): add comparison helper + types"
```

---

## Task 2: KeyboardVisual status overlay

**Files:**
- Modify: `src/components/KeyboardVisual.svelte`

- [ ] **Step 2.1: Add `keyStatus` prop**

Open the `<script lang="ts">` block in `KeyboardVisual.svelte`. Add `keyStatus` to the props rune (search for `let { ... } = $props()`):

```ts
let {
  layout,
  highlightScancode = $bindable(null),
  // ...existing props...
  keyStatus = {} as Record<string, "untested" | "passed" | "failed">,
}: {
  layout: Layout;
  highlightScancode?: string | null;
  // ...existing types...
  keyStatus?: Record<string, "untested" | "passed" | "failed">;
} = $props();
```

- [ ] **Step 2.2: Add status ring CSS**

In the `<style>` block (or scoped style), add:

```css
.key.status-passed { box-shadow: 0 0 0 2px var(--success, #34d399) inset; }
.key.status-failed { box-shadow: 0 0 0 2px var(--danger, #f87171) inset; }
.key.status-untested { /* default — no extra ring */ }
```

- [ ] **Step 2.3: Apply class on each rendered key**

Find where the keys are rendered (look for `class="key"`). Add the dynamic class:

```svelte
<button
  class={`key status-${keyStatus[sc] ?? "untested"}`}
  ...
>
```

(Replace `sc` with whatever variable holds the current scancode in the loop.)

- [ ] **Step 2.4: Manual smoke test**

Run: `npm run check`
Expected: 0 errors, 0 warnings.

- [ ] **Step 2.5: Commit**

```bash
git add src/components/KeyboardVisual.svelte
git commit -m "feat(keyboard-visual): support per-key status overlay"
```

---

## Task 3: HealthCheck page scaffold + routing

**Files:**
- Create: `src/pages/HealthCheck.svelte`
- Modify: `src/App.svelte`
- Modify: `src/lib/i18n.ts`

- [ ] **Step 3.1: Add i18n keys**

In `src/lib/i18n.ts`, add to both `en` and `fr` blocks (use the existing flat-key style):

```ts
// EN
"healthCheck.title": "Health Check",
"healthCheck.subtitle": "Press every key — we'll verify each one matches the installed layout.",
"healthCheck.passed": "Passed",
"healthCheck.failed": "Failed",
"healthCheck.untested": "Untested",
"healthCheck.runControlKeys": "Run control-key check",
"healthCheck.controlKeysOk": "Control keys OK — Enter, Shift+Enter, Tab, Backspace, Esc behave correctly.",
"healthCheck.controlKeysFail": "Control-key regression detected: {keys}",
"healthCheck.exportReport": "Export report (JSON)",
"healthCheck.summary": "{passed} passed · {failed} failed · {untested} untested",
"healthCheck.back": "Back to Settings",

// FR
"healthCheck.title": "Diagnostic clavier",
"healthCheck.subtitle": "Appuyez sur chaque touche — on vérifie qu'elle correspond à la disposition installée.",
"healthCheck.passed": "OK",
"healthCheck.failed": "Échec",
"healthCheck.untested": "Non testée",
"healthCheck.runControlKeys": "Lancer le test des touches de contrôle",
"healthCheck.controlKeysOk": "Touches de contrôle OK — Entrée, Maj+Entrée, Tab, Retour, Échap fonctionnent correctement.",
"healthCheck.controlKeysFail": "Régression détectée sur les touches de contrôle : {keys}",
"healthCheck.exportReport": "Exporter le rapport (JSON)",
"healthCheck.summary": "{passed} OK · {failed} échec · {untested} non testées",
"healthCheck.back": "Retour aux paramètres",
```

- [ ] **Step 3.2: Create the page**

Create `src/pages/HealthCheck.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import KeyboardVisual from "../components/KeyboardVisual.svelte";
  import type { Layout, KeyStatus, HealthCheckSession, ModState } from "../lib/types";
  import { expectedCodepointFor, compareKeystroke } from "../lib/healthCheck";
  import { SCANCODE_BY_EVENT_CODE } from "../lib/scancode";

  // Page reads its target from appState. Settings is the only entry point
  // and is responsible for setting healthCheckTarget before navigating.
  let target = $derived(appState.healthCheckTarget);
  let layout = $state<Layout | null>(null);

  // If we land here without a target (deep-link / refresh), bounce to Settings.
  $effect(() => {
    if (!target) {
      appState.page = "settings";
    }
  });

  onMount(async () => {
    if (!target) return;
    // The Settings page already loads layout JSON via its own pipeline.
    // For the health-check page we re-fetch by id from the bundled layouts.
    // Pattern mirrors how Test.svelte loads its layout — copy that import path.
    const all = await import("../lib/layouts").then(m => m.loadAllLayouts());
    layout = all.find(l => l.id === target!.layoutId) ?? null;
  });

  let session = $state<HealthCheckSession>({
    layoutId: target?.layoutId ?? "",
    klid: target?.klid ?? "",
    startedAt: new Date().toISOString(),
    results: [],
  });

  // Worst status wins (a single failure marks the key red even if it later passes).
  let keyStatus = $derived.by(() => {
    const map: Record<string, KeyStatus> = {};
    for (const r of session.results) {
      const prev = map[r.scancode];
      if (r.status === "failed") map[r.scancode] = "failed";
      else if (prev !== "failed") map[r.scancode] = r.status;
    }
    return map;
  });

  let summary = $derived.by(() => {
    let passed = 0, failed = 0;
    for (const v of Object.values(keyStatus)) {
      if (v === "passed") passed++;
      else if (v === "failed") failed++;
    }
    const total = layout ? Object.keys(layout.keys).length : 0;
    return { passed, failed, untested: total - passed - failed };
  });

  function onKeydown(e: KeyboardEvent) {
    if (!layout) return;
    const sc = SCANCODE_BY_EVENT_CODE[e.code];
    if (!sc) return;
    e.preventDefault();
    const mods: ModState = {
      shift: e.shiftKey,
      altgr: e.getModifierState("AltGraph"),
      capsLock: e.getModifierState("CapsLock"),
    };
    const expected = expectedCodepointFor(layout, sc, mods);
    if (expected === null) return; // dead key / unmapped / -1 → not testable in MVP
    const received = e.key.length === 1 ? e.key : "";
    const status = compareKeystroke(expected, received);
    session.results = [
      ...session.results,
      {
        scancode: sc,
        modifiers: mods,
        expectedCodepoint: expected,
        receivedChar: received,
        status,
        at: Date.now() - new Date(session.startedAt).getTime(),
      },
    ];
  }
</script>

<svelte:window onkeydown={onKeydown} />

{#if layout}
  <section class="page">
    <header>
      <h1>{t(appState.lang, "healthCheck.title")}</h1>
      <p>{t(appState.lang, "healthCheck.subtitle")}</p>
    </header>

    <KeyboardVisual {layout} {keyStatus} />

    <div class="actions">
      <button onclick={() => (appState.page = "settings")}>
        {t(appState.lang, "healthCheck.back")}
      </button>
      <!-- exportReport defined in Task 6 once the control-key probe is wired -->
    </div>

    <p class="summary">
      {t(appState.lang, "healthCheck.summary")
        .replace("{passed}", String(summary.passed))
        .replace("{failed}", String(summary.failed))
        .replace("{untested}", String(summary.untested))}
    </p>
  </section>
{:else}
  <p>Loading…</p>
{/if}
```

**Implementation note.** The `import("../lib/layouts")` line assumes there is
already a frontend module that exposes `loadAllLayouts()` returning the JSON
array. If the existing pattern in `Test.svelte`, `Preview.svelte`, etc. uses a
different mechanism (e.g. a Vite glob import or a Tauri command), copy that
exact pattern verbatim instead — do **not** invent a new loader.

- [ ] **Step 3.3: Extract scancode-from-event-code map**

Inspect `src/components/KeyboardVisual.svelte` — there is already a mapping from `event.code` to scancode used for the highlight feature. Extract it to `src/lib/scancode.ts`, export `SCANCODE_BY_EVENT_CODE`, and import from both `KeyboardVisual` and `HealthCheck`.

- [ ] **Step 3.4: Wire route in `App.svelte`**

Import the new page at the top:

```svelte
import HealthCheck from "./pages/HealthCheck.svelte";
```

In the routing block (the `{#if ... appState.page === "..."}` chain at line ~130), add a branch before `{/if}`:

```svelte
{:else if appState.page === "healthCheck"}
  <HealthCheck />
```

The page reads `appState.healthCheckTarget` itself and bounces back to
Settings if it is null — no prop wiring is required at the routing layer.

- [ ] **Step 3.5: Type check**

Run: `npm run check`
Expected: 0 errors.

- [ ] **Step 3.6: Commit**

```bash
git add src/pages/HealthCheck.svelte src/App.svelte src/lib/i18n.ts src/lib/scancode.ts src/components/KeyboardVisual.svelte
git commit -m "feat(health-check): add page scaffold + routing"
```

---

## Task 4: Settings entry point

**Files:**
- Modify: `src/pages/Settings.svelte`
- Modify: `src/lib/i18n.ts`

- [ ] **Step 4.1: Add Settings i18n key**

In `src/lib/i18n.ts`:

```ts
// EN
"settings.healthCheck": "Run keyboard health check",
"settings.healthCheckHint": "Verify every key on the installed layout produces the correct character.",

// FR
"settings.healthCheck": "Lancer le diagnostic clavier",
"settings.healthCheckHint": "Vérifie que chaque touche de la disposition installée produit le bon caractère.",
```

- [ ] **Step 4.2: Add a per-layout health-check button**

The KLID we pass to the probe must be the **installed** KLID — the same one
Rust writes to `HKLM\SOFTWARE\MagicWindows` at install time, surfaced in
`InstalledLayoutInfo.klid`. We attach a "health check" button to each
MagicWindows-installed layout row in the Settings list (existing
`{#each installedLayouts as info}` loop).

In `src/pages/Settings.svelte`, inside the row rendering for each
`InstalledLayoutInfo` where `info.isMagicWindows`, add:

```svelte
<button
  class="btn-secondary"
  onclick={() => {
    // Defense in depth: even though the surrounding {#if layoutIdFromDll(...)}
    // hides this button for unknown DLLs, the click handler also bails out
    // rather than navigate to HealthCheck with an empty layoutId.
    const layoutId = layoutIdFromDll(info.layoutFile);
    if (!layoutId) return;
    appState.healthCheckTarget = { layoutId, klid: info.klid };
    appState.page = "healthCheck";
  }}
>
  {t(appState.lang, "settings.healthCheck")}
</button>
```

Also add the helper at the top of the `<script>` block:

```ts
// "kbdaplfr.dll" → "apple-fr-azerty". Mirrors the dllName ↔ id convention
// used in layouts/*.json (see layout.dllName field).
const DLL_TO_LAYOUT_ID: Record<string, string> = {
  "kbdaplus.dll": "apple-us-qwerty",
  "kbdaplfr.dll": "apple-fr-azerty",
  "kbdapluk.dll": "apple-uk-qwerty",
  "kbdaplde.dll": "apple-de-qwertz",
  "kbdaples.dll": "apple-es-qwerty",
  "kbdaplit.dll": "apple-it-qwerty",
};

function layoutIdFromDll(dllName: string): string {
  return DLL_TO_LAYOUT_ID[dllName.toLowerCase()] ?? "";
}
```

If `layoutIdFromDll` returns empty (unknown DLL), hide the button — we have
no JSON to compare against. Wrap the button:

```svelte
{#if layoutIdFromDll(info.layoutFile)}
  <button …>…</button>
{/if}
```

- [ ] **Step 4.3: Manual smoke test**

Run: `npm run tauri dev`
1. Click gear → land on Settings.
2. Verify the new section is visible.
3. Click "Run keyboard health check" → land on HealthCheck.
4. Press a few keys — visual rings update (green/red).
5. Click "Back to Settings" → return.

- [ ] **Step 4.4: Commit**

```bash
git add src/pages/Settings.svelte src/lib/i18n.ts
git commit -m "feat(settings): add health-check entry point"
```

---

## Task 5: Rust control-key probe

**Files:**
- Create: `src-tauri/src/keyboard/health_check.rs`
- Modify: `src-tauri/src/keyboard/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 5.1: Define the test result type**

Create `src-tauri/src/keyboard/health_check.rs`:

```rust
//! Control-key regression probe.
//!
//! Loads the **target** keyboard layout via `LoadKeyboardLayoutW(klid, …)`
//! and calls `ToUnicodeEx` for VK_RETURN, Shift+VK_RETURN, VK_TAB, VK_BACK,
//! VK_ESCAPE against that exact HKL. Each one must produce no `wchar` output
//! — if it does, Chromium-based apps (Antigravity, VSCode, Slack…) will
//! swallow the keystroke as text input. This is the regression that broke
//! Shift+Enter before commit a8f8b7f.
//!
//! We deliberately accept the target KLID as a parameter rather than calling
//! `GetKeyboardLayout(0)`: the active foreground layout in the PowerShell
//! child process is not guaranteed to be the MagicWindows layout the user is
//! testing.

use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ControlKeyResult {
    pub name: String,         // "Enter", "Shift+Enter", ...
    pub vk: u8,
    pub shift: bool,
    pub passed: bool,
    /// hex representation of what ToUnicodeEx produced (empty = no output)
    pub produced: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ControlKeyReport {
    /// Echo of the KLID that was loaded for the probe.
    pub klid: String,
    pub results: Vec<ControlKeyResult>,
    pub all_passed: bool,
}
```

- [ ] **Step 5.2: Implement the Win32 probe**

Append to `health_check.rs`:

```rust
#[cfg(target_os = "windows")]
#[tauri::command]
pub fn health_check_control_keys(klid: String) -> Result<ControlKeyReport, String> {
    use std::process::Command;

    // Sanity-check the KLID before injecting it into a PowerShell string.
    // KLIDs are 8 hex characters (the MagicWindows ones start with 'a').
    if klid.len() != 8 || !klid.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("invalid KLID: {klid:?}"));
    }

    // We mirror the existing pattern in diagnostics.rs: shell out to
    // PowerShell with an inline P/Invoke. Avoids bringing in `windows-rs`
    // for a single call.
    let script = format!(r#"
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public class Probe {{
  [DllImport("user32.dll", CharSet=CharSet.Unicode)]
  public static extern IntPtr LoadKeyboardLayout(string klid, uint flags);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)]
  public static extern int ToUnicodeEx(uint vk, uint sc, byte[] state, StringBuilder buf, int sz, uint flags, IntPtr hkl);
  [DllImport("user32.dll")]
  public static extern uint MapVirtualKeyEx(uint code, uint mapType, IntPtr hkl);

  public static string Probe(IntPtr hkl, uint vk, bool shift) {{
    // MAPVK_VK_TO_VSC_EX = 4 (returns the scancode, including extended bit)
    uint sc = MapVirtualKeyEx(vk, 4, hkl);
    var state = new byte[256];
    if (shift) state[0x10] = 0x80;
    var buf = new StringBuilder(8);
    int n = ToUnicodeEx(vk, sc, state, buf, buf.Capacity, 0, hkl);
    return n > 0 ? buf.ToString() : "";
  }}
}}
'@

# KLF_NOTELLSHELL = 0x80, KLF_SUBSTITUTE_OK = 0x02. Loads without changing
# the active foreground layout for the calling thread.
$hkl = [Probe]::LoadKeyboardLayout('{klid}', 0x82)
if ($hkl -eq [IntPtr]::Zero) {{
  Write-Error "LoadKeyboardLayout failed for KLID '{klid}'"
  exit 1
}}

$o = New-Object PSObject -Property @{{
  klid = '{klid}'
  enter = [Probe]::Probe($hkl, 0x0D, $false)
  shiftEnter = [Probe]::Probe($hkl, 0x0D, $true)
  tab = [Probe]::Probe($hkl, 0x09, $false)
  back = [Probe]::Probe($hkl, 0x08, $false)
  esc = [Probe]::Probe($hkl, 0x1B, $false)
}}
$o | ConvertTo-Json -Compress
"#);

    let output = Command::new("powershell")
        .args(["-ExecutionPolicy", "Bypass", "-NoProfile", "-Command", &script])
        .output()
        .map_err(|e| format!("spawn powershell: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "probe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(json.trim())
        .map_err(|e| format!("parse probe output: {e}\nraw: {json}"))?;

    let mk = |name: &str, vk: u8, shift: bool, key: &str| {
        let produced = parsed[key].as_str().unwrap_or("");
        // PASS = no wchar produced. Any character (including 0x000A or
        // 0x000D) is a fail because Chromium will treat it as text input.
        let passed = produced.is_empty();
        let produced_hex = produced
            .chars()
            .map(|c| format!("{:04X}", c as u32))
            .collect::<Vec<_>>()
            .join(",");
        ControlKeyResult { name: name.into(), vk, shift, passed, produced: produced_hex }
    };

    let results = vec![
        mk("Enter",       0x0D, false, "enter"),
        mk("Shift+Enter", 0x0D, true,  "shiftEnter"),
        mk("Tab",         0x09, false, "tab"),
        mk("Backspace",   0x08, false, "back"),
        mk("Escape",      0x1B, false, "esc"),
    ];
    let all_passed = results.iter().all(|r| r.passed);

    Ok(ControlKeyReport { klid, results, all_passed })
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn health_check_control_keys(_klid: String) -> Result<ControlKeyReport, String> {
    Err("health_check is only available on Windows".into())
}
```

- [ ] **Step 5.3: Wire into mod.rs**

In `src-tauri/src/keyboard/mod.rs`, add:

```rust
pub mod health_check;
pub use health_check::{health_check_control_keys, ControlKeyReport, ControlKeyResult};
```

- [ ] **Step 5.4: Register the command**

In `src-tauri/src/lib.rs`, find the `tauri::generate_handler![ ... ]` block and add `keyboard::health_check_control_keys` to the list.

- [ ] **Step 5.5: Build check**

Run from `src-tauri/`:
```bash
cargo check
cargo clippy -- -D warnings
```
Expected: 0 errors, 0 warnings.

- [ ] **Step 5.6: Commit**

```bash
git add src-tauri/src/keyboard/health_check.rs src-tauri/src/keyboard/mod.rs src-tauri/src/lib.rs
git commit -m "feat(rust): control-key probe via ToUnicodeEx"
```

---

## Task 6: Wire control-key probe to UI

**Files:**
- Modify: `src/pages/HealthCheck.svelte`

- [ ] **Step 6.1: Call from frontend with the target KLID**

In `HealthCheck.svelte`, add to the `<script>` block:

```ts
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";

interface ControlKeyResult { name: string; passed: boolean; produced: string; }
interface ControlKeyReport { klid: string; results: ControlKeyResult[]; all_passed: boolean; }

let controlReport = $state<ControlKeyReport | null>(null);
let controlError = $state<string | null>(null);

async function runControlKeyCheck() {
  if (!target) return;
  controlError = null;
  controlReport = null;
  try {
    // Tauri command parameter casing: snake_case in Rust, camelCase here.
    controlReport = await invoke<ControlKeyReport>("health_check_control_keys", {
      klid: target.klid,
    });
  } catch (e) {
    controlError = String(e);
  }
}

async function exportReport() {
  const appVersion = await getVersion().catch(() => "unknown");
  const payload = {
    appVersion,
    exportedAt: new Date().toISOString(),
    session,
    controlReport,
    controlError,
  };
  const blob = new Blob([JSON.stringify(payload, null, 2)], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `magicwindows-health-${session.layoutId}-${Date.now()}.json`;
  a.click();
  URL.revokeObjectURL(url);
}
```

In the template, after the keyboard visual section (and inside the
`{#if layout}` guard added in Task 3):

```svelte
<section class="control-keys">
  <button onclick={runControlKeyCheck}>
    {t(appState.lang, "healthCheck.runControlKeys")}
  </button>

  {#if controlReport}
    {#if controlReport.all_passed}
      <p class="ok">{t(appState.lang, "healthCheck.controlKeysOk")}</p>
    {:else}
      <p class="fail">
        {t(appState.lang, "healthCheck.controlKeysFail").replace(
          "{keys}",
          controlReport.results.filter((r) => !r.passed).map((r) => r.name).join(", ")
        )}
      </p>
    {/if}
    <ul class="control-results">
      {#each controlReport.results as r}
        <li class={r.passed ? "ok" : "fail"}>
          <strong>{r.name}</strong> — {r.passed ? "OK" : `produced ${r.produced}`}
        </li>
      {/each}
    </ul>
  {/if}

  {#if controlError}
    <p class="fail">{controlError}</p>
  {/if}
</section>
```

And add the export button next to "Back" in the actions row from Task 3:

```svelte
<button onclick={exportReport}>
  {t(appState.lang, "healthCheck.exportReport")}
</button>
```

- [ ] **Step 6.2: Manual end-to-end test**

Run: `npm run tauri dev`
1. Install the FR layout (writes the KLID to `HKLM\SOFTWARE\MagicWindows\LastInstalledKLID`).
2. Open Settings → identify the row for the Apple FR layout → click "Run keyboard health check" on that row (the button only appears for `isMagicWindows` rows).
3. Click "Run control-key check".
4. **Expected after the codegen fix in `a8f8b7f`:** all 5 entries pass; report shows `klid` = the Apple-prefixed KLID (e.g. `a000040c`).
5. As a sanity check, temporarily revert the codegen fix (`0x000D, 0x000D` → `0x000D, 0x000A`), rebuild, reinstall — `Shift+Enter` should now show as failed with `produced 000A`. Restore the fix.
6. Verify the JSON export contains `session`, `controlReport`, `appVersion`, and `exportedAt` top-level keys.

- [ ] **Step 6.3: Commit**

```bash
git add src/pages/HealthCheck.svelte
git commit -m "feat(health-check): surface control-key probe in UI"
```

---

## Task 7: Wrap-up

- [ ] **Step 7.1: Full type check**

Run: `npm run check`
Expected: 0 errors.

- [ ] **Step 7.2: All tests**

Run: `npm test`
Expected: all green.

```bash
cd src-tauri && cargo test
```
Expected: all green.

- [ ] **Step 7.3: README / changelog mention**

Add a one-line mention to `README.md` under the existing feature list:
- "Health check: verify each key matches the installed layout, with one-click control-key regression probe."

- [ ] **Step 7.4: Final commit**

```bash
git add README.md
git commit -m "docs: mention health-check feature in README"
```

---

## Self-Review Notes

- **Spec coverage:** Free-form typing ✓ (Task 3), KeyboardVisual overlay ✓ (Task 2), control-key assertion against the **target** KLID ✓ (Task 5), per-layout entry from Settings ✓ (Task 4), CapsLock honoured for `cap: "1"` keys ✓ (Task 1), dead-key slots return null ✓ (Task 1), combined JSON export with control report + app version ✓ (Task 6), Svelte 5 native event syntax (`onclick`/`onkeydown`) ✓ (consistent with `App.svelte` line 59), i18n signature `t(appState.lang, key)` ✓ (matches existing usage). MVP exclusions documented ✓ (header).
- **Placeholder scan:** `SCANCODE_BY_EVENT_CODE` is initialised in step 3.3 by extracting from `KeyboardVisual`. The page imports it from `../lib/scancode` in step 3.2. The layout loader (`loadAllLayouts`) is documented as "copy the existing pattern from Test/Preview" — implementer must inspect those files and either reuse the same import or expose a small wrapper. Not a placeholder, but flagged so the implementer doesn't invent a new mechanism. ✓
- **Type consistency:** `KeyStatus`, `ModState`, `HealthCheckTarget`, `HealthCheckSession` defined in Task 1 and used consistently in Tasks 2–6. `ControlKeyReport` defined in Task 5, consumed in Task 6 with matching `klid`/`results`/`all_passed` field names. The Rust struct uses snake_case (`all_passed`); Tauri's serde default keeps it as-is, matching the TS interface. ✓
- **DRY:** Scancode mapping extracted to `src/lib/scancode.ts` in 3.3 instead of duplicating between `KeyboardVisual` and `HealthCheck`. The probe lives in its own module (`health_check.rs`) rather than being grafted onto `diagnostics.rs`. ✓
- **Naming collision avoided:** Frontend `ModState` is named distinctly from the existing `ModifierState` (mac-style toggles) — Step 1.1 calls this out explicitly so the implementer doesn't try to merge them. ✓
- **Out-of-MVP labelled:** Dead-key compose flow returns `null` from `expectedCodepointFor` rather than producing false failures (Task 1). Auto-show post-install is excluded from this plan. The summary's "untested" count includes dead-key slots since they cannot be tested in MVP — acceptable trade-off; documented in the spec header.

---

## Execution Handoff

**Plan complete and saved to `docs/superpowers/plans/2026-04-30-health-check.md`.**

Two execution options:

1. **Subagent-Driven (recommended)** — fresh subagent per task, review between tasks, faster iteration cycle and protected main context.
2. **Inline Execution** — execute tasks in this same session with checkpoints for review.

Which approach do you want?
