import { describe, it, expect } from "vitest";
import { parseHash } from "./router";

describe("parseHash", () => {
  it("treats empty hash as home", () => {
    expect(parseHash("")).toEqual({ kind: "home" });
    expect(parseHash("#")).toEqual({ kind: "home" });
    expect(parseHash("#/")).toEqual({ kind: "home" });
  });

  it("recognises the desktop route", () => {
    expect(parseHash("#/desktop")).toEqual({ kind: "desktop" });
    expect(parseHash("#desktop")).toEqual({ kind: "desktop" });
  });

  it("parses preview routes with a layout id", () => {
    expect(parseHash("#/preview/apple-fr-azerty")).toEqual({
      kind: "preview",
      layoutId: "apple-fr-azerty",
    });
  });

  it("rejects unknown routes", () => {
    expect(parseHash("#/foo")).toEqual({ kind: "notFound" });
    expect(parseHash("#/preview/")).toEqual({ kind: "notFound" });
    expect(parseHash("#/preview/UPPER")).toEqual({ kind: "notFound" });
  });

  it("ignores trailing slashes", () => {
    expect(parseHash("#/desktop/")).toEqual({ kind: "desktop" });
  });
});
