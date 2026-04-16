# Character-Based Keyboard Layout Detection

**Date:** 2026-04-16
**Status:** Approved — ready for implementation plan
**Replaces:** Position-based detection currently in [src/pages/Detect.svelte](../../../src/pages/Detect.svelte)

## Problem

The current detection flow asks the user to press keys at named physical positions ("press the first key on the upper letter row"). This is unintuitive — users identify keys by the **symbol printed on them**, not by abstract row/column descriptions.

## Solution

Ask the user to **press the physical key on which a given symbol is printed** (e.g. *"Appuyez sur la touche où vous voyez `@`"*). We capture `event.code` (the physical position, layout-independent) and match it to the position where each Apple layout prints that character.

**Critical UX point:** the user does NOT have to make the symbol *appear on screen* — they only need to press the key where the symbol is printed. We never look at `event.key`. This must be made very explicit in the microcopy, otherwise users will hunt for modifier shortcuts they don't need.

## The key insight

`event.code` always returns the physical key position regardless of the active Windows layout. So if the user looks at their physical keyboard, finds the key marked "@", and presses it (with or without any modifier — irrelevant), we get the position deterministically.

## Algorithm

### Data: where is each character printed?

Derived from inspection of the six layouts in `layouts/*.json`:

**`@` (U+0040) physical position per Apple layout:**

| Layout | Scancode | event.code | Layer (informational) |
|--------|----------|------------|-----------------------|
| apple-us-qwerty | 03 | Digit2 | shift |
| apple-uk-qwerty | 28 | Quote | shift |
| apple-fr-azerty | 29 | Backquote | base (top-left, very prominent) |
| apple-de-qwertz | 26 | KeyL | altgr (Option) |
| apple-es-qwerty | 03 | Digit2 | altgr |
| apple-it-qwerty | 27 | Semicolon | altgr |

**`ñ` (U+00F1)** — exists only on ES at scancode 27 (Semicolon).

(Layer is shown for reference only — the algorithm never reads it.)

### Question selection: scoring with ABSENT bucket

For a given character `c` and a candidate set `C`, compute partition buckets:
- One bucket per distinct `event.code` value among `{ position(c, layout) | layout in C, c is printed on layout }`
- One **`ABSENT` bucket** for layouts in `C` that do NOT print `c`

The best question is the one whose largest bucket is smallest (minimax). Including `ABSENT` is essential: `ñ` is excellent for `{US, ES}` precisely because it splits into `Semicolon → ES` and `ABSENT → US`.

### Iterative narrowing (pseudocode)

```
candidates = [all 6 layout ids]
questionsAsked = 0

while len(candidates) > 1 and questionsAsked < maxQuestions:
    char = pick_best_question(candidates)        # minimax over positions + ABSENT
    expected = { layout: position(char, layout) for layout in candidates if char in layout }
    wrongPresses = 0

    loop:
        response = await user_response()

        if response.kind == "modifier_only":
            continue                              # ignore, wait for the actual key
        if response.kind == "escape":
            return CANCELLED
        if response.kind == "no_such_key":        # button click
            candidates = [c for c in candidates if char not in printed_chars(c)]
            questionsAsked += 1
            break
        if response.kind == "key_pressed":
            if response.event_code in expected.values():
                candidates = [c for c in candidates if expected.get(c) == response.event_code]
                questionsAsked += 1
                break
            else:
                wrongPresses += 1
                show_oops_toast()
                if wrongPresses >= maxWrongPressesPerQuestion:
                    show_extended_help()          # explicit physical-key reminder
                # stay on the same question

if len(candidates) == 1: return candidates[0]
else: return DETECTION_FAILED                     # routes to Select.svelte
```

### Limits

| Constant | Value | Rationale |
|----------|-------|-----------|
| `maxQuestions` | 3 | True worst-case is 2 for the current 6 layouts; 3 leaves a safety margin |
| `maxWrongPressesPerQuestion` | 3 | Each "oops" is a UX retry, not a real attempt; resets when the question advances |

After `maxWrongPressesPerQuestion` errors on the same question, surface an explicit help message (see microcopy below) reminding the user the symbol does NOT need to appear on screen.

`questionsAsked` and `wrongPresses` are tracked separately. Wrong presses never count toward `maxQuestions`.

## UI / UX

### Layout

