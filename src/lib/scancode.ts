// Maps DOM `KeyboardEvent.code` values to the 2-hex-char Windows PS/2
// scancodes used as keys in the layout JSON files (`layouts/*.json`).
//
// Coverage matches every scancode that appears in any layout under
// `layouts/`. The inverse direction (scancode → event.code) lives in
// `scripts/scancode-map.mjs` and is used by the catalogue build script;
// keep the two files in sync if you add or remove keys.
//
// Sources: USB HID Usage Tables + W3C UI Events KeyboardEvent code spec.
//
// NOTE on Apple ISO hardware: the section/`<` keys (scancodes `29` and
// `56`) are physically swapped vs a generic PC ISO board when an Apple
// Magic Keyboard is plugged into Windows. The DOM `event.code` reports
// the *Windows* scancode the OS sees, not the printed keycap, so this
// table lists the standard PC mapping. `KeyboardVisual.svelte` mirrors
// the visual swap for ISO boards but the underlying scancode lookup
// stays identical — see the comment at the top of that component.

export const SCANCODE_BY_EVENT_CODE: Record<string, string> = {
  // Number / top row
  Backquote: "29",
  Digit1: "02",
  Digit2: "03",
  Digit3: "04",
  Digit4: "05",
  Digit5: "06",
  Digit6: "07",
  Digit7: "08",
  Digit8: "09",
  Digit9: "0a",
  Digit0: "0b",
  Minus: "0c",
  Equal: "0d",

  // Letter row 1 (QWERTY)
  KeyQ: "10",
  KeyW: "11",
  KeyE: "12",
  KeyR: "13",
  KeyT: "14",
  KeyY: "15",
  KeyU: "16",
  KeyI: "17",
  KeyO: "18",
  KeyP: "19",
  BracketLeft: "1a",
  BracketRight: "1b",

  // Letter row 2 (home row)
  KeyA: "1e",
  KeyS: "1f",
  KeyD: "20",
  KeyF: "21",
  KeyG: "22",
  KeyH: "23",
  KeyJ: "24",
  KeyK: "25",
  KeyL: "26",
  Semicolon: "27",
  Quote: "28",
  Backslash: "2b",

  // Letter row 3 (bottom row)
  IntlBackslash: "56",
  KeyZ: "2c",
  KeyX: "2d",
  KeyC: "2e",
  KeyV: "2f",
  KeyB: "30",
  KeyN: "31",
  KeyM: "32",
  Comma: "33",
  Period: "34",
  Slash: "35",

  // Space
  Space: "39",
};

/**
 * Returns the layout-JSON scancode (lowercase 2-hex chars) for a DOM
 * `KeyboardEvent.code`, or `undefined` if no mapping exists (e.g. the
 * key pressed isn't a character key the layout JSON cares about).
 */
export function scancodeFromEventCode(eventCode: string): string | undefined {
  return SCANCODE_BY_EVENT_CODE[eventCode];
}
