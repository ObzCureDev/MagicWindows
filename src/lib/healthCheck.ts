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
