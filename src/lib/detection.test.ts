import { describe, it, expect } from "vitest";
import { layoutsWithChar, applyResponse, isExpectedPress, pickBestQuestion } from "./detection";
import type { DetectionCharEntry, DetectionCatalogue } from "./types";

const AT: DetectionCharEntry = {
  char: "@",
  codepoint: "0040",
  positions: {
    "apple-us-qwerty": "Digit2",
    "apple-uk-qwerty": "Quote",
    "apple-fr-azerty": "Backquote",
    "apple-de-qwertz": "KeyL",
    "apple-es-qwerty": "Digit2",
    "apple-it-qwerty": "Semicolon",
  },
};

const NTILDE: DetectionCharEntry = {
  char: "ñ",
  codepoint: "00f1",
  positions: { "apple-es-qwerty": "Semicolon" },
};

describe("layoutsWithChar", () => {
  it("returns all candidates that have the char printed", () => {
    const result = layoutsWithChar(AT, ["apple-us-qwerty", "apple-fr-azerty"]);
    expect(result).toEqual(["apple-us-qwerty", "apple-fr-azerty"]);
  });

  it("excludes candidates that do not have the char", () => {
    const result = layoutsWithChar(NTILDE, ["apple-us-qwerty", "apple-es-qwerty"]);
    expect(result).toEqual(["apple-es-qwerty"]);
  });

  it("returns empty array when no candidate has the char", () => {
    const result = layoutsWithChar(NTILDE, ["apple-us-qwerty", "apple-fr-azerty"]);
    expect(result).toEqual([]);
  });
});

describe("applyResponse", () => {
  const all = ["apple-us-qwerty", "apple-uk-qwerty", "apple-fr-azerty", "apple-de-qwertz", "apple-es-qwerty", "apple-it-qwerty"];

  it("narrows by event.code (Digit2 -> US + ES)", () => {
    const result = applyResponse(AT, all, { kind: "key_pressed", eventCode: "Digit2" });
    expect(result.sort()).toEqual(["apple-es-qwerty", "apple-us-qwerty"]);
  });

  it("narrows by event.code (Backquote -> FR alone)", () => {
    const result = applyResponse(AT, all, { kind: "key_pressed", eventCode: "Backquote" });
    expect(result).toEqual(["apple-fr-azerty"]);
  });

  it("no_such_key removes layouts that have the char (US vs ES with ñ)", () => {
    const result = applyResponse(NTILDE, ["apple-us-qwerty", "apple-es-qwerty"], { kind: "no_such_key" });
    expect(result).toEqual(["apple-us-qwerty"]);
  });

  it("returns candidates unchanged when event.code is unknown for this question", () => {
    const result = applyResponse(AT, all, { kind: "key_pressed", eventCode: "KeyZ" });
    expect(result).toEqual(all);
  });
});

describe("isExpectedPress", () => {
  const all = ["apple-us-qwerty", "apple-fr-azerty"];

  it("true when eventCode matches a position for at least one candidate", () => {
    expect(isExpectedPress(AT, all, "Digit2")).toBe(true);
    expect(isExpectedPress(AT, all, "Backquote")).toBe(true);
  });

  it("false when eventCode does not match any candidate position", () => {
    expect(isExpectedPress(AT, all, "KeyZ")).toBe(false);
  });
});

describe("pickBestQuestion", () => {
  const CATALOGUE: DetectionCatalogue = {
    generatedAt: "2026-04-16T00:00:00Z",
    characters: [AT, NTILDE],
  };

  it("returns the entry that minimizes the worst-case bucket", () => {
    // For all 6 layouts, '@' splits into {US,ES}=2, {UK}=1, {FR}=1, {DE}=1, {IT}=1, ABSENT=0 → max bucket 2
    // 'ñ' splits into {ES}=1, ABSENT=5 → max bucket 5
    // '@' wins.
    const all = ["apple-us-qwerty", "apple-uk-qwerty", "apple-fr-azerty", "apple-de-qwertz", "apple-es-qwerty", "apple-it-qwerty"];
    const result = pickBestQuestion(CATALOGUE, all);
    expect(result?.char).toBe("@");
  });

  it("picks ñ when narrowing {US, ES}", () => {
    // For {US, ES}: '@' splits as Digit2=2 (no narrowing) → max 2
    //              'ñ' splits as Semicolon=1, ABSENT=1 → max 1 → wins
    const result = pickBestQuestion(CATALOGUE, ["apple-us-qwerty", "apple-es-qwerty"]);
    expect(result?.char).toBe("ñ");
  });

  it("returns null when no entry distinguishes any candidates", () => {
    // Single candidate left — no question helps
    const result = pickBestQuestion(CATALOGUE, ["apple-us-qwerty"]);
    expect(result).toBeNull();
  });
});
