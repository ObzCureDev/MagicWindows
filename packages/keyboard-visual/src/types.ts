export interface KeyMapping {
  vk: string;
  cap: string;
  base: string;
  shift: string;
  ctrl: string;
  altgr: string;
  altgrShift: string;
}

export interface Layout {
  keys: Record<string, KeyMapping>;
}

export type KeyStatus = "untested" | "passed" | "failed";
