export interface LayoutMeta {
  id: string;
  name: Record<string, string>;
  locale: string;
  description: Record<string, string>;
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

export type Page = "welcome" | "detect" | "select" | "preview" | "install" | "done" | "about";
export type Lang = "en" | "fr";
export type Theme = "light" | "dark" | "system";

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
