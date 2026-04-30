import { describe, it, expect } from "vitest";
import { sha256, manifestEntry, layoutMetadata } from "./build-web-release.mjs";

describe("sha256", () => {
  it("hashes a known buffer correctly", () => {
    const hex = sha256(Buffer.from("hello"));
    // SHA-256 of "hello"
    expect(hex).toBe("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
  });
});

describe("manifestEntry", () => {
  it("builds a download entry from inputs", () => {
    const entry = manifestEntry({
      url: "https://x/y.zip",
      size: 12345,
      sha256: "deadbeef",
    });
    expect(entry).toEqual({ url: "https://x/y.zip", size: 12345, sha256: "deadbeef" });
  });
});

describe("layoutMetadata", () => {
  it("extracts dllName, displayName (en), localeId from a layout JSON", () => {
    const layout = {
      id: "apple-fr-azerty",
      name: { en: "FR MagicKeyboard", fr: "FR MagicKeyboard" },
      locale: "fr-FR",
      localeId: "0000040c",
      dllName: "kbdaplfr",
      description: { en: "...", fr: "..." },
      detectionKeys: [],
      keys: {},
      deadKeys: {},
    };
    expect(layoutMetadata(layout)).toEqual({
      dllName: "kbdaplfr",
      displayName: "FR MagicKeyboard",
      localeId: "0000040c",
    });
  });

  it("rejects a layout missing dllName", () => {
    expect(() => layoutMetadata({ name: { en: "x" }, localeId: "0000040c" }))
      .toThrow(/dllName/);
  });

  it("rejects a malformed localeId", () => {
    expect(() => layoutMetadata({ dllName: "x", name: { en: "x" }, localeId: "bad" }))
      .toThrow(/localeId/);
  });
});
