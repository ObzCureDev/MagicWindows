<script lang="ts">
  import type { Layout } from "../lib/types";

  interface Props {
    layout: Layout;
    activeLayer: "base" | "shift" | "altgr" | "altgrShift";
    keyStatus?: Record<string, "untested" | "passed" | "failed">;
  }

  let { layout, activeLayer, keyStatus = {} }: Props = $props();

  // ── ISO vs ANSI detection ─────────────────────────────────────────────
  const isISO = $derived(!!layout.keys["56"]);

  // ── Scancode rows ─────────────────────────────────────────────────────
  // On Apple ISO Magic Keyboards plugged into Windows, scancodes 29 and 56 are
  // physically swapped vs a generic PC ISO board: the top-left "section" key
  // (above Tab, next to 1) sends scancode 56, and the </> key (between LShift
  // and Z/W) sends scancode 29. The visual must mirror the PHYSICAL positions
  // the user sees on their keycaps — so for ISO layouts we render scancode 56
  // at the top-left of the number row, and scancode 29 in the bottom-row
  // 102nd-key slot.
  const numberRow = $derived(
    isISO
      ? ["56", "02", "03", "04", "05", "06", "07", "08", "09", "0a", "0b", "0c", "0d"]
      : ["29", "02", "03", "04", "05", "06", "07", "08", "09", "0a", "0b", "0c", "0d"],
  );
  const topRow    = ["10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "1a", "1b"];
  const homeRow   = ["1e", "1f", "20", "21", "22", "23", "24", "25", "26", "27", "28", "2b"];
  const bottomRow = ["2c", "2d", "2e", "2f", "30", "31", "32", "33", "34", "35"];
  const isoExtraScancode = "29"; // bottom-row 102nd-key slot, after LShift, on Apple ISO

  // ── Character helpers ─────────────────────────────────────────────────
  function hexToChar(hex: string): string {
    if (!hex || hex === "-1") return "";
    const isDead = hex.endsWith("@");
    const clean = isDead ? hex.slice(0, -1) : hex;
    const cp = parseInt(clean, 16);
    if (isNaN(cp) || cp === 0) return "";
    const ch = String.fromCodePoint(cp);
    return isDead ? ch + "\u0332" : ch;
  }

  function getKeyLabel(scancode: string): string {
    const mapping = layout.keys[scancode];
    if (!mapping) return "";
    switch (activeLayer) {
      case "base":      return hexToChar(mapping.base);
      case "shift":     return hexToChar(mapping.shift);
      case "altgr":     return hexToChar(mapping.altgr);
      case "altgrShift":return hexToChar(mapping.altgrShift);
    }
  }

  function getKeyChars(scancode: string): { tl: string; tr: string; bl: string; br: string } {
    const m = layout.keys[scancode];
    if (!m) return { tl: "", tr: "", bl: "", br: "" };
    return {
      tl: hexToChar(m.shift),
      tr: hexToChar(m.altgrShift),
      bl: hexToChar(m.base),
      br: hexToChar(m.altgr),
    };
  }

  function isDifferent(scancode: string): boolean {
    const mapping = layout.keys[scancode];
    if (!mapping) return false;
    if (activeLayer === "altgr") return mapping.altgr !== "-1";
    if (activeLayer === "altgrShift") return mapping.altgrShift !== "-1";
    const val = activeLayer === "base" ? mapping.base : mapping.shift;
    if (!val || val === "-1") return false;
    const clean = val.endsWith("@") ? val.slice(0, -1) : val;
    const cp = parseInt(clean, 16);
    if (isNaN(cp)) return false;
    return cp > 0x7e || val.endsWith("@");
  }

  function isDeadKey(scancode: string): boolean {
    const mapping = layout.keys[scancode];
    if (!mapping) return false;
    const val = (() => {
      switch (activeLayer) {
        case "base":       return mapping.base;
        case "shift":      return mapping.shift;
        case "altgr":      return mapping.altgr;
        case "altgrShift": return mapping.altgrShift;
      }
    })();
    return !!val && val.endsWith("@");
  }

  function getTooltip(scancode: string): string {
    const m = layout.keys[scancode];
    if (!m) return "";
    const entries: string[] = [];
    const addEntry = (label: string, hex: string) => {
      if (!hex || hex === "-1") return;
      const isDead = hex.endsWith("@");
      const clean = isDead ? hex.slice(0, -1) : hex;
      const cp = parseInt(clean, 16);
      if (!isNaN(cp) && cp !== 0) {
        entries.push(`${label}: U+${cp.toString(16).toUpperCase().padStart(4, "0")}${isDead ? " (dead)" : ""}`);
      }
    };
    addEntry("Base", m.base);
    addEntry("Shift", m.shift);
    if (m.altgr && m.altgr !== "-1") addEntry("AltGr", m.altgr);
    if (m.altgrShift && m.altgrShift !== "-1") addEntry("AltGr+Shift", m.altgrShift);
    return entries.join("\n");
  }
</script>

<!-- Outer wrapper handles responsive scaling -->
<div class="kbd-scaler">
  <div class="kbd-body" role="img" aria-label="Keyboard layout preview">

    <!-- ── Row 0: Function row (decorative, dimmed) ───────────────────── -->
    <div class="kbd-row kbd-row--fn">
      <div class="key key--fnkey" title="Escape">
        <span class="fn-text">esc</span>
      </div>

      <!-- F1 — brightness down -->
      <div class="key key--fnkey" title="F1 — Brightness Down">
        <svg class="fn-icon" viewBox="0 0 16 16" aria-hidden="true">
          <circle cx="8" cy="8" r="1.8" fill="none" stroke="currentColor" stroke-width="1.1"/>
          <g stroke="currentColor" stroke-width="1.1" stroke-linecap="round">
            <line x1="8" y1="3.6" x2="8" y2="4.8"/>
            <line x1="8" y1="11.2" x2="8" y2="12.4"/>
            <line x1="3.6" y1="8" x2="4.8" y2="8"/>
            <line x1="11.2" y1="8" x2="12.4" y2="8"/>
            <line x1="4.9" y1="4.9" x2="5.7" y2="5.7"/>
            <line x1="10.3" y1="10.3" x2="11.1" y2="11.1"/>
            <line x1="11.1" y1="4.9" x2="10.3" y2="5.7"/>
            <line x1="5.7" y1="10.3" x2="4.9" y2="11.1"/>
          </g>
        </svg>
        <span class="fn-tag">F1</span>
      </div>

      <!-- F2 — brightness up -->
      <div class="key key--fnkey" title="F2 — Brightness Up">
        <svg class="fn-icon" viewBox="0 0 16 16" aria-hidden="true">
          <circle cx="8" cy="8" r="2.4" fill="none" stroke="currentColor" stroke-width="1.1"/>
          <g stroke="currentColor" stroke-width="1.1" stroke-linecap="round">
            <line x1="8" y1="2.5" x2="8" y2="4.4"/>
            <line x1="8" y1="11.6" x2="8" y2="13.5"/>
            <line x1="2.5" y1="8" x2="4.4" y2="8"/>
            <line x1="11.6" y1="8" x2="13.5" y2="8"/>
            <line x1="4.1" y1="4.1" x2="5.4" y2="5.4"/>
            <line x1="10.6" y1="10.6" x2="11.9" y2="11.9"/>
            <line x1="11.9" y1="4.1" x2="10.6" y2="5.4"/>
            <line x1="5.4" y1="10.6" x2="4.1" y2="11.9"/>
          </g>
        </svg>
        <span class="fn-tag">F2</span>
      </div>

      <!-- F3 — Mission Control -->
      <div class="key key--fnkey" title="F3 — Mission Control">
        <svg class="fn-icon" viewBox="0 0 16 16" aria-hidden="true">
          <g fill="none" stroke="currentColor" stroke-width="1">
            <rect x="2.5" y="3" width="4.5" height="3" rx="0.5"/>
            <rect x="9" y="3" width="4.5" height="3" rx="0.5"/>
            <rect x="2.5" y="8" width="4.5" height="5" rx="0.5"/>
            <rect x="9" y="8" width="4.5" height="5" rx="0.5"/>
          </g>
        </svg>
        <span class="fn-tag">F3</span>
      </div>

      <!-- F4 — Spotlight -->
      <div class="key key--fnkey" title="F4 — Spotlight">
        <svg class="fn-icon" viewBox="0 0 16 16" aria-hidden="true">
          <g fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round">
            <circle cx="6.8" cy="6.8" r="3.4"/>
            <line x1="9.4" y1="9.4" x2="13" y2="13"/>
          </g>
        </svg>
        <span class="fn-tag">F4</span>
      </div>

      <!-- F5 — Dictation -->
      <div class="key key--fnkey" title="F5 — Dictation">
        <svg class="fn-icon" viewBox="0 0 16 16" aria-hidden="true">
          <g fill="none" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" stroke-linejoin="round">
            <rect x="6" y="2.5" width="4" height="6.5" rx="2"/>
            <path d="M4 8 a4 4 0 0 0 8 0"/>
            <line x1="8" y1="12" x2="8" y2="13.5"/>
            <line x1="6.5" y1="13.5" x2="9.5" y2="13.5"/>
          </g>
        </svg>
        <span class="fn-tag">F5</span>
      </div>

      <!-- F6 — Do Not Disturb -->
      <div class="key key--fnkey" title="F6 — Do Not Disturb">
        <svg class="fn-icon" viewBox="0 0 16 16" aria-hidden="true">
          <path d="M12 10.5 A5 5 0 1 1 6.2 3 a4 4 0 0 0 5.8 7.5 z"
                fill="none" stroke="currentColor" stroke-width="1.1" stroke-linejoin="round"/>
        </svg>
        <span class="fn-tag">F6</span>
      </div>

      <!-- F7 — Previous track -->
      <div class="key key--fnkey" title="F7 — Previous">
        <svg class="fn-icon" viewBox="0 0 16 16" aria-hidden="true">
          <g fill="currentColor">
            <rect x="2.5" y="4.5" width="1.3" height="7" rx="0.3"/>
            <polygon points="13.5,4.5 13.5,11.5 8.5,8"/>
            <polygon points="8.5,4.5 8.5,11.5 4,8"/>
          </g>
        </svg>
        <span class="fn-tag">F7</span>
      </div>

      <!-- F8 — Play/Pause -->
      <div class="key key--fnkey" title="F8 — Play / Pause">
        <svg class="fn-icon" viewBox="0 0 16 16" aria-hidden="true">
          <g fill="currentColor">
            <polygon points="2.5,3.5 2.5,12.5 8,8"/>
            <rect x="9.5" y="3.5" width="1.5" height="9" rx="0.3"/>
            <rect x="12" y="3.5" width="1.5" height="9" rx="0.3"/>
          </g>
        </svg>
        <span class="fn-tag">F8</span>
      </div>

      <!-- F9 — Next track -->
      <div class="key key--fnkey" title="F9 — Next">
        <svg class="fn-icon" viewBox="0 0 16 16" aria-hidden="true">
          <g fill="currentColor">
            <polygon points="2.5,4.5 2.5,11.5 7.5,8"/>
            <polygon points="7.5,4.5 7.5,11.5 12,8"/>
            <rect x="12.2" y="4.5" width="1.3" height="7" rx="0.3"/>
          </g>
        </svg>
        <span class="fn-tag">F9</span>
      </div>

      <!-- F10 — Mute -->
      <div class="key key--fnkey" title="F10 — Mute">
        <svg class="fn-icon" viewBox="0 0 16 16" aria-hidden="true">
          <polygon points="2,6.2 5,6.2 8.5,3.5 8.5,12.5 5,9.8 2,9.8" fill="currentColor"/>
          <g fill="none" stroke="currentColor" stroke-width="1.1" stroke-linecap="round">
            <line x1="11" y1="6" x2="14" y2="10"/>
            <line x1="14" y1="6" x2="11" y2="10"/>
          </g>
        </svg>
        <span class="fn-tag">F10</span>
      </div>

      <!-- F11 — Volume Down -->
      <div class="key key--fnkey" title="F11 — Volume Down">
        <svg class="fn-icon" viewBox="0 0 16 16" aria-hidden="true">
          <polygon points="2,6.2 5,6.2 8.5,3.5 8.5,12.5 5,9.8 2,9.8" fill="currentColor"/>
          <path d="M10.8 6.2 a3 3 0 0 1 0 3.6"
                fill="none" stroke="currentColor" stroke-width="1.1" stroke-linecap="round"/>
        </svg>
        <span class="fn-tag">F11</span>
      </div>

      <!-- F12 — Volume Up -->
      <div class="key key--fnkey" title="F12 — Volume Up">
        <svg class="fn-icon" viewBox="0 0 16 16" aria-hidden="true">
          <polygon points="2,6.2 5,6.2 8.5,3.5 8.5,12.5 5,9.8 2,9.8" fill="currentColor"/>
          <g fill="none" stroke="currentColor" stroke-width="1.1" stroke-linecap="round">
            <path d="M10.8 6.2 a3 3 0 0 1 0 3.6"/>
            <path d="M12.6 4.5 a5 5 0 0 1 0 7"/>
          </g>
        </svg>
        <span class="fn-tag">F12</span>
      </div>

      <!-- Touch ID -->
      <div class="key key--touchid" title="Touch ID" aria-label="Touch ID"></div>
    </div>

    <!-- ── Row 1: Number row + Backspace ─────────────────────────────── -->
    <div class="kbd-row">
      {#each numberRow as sc}
        {@const chars = getKeyChars(sc)}
        <div
          class={`key key--char status-${keyStatus[sc] ?? "untested"}`}
          class:key--different={isDifferent(sc)}
          class:key--dead={isDeadKey(sc)}
          title={getTooltip(sc)}
          aria-label={getKeyLabel(sc)}
        >
          <span class="key__tr">{chars.tr}</span>
          <span class="key__tl">{chars.tl}</span>
          <span class="key__br">{chars.br}</span>
          <span class="key__bl">{chars.bl}</span>
        </div>
      {/each}
      <div class="key key--backspace key--mod" title="Delete / Backspace">
        <span class="key__mod-label">delete</span>
      </div>
    </div>

    <!-- ── Row 2: Tab + letter row + (ISO: part of Enter) ────────────── -->
    <div class="kbd-row">
      <div class="key key--tab key--mod" title="Tab">
        <span class="key__mod-label">tab</span>
      </div>
      {#each topRow as sc}
        {@const chars = getKeyChars(sc)}
        <div
          class={`key key--char status-${keyStatus[sc] ?? "untested"}`}
          class:key--different={isDifferent(sc)}
          class:key--dead={isDeadKey(sc)}
          title={getTooltip(sc)}
          aria-label={getKeyLabel(sc)}
        >
          <span class="key__tr">{chars.tr}</span>
          <span class="key__tl">{chars.tl}</span>
          <span class="key__br">{chars.br}</span>
          <span class="key__bl">{chars.bl}</span>
        </div>
      {/each}
      {#if isISO}
        <div class="key key--enter-iso-top key--mod" aria-hidden="true"></div>
      {/if}
    </div>

    <!-- ── Row 3: Caps Lock + home row + Enter ───────────────────────── -->
    <div class="kbd-row">
      <div class="key key--caps key--mod" title="Caps Lock">
        <span class="key__mod-label">caps lock</span>
      </div>
      {#each homeRow as sc}
        {@const chars = getKeyChars(sc)}
        <div
          class={`key key--char status-${keyStatus[sc] ?? "untested"}`}
          class:key--different={isDifferent(sc)}
          class:key--dead={isDeadKey(sc)}
          title={getTooltip(sc)}
          aria-label={getKeyLabel(sc)}
        >
          <span class="key__tr">{chars.tr}</span>
          <span class="key__tl">{chars.tl}</span>
          <span class="key__br">{chars.br}</span>
          <span class="key__bl">{chars.bl}</span>
        </div>
      {/each}
      {#if isISO}
        <div class="key key--enter-iso-bottom key--mod" title="Return">
          <span class="key__mod-label">return</span>
        </div>
      {:else}
        <div class="key key--enter-ansi key--mod" title="Return">
          <span class="key__mod-label">return</span>
        </div>
      {/if}
    </div>

    <!-- ── Row 4: Shift + bottom row + Right Shift ───────────────────── -->
    <div class="kbd-row">
      {#if isISO}
        <div class="key key--lshift-iso key--mod" title="Shift">
          <span class="key__mod-label">shift</span>
        </div>
        {@const isoChars = getKeyChars(isoExtraScancode)}
        <div
          class={`key key--char key--iso-extra status-${keyStatus[isoExtraScancode] ?? "untested"}`}
          class:key--different={isDifferent(isoExtraScancode)}
          class:key--dead={isDeadKey(isoExtraScancode)}
          title={getTooltip(isoExtraScancode)}
          aria-label={getKeyLabel(isoExtraScancode)}
        >
          <span class="key__tr">{isoChars.tr}</span>
          <span class="key__tl">{isoChars.tl}</span>
          <span class="key__br">{isoChars.br}</span>
          <span class="key__bl">{isoChars.bl}</span>
        </div>
      {:else}
        <div class="key key--lshift-ansi key--mod" title="Shift">
          <span class="key__mod-label">shift</span>
        </div>
      {/if}
      {#each bottomRow as sc}
        {@const chars = getKeyChars(sc)}
        <div
          class={`key key--char status-${keyStatus[sc] ?? "untested"}`}
          class:key--different={isDifferent(sc)}
          class:key--dead={isDeadKey(sc)}
          title={getTooltip(sc)}
          aria-label={getKeyLabel(sc)}
        >
          <span class="key__tr">{chars.tr}</span>
          <span class="key__tl">{chars.tl}</span>
          <span class="key__br">{chars.br}</span>
          <span class="key__bl">{chars.bl}</span>
        </div>
      {/each}
      <div class="key key--rshift key--mod" title="Shift">
        <span class="key__mod-label">shift</span>
      </div>
    </div>

    <!-- ── Row 5: Modifiers + Space ──────────────────────────────────── -->
    <div class="kbd-row kbd-row--space">
      <div class="key key--fn key--mod" title="Globe / Function">
        <svg class="key__globe" viewBox="0 0 16 16" aria-hidden="true">
          <g fill="none" stroke="currentColor" stroke-width="1.1">
            <circle cx="8" cy="8" r="5.3"/>
            <ellipse cx="8" cy="8" rx="2.1" ry="5.3"/>
            <line x1="2.7" y1="8" x2="13.3" y2="8"/>
            <path d="M3.3 5 Q8 6.5 12.7 5"/>
            <path d="M3.3 11 Q8 9.5 12.7 11"/>
          </g>
        </svg>
      </div>
      <div class="key key--ctrl key--mod">
        <span class="key__mod-glyph">⌃</span>
        <span class="key__mod-label">control</span>
      </div>
      <div class="key key--alt key--mod">
        <span class="key__mod-glyph">⌥</span>
        <span class="key__mod-label">option</span>
      </div>
      <div class="key key--cmd key--mod key--cmd-left">
        <span class="key__mod-glyph">⌘</span>
        <span class="key__mod-label">command</span>
      </div>
      <div class="key key--space key--mod" title="Space">
        <span class="key__mod-label sr-only">space</span>
      </div>
      <div class="key key--cmd key--mod key--cmd-right">
        <span class="key__mod-glyph">⌘</span>
        <span class="key__mod-label">command</span>
      </div>
      <div class="key key--alt key--mod">
        <span class="key__mod-glyph">⌥</span>
        <span class="key__mod-label">option</span>
      </div>
      <!-- Arrow cluster (inverted T) -->
      <div class="kbd-arrows">
        <div class="key key--arrow key--mod" aria-label="Up Arrow">▲</div>
        <div class="key key--arrow key--mod" aria-label="Left Arrow">◀</div>
        <div class="key key--arrow key--mod" aria-label="Down Arrow">▼</div>
        <div class="key key--arrow key--mod" aria-label="Right Arrow">▶</div>
      </div>
    </div>

  </div><!-- /kbd-body -->
</div><!-- /kbd-scaler -->

<style>
  /* ═══════════════════════════════════════════════════════════════════════
     APPLE MAGIC KEYBOARD — visual replica
     Light mode: white aluminum shell, soft printed-on keys
     Dark mode:  Space Black / dark grey tones
     ═══════════════════════════════════════════════════════════════════════ */

  .kbd-body {
    /* Light defaults — matches the 2021+ Magic Keyboard photo */
    --kbd-shell:        #e8e8ec;
    --kbd-shell-edge:   #c5c5cb;
    --kbd-shell-shadow:
      0 1px 0 rgba(255,255,255,0.7) inset,
      0 -1px 0 rgba(0,0,0,0.04) inset,
      0 6px 18px rgba(0,0,0,0.10),
      0 1px 3px rgba(0,0,0,0.07);

    --key-bg:           #fbfbfd;
    --key-bg-top:       #ffffff;
    --key-border:       rgba(0,0,0,0.08);
    --key-shadow:
      0 0.5px 0 rgba(255,255,255,0.95) inset,
      0 1px 1.5px rgba(0,0,0,0.07);
    --key-text:         #2a2a2c;
    --key-mod-text:     #6e6e73;
    --key-mod-size:     8.5px;
    --key-glyph-size:   12px;
    --key-hover-bg:     #f3f3f7;

    --key-diff-border:  var(--color-key-different, #e67e00);
    --key-diff-bg:      var(--color-key-different-bg, rgba(230,126,0,0.12));
    --key-diff-text:    var(--color-key-different, #e67e00);
    --key-dead-underline: rgba(0,0,0,0.45);

    --touchid-bg:       linear-gradient(180deg, #ffffff 0%, #ececf0 100%);

    /* Base unit and rhythm */
    --u: 40px;
    --gap: 5px;
    --radius-key: 8px;
    --radius-body: 18px;
  }

  @media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) .kbd-body {
      --kbd-shell:        #2a2a2c;
      --kbd-shell-edge:   #1a1a1c;
      --kbd-shell-shadow:
        0 1px 0 rgba(255,255,255,0.05) inset,
        0 -1px 0 rgba(0,0,0,0.4) inset,
        0 6px 22px rgba(0,0,0,0.55),
        0 1px 3px rgba(0,0,0,0.35);

      --key-bg:           #4a4a4c;
      --key-bg-top:       #5a5a5e;
      --key-border:       rgba(0,0,0,0.45);
      --key-shadow:
        0 0.5px 0 rgba(255,255,255,0.08) inset,
        0 1px 1.5px rgba(0,0,0,0.45);
      --key-text:         #f5f5f7;
      --key-mod-text:     #98989d;
      --key-hover-bg:     #5a5a5e;
      --key-dead-underline: rgba(255,255,255,0.4);
      --touchid-bg:       linear-gradient(180deg, #5a5a5e 0%, #444446 100%);
    }
  }

  :root[data-theme="dark"] .kbd-body {
    --kbd-shell:        #2a2a2c;
    --kbd-shell-edge:   #1a1a1c;
    --kbd-shell-shadow:
      0 1px 0 rgba(255,255,255,0.05) inset,
      0 -1px 0 rgba(0,0,0,0.4) inset,
      0 6px 22px rgba(0,0,0,0.55),
      0 1px 3px rgba(0,0,0,0.35);

    --key-bg:           #4a4a4c;
    --key-bg-top:       #5a5a5e;
    --key-border:       rgba(0,0,0,0.45);
    --key-shadow:
      0 0.5px 0 rgba(255,255,255,0.08) inset,
      0 1px 1.5px rgba(0,0,0,0.45);
    --key-text:         #f5f5f7;
    --key-mod-text:     #98989d;
    --key-hover-bg:     #5a5a5e;
    --key-dead-underline: rgba(255,255,255,0.4);
    --touchid-bg:       linear-gradient(180deg, #5a5a5e 0%, #444446 100%);
  }

  /* ── Responsive scaling wrapper ──────────────────────────────────────── */
  .kbd-scaler {
    width: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    overflow-x: auto;
    padding: 8px 0;
  }

  /* ── Keyboard body (the aluminum slab) ───────────────────────────────── */
  .kbd-body {
    display: inline-flex;
    flex-direction: column;
    gap: var(--gap);
    padding: 10px 11px 12px;
    background: var(--kbd-shell);
    border: 1px solid var(--kbd-shell-edge);
    border-radius: var(--radius-body);
    box-shadow: var(--kbd-shell-shadow);
    flex-shrink: 0;
    background-image: linear-gradient(
      178deg,
      color-mix(in srgb, var(--kbd-shell) 80%, white 20%) 0%,
      var(--kbd-shell) 50%,
      color-mix(in srgb, var(--kbd-shell) 92%, black 8%) 100%
    );
  }

  /* ── Rows ────────────────────────────────────────────────────────────── */
  .kbd-row {
    display: flex;
    flex-direction: row;
    gap: var(--gap);
    align-items: flex-end;
  }

  .kbd-row--space {
    align-items: center;
  }

  .kbd-row--fn {
    align-items: center;
    margin-bottom: 3px;
  }

  /* ── Base key ────────────────────────────────────────────────────────── */
  .key {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    height: var(--u);
    min-width: var(--u);
    border-radius: var(--radius-key);
    background: linear-gradient(180deg, var(--key-bg-top) 0%, var(--key-bg) 100%);
    border: 1px solid var(--key-border);
    box-shadow: var(--key-shadow);
    color: var(--key-text);
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", Helvetica, Arial, sans-serif;
    font-size: 13px;
    font-weight: 400;
    cursor: default;
    user-select: none;
    flex-shrink: 0;
    transition: filter 100ms ease, transform 100ms ease;
  }

  .key:hover {
    filter: brightness(1.04);
    transform: translateY(-0.5px);
  }

  /* ── Character key quadrant layout ───────────────────────────────────── */
  .key--char {
    font-size: 12.5px;
  }

  .key__tl, .key__tr, .key__bl, .key__br {
    position: absolute;
    line-height: 1;
    font-size: 10.5px;
    font-weight: 400;
    color: var(--key-text);
  }

  .key__tl { top: 5px;    left: 6px; }
  .key__tr { top: 5px;    right: 6px; }
  .key__bl { bottom: 5px; left: 6px; }
  .key__br { bottom: 5px; right: 6px; }

  /* ── Modifier key shared style ───────────────────────────────────────── */
  .key--mod {
    background: linear-gradient(
      180deg,
      color-mix(in srgb, var(--key-bg-top) 92%, var(--kbd-shell) 8%) 0%,
      color-mix(in srgb, var(--key-bg) 88%, var(--kbd-shell) 12%) 100%
    );
  }

  .key__mod-label {
    font-size: var(--key-mod-size);
    font-weight: 400;
    color: var(--key-mod-text);
    letter-spacing: 0.01em;
    white-space: nowrap;
    text-align: center;
    padding: 0 4px;
  }

  .key__mod-glyph {
    font-size: var(--key-glyph-size);
    line-height: 1;
    color: var(--key-text);
    font-weight: 400;
  }

  /* Modifiers with both glyph + label stack vertically */
  .key--ctrl, .key--alt, .key--cmd {
    flex-direction: column;
    gap: 1px;
    padding-top: 2px;
  }

  .key--ctrl .key__mod-label,
  .key--alt .key__mod-label,
  .key--cmd .key__mod-label {
    font-size: 7.5px;
  }

  /* ── Globe (fn) key ──────────────────────────────────────────────────── */
  .key__globe {
    width: 14px;
    height: 14px;
    color: var(--key-text);
  }

  /* ── Function row (decorative, dimmed) ───────────────────────────────── */
  .key--fnkey {
    flex: 1 1 0;
    min-width: 0;
    width: auto;
    height: calc(var(--u) * 0.62);
    flex-direction: column;
    gap: 1px;
    padding: 3px 0 2px;
    opacity: 0.62;
  }

  .key--fnkey:hover {
    opacity: 0.85;
  }

  .fn-icon {
    width: 13px;
    height: 13px;
    color: var(--key-text);
    display: block;
  }

  .fn-tag {
    font-size: 6.5px;
    color: var(--key-mod-text);
    letter-spacing: 0.04em;
    line-height: 1;
    font-weight: 500;
  }

  .fn-text {
    font-size: 9px;
    color: var(--key-mod-text);
    text-transform: lowercase;
    letter-spacing: 0.01em;
    line-height: 1;
  }

  .key--touchid {
    flex: 0 0 auto;
    width: calc(var(--u) * 0.62);
    height: calc(var(--u) * 0.62);
    min-width: 0;
    border-radius: 50%;
    background: var(--touchid-bg);
    border: 1px solid var(--key-border);
    box-shadow:
      0 0.5px 0 rgba(255,255,255,0.95) inset,
      0 1px 1.5px rgba(0,0,0,0.07),
      0 0 0 1px rgba(255,255,255,0.04) inset;
    opacity: 0.78;
    position: relative;
  }

  .key--touchid::after {
    content: "";
    position: absolute;
    inset: 22%;
    border-radius: 50%;
    border: 0.5px solid rgba(0,0,0,0.06);
    background: radial-gradient(circle at 50% 40%, transparent 60%, rgba(0,0,0,0.04) 100%);
  }

  :root[data-theme="dark"] .key--touchid::after,
  :root:not([data-theme="light"]) .key--touchid::after {
    border-color: rgba(255,255,255,0.06);
    background: radial-gradient(circle at 50% 40%, transparent 60%, rgba(0,0,0,0.25) 100%);
  }

  /* ── Highlighted (different from Windows default) ────────────────────── */
  .key--different {
    border-color: var(--key-diff-border) !important;
    background: linear-gradient(
      180deg,
      color-mix(in srgb, var(--key-bg-top) 80%, var(--key-diff-border) 20%) 0%,
      color-mix(in srgb, var(--key-bg) 75%, var(--key-diff-border) 25%) 100%
    ) !important;
    box-shadow:
      0 0.5px 0 rgba(255,255,255,0.85) inset,
      0 1px 1.5px rgba(0,0,0,0.08),
      0 0 0 1px color-mix(in srgb, var(--key-diff-border) 30%, transparent 70%) !important;
  }

  .key--different .key__tl,
  .key--different .key__tr,
  .key--different .key__bl,
  .key--different .key__br {
    color: var(--key-diff-text);
    font-weight: 600;
  }

  /* ── Per-key status overlay (Health Check) ───────────────────────────── */
  /* .status-untested is the default — no extra ring (intentionally no rule). */
  .key.status-passed { box-shadow: 0 0 0 2px var(--color-success, #34d399) inset; }
  .key.status-failed { box-shadow: 0 0 0 2px var(--color-danger, #f87171) inset; }

  /* ── Dead key indicator ──────────────────────────────────────────────── */
  .key--dead::after {
    content: "";
    position: absolute;
    bottom: 2px;
    left: 22%;
    right: 22%;
    height: 2px;
    border-radius: 1px;
    background: var(--key-dead-underline);
  }

  /* ── Accessibility ───────────────────────────────────────────────────── */
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0,0,0,0);
    white-space: nowrap;
  }

  /* ═══════════════════════════════════════════════════════════════════════
     KEY WIDTHS — calc(N * var(--u) + (N-1) * var(--gap))
     ═══════════════════════════════════════════════════════════════════════ */

  .key--backspace {
    width: calc(1.5 * var(--u) + 0.5 * var(--gap));
  }

  .key--tab {
    width: calc(1.5 * var(--u) + 0.5 * var(--gap));
    justify-content: flex-start;
    padding-left: 7px;
  }

  .key--caps {
    width: calc(1.75 * var(--u) + 0.75 * var(--gap));
    justify-content: flex-start;
    padding-left: 7px;
  }

  .key--caps::after {
    content: "";
    position: absolute;
    right: 8px;
    top: 50%;
    transform: translateY(-50%);
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: rgba(0,0,0,0.18);
    box-shadow: inset 0 1px 1.5px rgba(0,0,0,0.25);
  }

  /* ISO Enter — top stub */
  .key--enter-iso-top {
    width: calc(1.25 * var(--u) + 0.25 * var(--gap));
    border-bottom-left-radius: 0;
    border-bottom-right-radius: 0;
    border-bottom: none;
    height: calc(var(--u) + 1px);
    box-shadow:
      0 0.5px 0 rgba(255,255,255,0.95) inset,
      1px 0 0 rgba(0,0,0,0.08),
      -1px 0 0 rgba(0,0,0,0.08);
    z-index: 1;
  }

  .key--enter-iso-bottom {
    width: calc(1.75 * var(--u) + 0.75 * var(--gap));
    border-top-right-radius: 0;
    justify-content: flex-start;
    padding-left: 9px;
  }

  .key--enter-ansi {
    width: calc(2.25 * var(--u) + 1.25 * var(--gap));
    justify-content: flex-start;
    padding-left: 9px;
  }

  .key--lshift-iso {
    width: calc(1.25 * var(--u) + 0.25 * var(--gap));
    justify-content: flex-start;
    padding-left: 7px;
  }

  .key--iso-extra {
    min-width: var(--u);
    width: var(--u);
  }

  .key--lshift-ansi {
    width: calc(2.25 * var(--u) + 1.25 * var(--gap));
    justify-content: flex-start;
    padding-left: 7px;
  }

  .key--rshift {
    width: calc(2.75 * var(--u) + 1.75 * var(--gap));
    justify-content: flex-end;
    padding-right: 7px;
  }

  .key--fn {
    width: calc(1.25 * var(--u) + 0.25 * var(--gap));
  }

  .key--ctrl {
    width: calc(1.25 * var(--u) + 0.25 * var(--gap));
  }

  .key--alt {
    width: calc(1.25 * var(--u) + 0.25 * var(--gap));
  }

  .key--cmd {
    width: calc(1.5 * var(--u) + 0.5 * var(--gap));
  }

  .key--space {
    flex: 1;
    min-width: calc(6.25 * var(--u) + 5.25 * var(--gap));
  }

  /* ── Arrow cluster (inverted T) ──────────────────────────────────────── */
  .kbd-arrows {
    display: grid;
    grid-template-columns: var(--u) var(--u) var(--u);
    grid-template-rows: calc(var(--u) * 0.5) calc(var(--u) * 0.5);
    gap: 2px;
  }

  .key--arrow {
    font-size: 8px;
    min-width: unset;
    width: var(--u);
    height: calc(var(--u) * 0.5);
    padding: 0;
    color: var(--key-mod-text);
  }

  .key--arrow:nth-child(1) { grid-column: 2; grid-row: 1; }
  .key--arrow:nth-child(2) { grid-column: 1; grid-row: 2; }
  .key--arrow:nth-child(3) { grid-column: 2; grid-row: 2; }
  .key--arrow:nth-child(4) { grid-column: 3; grid-row: 2; }
</style>
