import { describe, it, expect } from "vitest";
import { isValidManifest } from "./manifest";

describe("isValidManifest", () => {
  it("accepts a complete manifest", () => {
    const valid = {
      version: "0.4.0",
      generatedAt: "2026-05-01T12:00:00Z",
      downloads: {
        "apple-fr-azerty": {
          url: "https://example.com/x.zip",
          size: 1234,
          sha256: "abc",
        },
      },
    };
    expect(isValidManifest(valid)).toBe(true);
  });

  it("rejects a manifest missing version", () => {
    const invalid = { generatedAt: "now", downloads: {} };
    expect(isValidManifest(invalid)).toBe(false);
  });

  it("rejects a manifest with malformed downloads entry", () => {
    const invalid = {
      version: "0.4.0",
      generatedAt: "now",
      downloads: { "apple-fr-azerty": { url: "x" } },
    };
    expect(isValidManifest(invalid)).toBe(false);
  });

  it("rejects null and undefined", () => {
    expect(isValidManifest(null)).toBe(false);
    expect(isValidManifest(undefined)).toBe(false);
  });
});