```
┌──────────────────────────────────────────────────┐
│  [progress bar — questionsAsked / maxQuestions]  │
│                                                  │
│  Appuyez sur la touche où vous voyez ce symbole  │
│                                                  │
│              ┌─────┐                             │
│              │  @  │      ← large character card │
│              └─────┘                             │
│                                                  │
│  Repérez ce symbole sur votre clavier physique,  │
│  puis appuyez sur cette touche. Le symbole       │
│  n'a pas besoin de s'afficher à l'écran.         │
│                                                  │
│  [Je n'ai pas cette touche]    (conditional)     │
│  [Choisir manuellement]        (always visible)  │
│                                                  │
│  [keyboard mockup — neutral, slightly desaturated]│
└──────────────────────────────────────────────────┘
```

When the user presses a key not in the expected set, a transient toast appears above the prompt:
> *Oups, ce n'est peut-être pas la bonne touche. Réessayez.*

After `maxWrongPressesPerQuestion` errors on the same question, replace the toast with a sticky help block:
> *Appuyez simplement sur la touche physique où vous voyez ce symbole. Il n'est pas nécessaire que ce symbole s'affiche à l'écran.*

### Mockup styling during detection

The `mk-body` keyboard mockup stays visible but is rendered **slightly desaturated** (e.g. `opacity: 0.6` or a `filter: saturate(0.6)`) to signal it's a reference, not the active subject. **No expected-position highlight** — that would defeat the purpose by leaking the answer.

### Why no "Option" hint in the prompt

Earlier drafts mentioned "you may need to use Shift or Option". This is removed: it pushes users to chase shortcuts that produce the symbol on screen, when all we need is the physical key. The new microcopy explicitly says the symbol *doesn't need to appear*.

### i18n strings

| Key | EN | FR |
|-----|----|----|
| `detect.charPrompt` | Press the key where you see this symbol | Appuyez sur la touche où vous voyez ce symbole |
| `detect.charHint` | Find this symbol on your physical keyboard, then press that key. The symbol does not need to appear on screen. | Repérez ce symbole sur votre clavier physique, puis appuyez sur cette touche. Le symbole n'a pas besoin de s'afficher à l'écran. |
| `detect.noKey` | I don't have this key | Je n'ai pas cette touche |
| `detect.manual` | Pick manually | Choisir manuellement |
| `detect.wrongKey` | Oops, that might be the wrong key. Try again. | Oups, ce n'est peut-être pas la bonne touche. Réessayez. |
| `detect.wrongKeyHelp` | Just press the physical key where you see this symbol. The symbol does not need to appear on screen. | Appuyez simplement sur la touche physique où vous voyez ce symbole. Il n'est pas nécessaire que ce symbole s'affiche à l'écran. |
| `detect.failedBanner` | Detection failed. Pick your keyboard manually. | Détection impossible. Choisissez votre clavier manuellement. |

## Implementation rules (event handling)

These constraints are part of the design, not optional polish:

1. **Window-level listener** — `window.addEventListener("keydown", ...)`, not bound to an `<input>`. Detection page has no input field.
2. **Ignore `event.repeat`** — only process the first keydown of a press.
3. **Always `preventDefault()`** during the detection step — avoids browser shortcuts firing (Backspace nav, Space scroll, etc.).
4. **Ignore modifier-only keydowns** — `Shift`, `Alt`, `Control`, `Meta`, `Fn`. Wait for the actual key.
5. **`Escape` cancels detection** — returns to Welcome (matching current behavior).
6. **Avoid dead keys as question characters** — dead keys (e.g. `^`, `~`) require a follow-up key to commit; selecting them as questions confuses users. The catalogue generator must filter out characters whose layout entry ends with `@` (the dead-key marker in our JSON format).
7. **"Choisir manuellement" button always visible** — gives users an escape hatch at any point, not only after failure.

## Architecture: build-time catalogue

A catalogue (JSON) is **auto-generated at build time** from `layouts/*.json` and shipped with the frontend. No runtime Tauri call, no manual maintenance.

### Catalogue shape

```jsonc
// src/lib/detection-catalogue.generated.json
{
  "characters": [
    {
      "char": "@",
      "codepoint": "0040",
      "positions": {
        "apple-us-qwerty": "Digit2",
        "apple-uk-qwerty": "Quote",
        "apple-fr-azerty": "Backquote",
        "apple-de-qwertz": "KeyL",
        "apple-es-qwerty": "Digit2",
        "apple-it-qwerty": "Semicolon"
      }
    },
    {
      "char": "ñ",
      "codepoint": "00f1",
      "positions": {
        "apple-es-qwerty": "Semicolon"
      }
    }
    // ...
  ]
}
```

