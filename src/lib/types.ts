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

export type Page = "welcome" | "detect" | "select" | "preview" | "install" | "done";
export type Lang = "en" | "fr";
