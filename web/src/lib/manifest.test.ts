import { describe, it, expect } from "vitest";
import { isValidManifest } from "./manifest";

describe("isValidManifest", () => {
  it("accepts a complete dual-arch manifest", () => {
    const valid = {
      version: "0.4.0",
      generatedAt: "2026-05-02T12:00:00Z",
      downloads: {
        "apple-fr-azerty": {
          x64:   { url: "https://x/y-x64.zip",   size: 1234, sha256: "abc" },
          arm64: { url: "https://x/y-arm64.zip", size: 1300, sha256: "def" },
        },
      },
    };
    expect(isValidManifest(valid)).toBe(true);
  });

  it("accepts a manifest with only one arch per layout", () => {
    // Realistic when a build environment lacks the ARM64 cross-compiler.
    const valid = {
      version: "0.4.0",
      generatedAt: "2026-05-02T12:00:00Z",
      downloads: {
        "apple-fr-azerty": {
          x64: { url: "https://x/y-x64.zip", size: 1234, sha256: "abc" },
        },
      },
    };
    expect(isValidManifest(valid)).toBe(true);
  });

  it("rejects a manifest missing version", () => {
    const invalid = { generatedAt: "now", downloads: {} };
    expect(isValidManifest(invalid)).toBe(false);
  });

  it("rejects a manifest with malformed download entry", () => {
    const invalid = {
      version: "0.4.0",
      generatedAt: "now",
      downloads: { "apple-fr-azerty": { x64: { url: "x" } } },
    };
    expect(isValidManifest(invalid)).toBe(false);
  });

  it("rejects unknown architecture keys", () => {
    const invalid = {
      version: "0.4.0",
      generatedAt: "now",
      downloads: {
        "apple-fr-azerty": {
          x86: { url: "x", size: 1, sha256: "h" },
        },
      },
    };
    expect(isValidManifest(invalid)).toBe(false);
  });

  it("rejects null and undefined", () => {
    expect(isValidManifest(null)).toBe(false);
    expect(isValidManifest(undefined)).toBe(false);
  });
});