A layout missing from `positions` means the character is ABSENT on it — used directly as the ABSENT bucket in the algorithm.

### Generator

A small Node script (`scripts/build-detection-catalogue.mjs`) runs as a `prebuild` step in `package.json`:

1. Read all `layouts/*.json`.
2. For each layout, walk every key mapping; for every layer (`base`, `shift`, `altgr`, `altgrShift`), collect `(codepoint, scancode)` pairs.
3. Skip dead keys (mappings ending with `@`).
4. Build a per-character set of `(layout, scancode)` pairs.
5. Translate scancode → DOM `event.code` via a static lookup (small table, ~80 entries).
6. Emit `src/lib/detection-catalogue.generated.json`.
7. Optionally curate: keep only characters that distinguish at least 2 layouts (filter out characters present in identical positions across all 6).

### Why build-time over runtime Tauri command

- One source of truth (`layouts/*.json`)
- Zero IPC during the critical UX moment
- Easier to test the algorithm with static fixtures
- Less coupling between frontend and Tauri commands
- Generator runs in CI so catalogue stays in sync with layouts

The existing Tauri commands `get_detection_keys` and `match_detection` become unused. Mark them deprecated; remove in a follow-up if no other consumer surfaces.

## Files to change

| File | Change |
|------|--------|
| `scripts/build-detection-catalogue.mjs` | NEW — generator script (Node, no deps) |
| `src/lib/detection-catalogue.generated.json` | NEW — generated, gitignored, built before Vite |
| `package.json` | Add `prebuild` and `predev` scripts invoking the generator |
| `.gitignore` | Add the generated JSON file |
| `src/lib/detection.ts` | NEW — algorithm: `pickBestQuestion`, `applyResponse`, `printedChars` |
| `src/lib/types.ts` | Add `DetectionChar`, `DetectionResponse`, `DetectionState` |
| `src/pages/Detect.svelte` | Replace position flow with character flow + oops toast + extended help + always-visible "manual" button |
| `src/pages/Select.svelte` | Accept an optional inline banner prop (or read from state) for the "Detection failed" message |
| `src/lib/i18n.ts` | Add the 7 keys above |
| `src/lib/stores.svelte.ts` | Add `detectionFailedMessage?: string` field |
| `src/components/KeyboardVisual.svelte` / `mk-*` styles | No structural change; add a `.kbd-body--neutral` modifier (or equivalent for `mk-body`) that desaturates during detection |

`KeyboardVisual.svelte` and the inline `mk-*` mockup keep their current geometry — only the detection-mode opacity/saturation changes.

## Edge cases

- **Modifier-only keydown** — ignored per rule #4
- **`event.repeat: true`** — ignored per rule #2
- **Dead key as user response** — counts as a normal `event.code`; we already filter dead keys OUT of the catalogue (rule #6), so dead keys can still legitimately be PRESSED by the user as part of finding a non-dead character
- **`Escape`** — cancels detection per rule #5
- **Function/navigation keys during prompt** — treated as wrong-key (oops toast)
- **User pastes via clipboard** — not supported; only keydown events count
- **`maxQuestions` exceeded** — write `appState.detectionFailedMessage`, route to `Select.svelte` which renders an inline banner above the layout list
- **Same physical position across layouts** — e.g. US and ES both have `@` at Digit2 → resolved by the next question (`ñ`)
- **User has no Apple keyboard at all** — "Choisir manuellement" button is always visible (rule #7)

## Out of scope

- Highlighting the expected position on the mockup (deliberately omitted to avoid biasing the answer)
- Visual symbol overlay on the mockup keys
- Detecting custom Windows layouts the user may have installed
- Statistical learning from prior detections to refine question order
- Voice/accessibility cues beyond existing aria labels
- Removing the deprecated Tauri commands (separate cleanup PR)

## Resolved decisions (formerly open questions)

1. **Limits** — `maxQuestions = 3`, `maxWrongPressesPerQuestion = 3`. Two separate counters. ✅
2. **Fallback after failure** — route directly to `Select.svelte` with an inline banner; no intermediate screen. ✅
3. **Mockup during detection** — visible but desaturated; no expected-position highlight. ✅
4. **Catalogue source** — auto-generated at build time from `layouts/*.json`, shipped as static JSON. ✅
