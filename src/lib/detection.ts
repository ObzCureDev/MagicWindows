import type { DetectionCatalogue, DetectionCharEntry, DetectionResponse } from "./types";

/**
 * Returns the set of layoutIds that DO have this char printed on a key.
 */
export function layoutsWithChar(entry: DetectionCharEntry, candidates: string[]): string[] {
  return candidates.filter((id) => entry.positions[id] !== undefined);
}

/**
 * Picks the catalogue entry that produces the smallest worst-case bucket.
 * Scoring: partition `candidates` by entry.positions[layoutId] (or "ABSENT" if missing).
 * The best entry minimizes the size of its largest bucket.
 * Returns null if no entry distinguishes any two candidates.
 */
export function pickBestQuestion(
  catalogue: DetectionCatalogue,
  candidates: string[],
): DetectionCharEntry | null {
  if (candidates.length <= 1) return null;

  const score = (entry: DetectionCharEntry): number => {
    // Bucket size by event.code, plus an ABSENT bucket
    const buckets = new Map<string, number>();
    let absent = 0;
    for (const id of candidates) {
      const pos = entry.positions[id];
      if (pos === undefined) {
        absent += 1;
      } else {
        buckets.set(pos, (buckets.get(pos) ?? 0) + 1);
      }
    }
    const maxPositionBucket = buckets.size === 0 ? 0 : Math.max(...buckets.values());
    return Math.max(maxPositionBucket, absent);
  };

  let best: DetectionCharEntry | null = null;
  let bestScore = candidates.length + 1;
  for (const entry of catalogue.characters) {
    const s = score(entry);
    if (s < candidates.length && s < bestScore) {
      bestScore = s;
      best = entry;
    }
  }
  return best;
}

/**
 * Applies a user response to the candidate set and returns the narrowed set.
 * - "key_pressed" with a known eventCode: keep layouts whose entry.positions[id] === eventCode
 * - "no_such_key": keep layouts where entry.positions[id] is undefined (char absent)
 * - "key_pressed" with an unknown eventCode: returns candidates unchanged (caller should treat as "wrong key")
 */
export function applyResponse(
  entry: DetectionCharEntry,
  candidates: string[],
  response: DetectionResponse,
): string[] {
  if (response.kind === "no_such_key") {
    return candidates.filter((id) => entry.positions[id] === undefined);
  }
  // key_pressed
  const expectedCodes = new Set(Object.values(entry.positions));
  if (!expectedCodes.has(response.eventCode)) {
    return candidates; // unknown for this question — caller treats as "wrong key"
  }
  return candidates.filter((id) => entry.positions[id] === response.eventCode);
}

/**
 * True when the user's keypress matches an expected position for at least one candidate.
 */
export function isExpectedPress(
  entry: DetectionCharEntry,
  candidates: string[],
  eventCode: string,
): boolean {
  return candidates.some((id) => entry.positions[id] === eventCode);
}
