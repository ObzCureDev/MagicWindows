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
