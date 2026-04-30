export interface LayoutMeta {
  id: string;
  name: Record<string, string>;
  locale: string;
  description: Record<string, string>;
  /** DLL filename without extension (e.g. "kbdaplfr"). Surfaced by `list_layouts`
   * so the UI can derive a DLL→layout-id map from the runtime layout list
   * (Settings page, Health Check button gating). */
  dllName: string;
}

export interface DetectionKey {
  eventCode: string;
  prompt: Record<string, string>;
  expectedBase: string;
}

export interface KeyMapping {
  vk: string;
  cap: string;
  base: string;
  shift: string;
  ctrl: string;
  altgr: string;
  altgrShift: string;
}

export interface DeadKey {
  name: string;
  combinations: Record<string, string>;
}

export interface Layout {
  id: string;
  name: Record<string, string>;
  locale: string;
  localeId: string;
  dllName: string;
  description: Record<string, string>;
  detectionKeys: DetectionKey[];
  keys: Record<string, KeyMapping>;
  deadKeys: Record<string, DeadKey>;
}

export interface DetectionResult {
  eventCode: string;
  receivedChar: string;
}

export type Page = "welcome" | "detect" | "select" | "preview" | "install" | "test" | "done" | "about" | "modifiers" | "settings" | "healthCheck" | "bluetoothPairing";
export type Lang = "en" | "fr";
export type Theme = "light" | "dark" | "system";

// ── Character-based detection (see docs/superpowers/specs/2026-04-16-character-based-detection-design.md)

export interface DetectionCharEntry {
  char: string;
  codepoint: string;
  /**
   * Map from layoutId to the list of DOM event.code values where this char may be pressed
   * on that layout. Multiple positions account for chars printed on more than one key in
   * the same layout AND Apple-on-Windows hardware quirks (e.g. ISO section-key swap, where
   * the top-left key reports IntlBackslash instead of Backquote on Apple ISO boards).
   * The first entry is the canonical position used for question-scoring.
   */
  positions: Record<string, string[]>;
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

// ── Mac-style modifier keys (see docs/superpowers/specs/2026-04-17-mac-style-modifiers-design.md)

export interface ModifierToggles {
  swapCmdCtrlLeft: boolean;
  swapCmdCtrlRight: boolean;
  capsToCtrl: boolean;
  swapOptionCmd: boolean;
}

export interface RawScancodePair {
  /** 4-hex-char little-endian scancode emitted (e.g. "1D00" for LCtrl). */
  newCode: string;
  /** 4-hex-char little-endian scancode received from the keyboard. */
  oldCode: string;
}

export interface ModifierState {
  /** Best-effort reverse-derivation of which toggles match the current registry value. */
  current: ModifierToggles;
  /** True if the registry has entries that don't correspond to any of our toggles. */
  hasExternalMappings: boolean;
  /** All raw pairs found in the registry (for the warning details). */
  rawEntries: RawScancodePair[];
}

// ── Settings page: system layout management
// (see docs/superpowers/specs/2026-04-19-settings-error-flow-design.md)

export interface InstalledLayoutInfo {
  /** 8-char registry subkey name, e.g. "0000040c". */
  klid: string;
  /** `Layout File` registry value, e.g. "KBDFR.DLL" or "kbdaplfr.dll". */
  layoutFile: string;
  /** `Layout Text` registry value; may be empty. */
  layoutText: string;
  /** True when this layout was installed by MagicWindows (layoutFile starts with "kbdapl"). */
  isMagicWindows: boolean;
  /** True when this KLID appears in HKCU\Keyboard Layout\Preload. */
  isInUse: boolean;
}

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

// ── F12 / Eject remap (see src-tauri/src/keyboard/f12_remap.rs)

export type F12Action =
  | "default" | "disabled" | "calculator" | "search"
  | "mail" | "appsMenu" | "volumeMute";

// ── Apple keyboard hardware probe

export interface AppleKeyboardInfo {
  friendlyName: string;
  hardwareId: string;
  status: string;
}
