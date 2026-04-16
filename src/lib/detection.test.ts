import { describe, it, expect } from "vitest";
import { layoutsWithChar, applyResponse, isExpectedPress } from "./detection";
import type { DetectionCharEntry } from "./types";

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
