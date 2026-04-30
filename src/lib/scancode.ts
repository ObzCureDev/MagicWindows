import { SCANCODE_TO_CODE } from "../../scripts/scancode-map.mjs";

// Sources: USB HID Usage Tables + W3C UI Events KeyboardEvent code spec.
//
// NOTE on Apple ISO hardware: the section/`<` keys (scancodes `29` and
// `56`) are physically swapped vs a generic PC ISO board when an Apple
// Magic Keyboard is plugged into Windows. The DOM `event.code` reports
// the *Windows* scancode the OS sees, not the printed keycap, so the
// canonical table lists the standard PC mapping. `KeyboardVisual.svelte`
// mirrors the visual swap for ISO boards but the underlying scancode
// lookup stays identical — see the comment at the top of that component.

/**
 * Maps DOM `KeyboardEvent.code` values (e.g. "KeyA", "Digit2", "Backquote",
 * "IntlBackslash") to the 2-hex layout-JSON scancode strings used in
 * `layouts/*.json`.
 *
 * This is the inverse of the canonical table in scripts/scancode-map.mjs —
 * THAT file is the source of truth. Do not edit the entries here directly;
 * update the build script and the inverse table will follow.
 */
export const SCANCODE_BY_EVENT_CODE: Record<string, string> = Object.fromEntries(
  Object.entries(SCANCODE_TO_CODE).map(([sc, code]) => [code as string, sc]),
);

/**
 * Returns the layout-JSON scancode (lowercase 2-hex chars) for a DOM
 * `KeyboardEvent.code`, or `null` if no mapping exists (e.g. the
 * key pressed isn't a character key the layout JSON cares about).
 */
export function scancodeFromEventCode(code: string): string | null {
  return SCANCODE_BY_EVENT_CODE[code] ?? null;
}
