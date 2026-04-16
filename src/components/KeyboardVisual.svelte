<script lang="ts">
  import type { Layout } from "../lib/types";

  interface Props {
    layout: Layout;
    activeLayer: "base" | "shift" | "altgr" | "altgrShift";
  }

  let { layout, activeLayer }: Props = $props();

  // ── ISO vs ANSI detection ─────────────────────────────────────────────
  const isISO = $derived(!!layout.keys["56"]);

  // ── Scancode rows ─────────────────────────────────────────────────────
  const numberRow = ["29", "02", "03", "04", "05", "06", "07", "08", "09", "0a", "0b", "0c", "0d"];
  const topRow    = ["10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "1a", "1b"];
  const homeRow   = ["1e", "1f", "20", "21", "22", "23", "24", "25", "26", "27", "28", "2b"];
  // Bottom row keys are the same for ISO and ANSI; layout differences are
  // handled by the Left Shift width and the extra ISO key (scancode 56).
  const bottomRow = ["2c", "2d", "2e", "2f", "30", "31", "32", "33", "34", "35"];

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

  // Returns all four corner characters for a key: [topLeft, topRight, bottomLeft, bottomRight]
  // topLeft = shift, topRight = altgrShift, bottomLeft = base, bottomRight = altgr
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

  // Tooltip: show Unicode codepoints for all active characters
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

    <!-- ── Row 1: Number row + Backspace ─────────────────────────────── -->
    <div class="kbd-row">
      {#each numberRow as sc}
        {@const chars = getKeyChars(sc)}
        <div
          class="key key--char"
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
          class="key key--char"
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
        <!-- ISO Enter top stub — rendered as part of L-shape via grid -->
        <div class="key key--enter-iso-top key--mod" aria-hidden="true"></div>
      {:else}
        <!-- ANSI: no extra key here; Enter is on row 3 -->
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
          class="key key--char"
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
        <!-- Extra ISO key between Left Shift and Z -->
        {@const isoChars = getKeyChars("56")}
        <div
          class="key key--char key--iso-extra"
          class:key--different={isDifferent("56")}
          class:key--dead={isDeadKey("56")}
          title={getTooltip("56")}
          aria-label={getKeyLabel("56")}
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
          class="key key--char"
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
      <div class="key key--fn key--mod"><span class="key__mod-label">fn</span></div>
      <div class="key key--ctrl key--mod"><span class="key__mod-label">control</span></div>
      <div class="key key--alt key--mod"><span class="key__mod-label">option</span></div>
      <div class="key key--cmd key--mod key--cmd-left">
        <span class="key__cmd-icon" aria-hidden="true">⌘</span>
        <span class="key__mod-label">command</span>
      </div>
      <div class="key key--space key--mod" title="Space">
        <span class="key__mod-label sr-only">space</span>
      </div>
      <div class="key key--cmd key--mod key--cmd-right">
        <span class="key__cmd-icon" aria-hidden="true">⌘</span>
        <span class="key__mod-label">command</span>
      </div>
      <div class="key key--alt key--mod"><span class="key__mod-label">option</span></div>
      <!-- Arrow cluster -->
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
  /* ── Design tokens (local, theme-aware) ─────────────────────────────── */
  .kbd-body {
    /* Light-mode defaults — Apple Magic Keyboard silver */
    --kbd-shell:        #c8c8cc;
    --kbd-shell-border: #a8a8b0;
    --kbd-shell-shadow: 0 2px 0 #909098, 0 6px 24px rgba(0,0,0,0.18), 0 2px 6px rgba(0,0,0,0.12);

    --key-bg:           #f0f0f4;
    --key-bg-top:       #ffffff;
    --key-border:       rgba(0,0,0,0.14);
    --key-shadow:       0 2px 0 rgba(0,0,0,0.22), 0 1px 0 rgba(0,0,0,0.1);
    --key-text:         #1d1d1f;
    --key-mod-text:     #3a3a3c;
    --key-mod-size:     9px;
    --key-hover-bg:     #e2e2e8;

    --key-diff-border:  var(--color-key-different, #e67e00);
    --key-diff-bg:      var(--color-key-different-bg, rgba(230,126,0,0.12));
    --key-diff-text:    var(--color-key-different, #e67e00);
    --key-dead-underline: rgba(0,0,0,0.4);

    /* Base unit — all sizes derived from this */
    --u: 40px;
    --gap: 5px;
    --radius-key: 5px;
    --radius-body: 14px;
  }

  /* Dark mode overrides */
  @media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) .kbd-body {
      --kbd-shell:        #3a3a3c;
      --kbd-shell-border: #2c2c2e;
      --kbd-shell-shadow: 0 2px 0 #1c1c1e, 0 6px 24px rgba(0,0,0,0.55), 0 2px 6px rgba(0,0,0,0.3);

      --key-bg:           #48484a;
      --key-bg-top:       #5a5a5e;
      --key-border:       rgba(0,0,0,0.5);
      --key-shadow:       0 2px 0 rgba(0,0,0,0.6), 0 1px 0 rgba(0,0,0,0.35);
      --key-text:         #f5f5f7;
      --key-mod-text:     #aeaeb2;
      --key-hover-bg:     #58585c;
      --key-dead-underline: rgba(255,255,255,0.4);
    }
  }

  :root[data-theme="dark"] .kbd-body {
    --kbd-shell:        #3a3a3c;
    --kbd-shell-border: #2c2c2e;
    --kbd-shell-shadow: 0 2px 0 #1c1c1e, 0 6px 24px rgba(0,0,0,0.55), 0 2px 6px rgba(0,0,0,0.3);

    --key-bg:           #48484a;
    --key-bg-top:       #5a5a5e;
    --key-border:       rgba(0,0,0,0.5);
    --key-shadow:       0 2px 0 rgba(0,0,0,0.6), 0 1px 0 rgba(0,0,0,0.35);
    --key-text:         #f5f5f7;
    --key-mod-text:     #aeaeb2;
    --key-hover-bg:     #58585c;
    --key-dead-underline: rgba(255,255,255,0.4);
  }

  /* ── Responsive scaling wrapper ──────────────────────────────────────── */
  .kbd-scaler {
    width: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    /* Let the keyboard shrink gracefully on narrow viewports */
    overflow-x: auto;
    padding: 8px 0;
  }

  /* ── Keyboard body (the aluminum slab) ───────────────────────────────── */
  .kbd-body {
    display: inline-flex;
    flex-direction: column;
    gap: var(--gap);
    padding: 14px 14px 16px;
    background: var(--kbd-shell);
    border: 1px solid var(--kbd-shell-border);
    border-radius: var(--radius-body);
    box-shadow: var(--kbd-shell-shadow);
    flex-shrink: 0;
    /* Subtle brushed-metal look via gradient */
    background-image: linear-gradient(
      175deg,
      color-mix(in srgb, var(--kbd-shell) 70%, white 30%) 0%,
      var(--kbd-shell) 40%,
      color-mix(in srgb, var(--kbd-shell) 90%, black 10%) 100%
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

  /* ── Base key ────────────────────────────────────────────────────────── */
  .key {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    height: var(--u);
    min-width: var(--u);
    border-radius: var(--radius-key);
    background:
      linear-gradient(
        180deg,
        var(--key-bg-top) 0%,
        var(--key-bg) 60%
      );
    border: 1px solid var(--key-border);
    box-shadow: var(--key-shadow);
    color: var(--key-text);
    font-family: -apple-system, BlinkMacSystemFont, "Helvetica Neue", Helvetica, Arial, sans-serif;
    font-size: 14px;
    cursor: default;
    user-select: none;
    flex-shrink: 0;
    transition: filter 100ms ease, transform 100ms ease;
    /* Inset highlight on top edge for 3D bevel feel */
    outline: 1px solid rgba(255,255,255,0.35);
    outline-offset: -2px;
  }

  .key:hover {
    filter: brightness(1.06);
    transform: translateY(-1px);
  }

  /* ── Character key quadrant layout ───────────────────────────────────── */
  .key--char {
    font-size: 13px;
  }

  /* All four label spans are absolutely positioned in their corner */
  .key__tl,
  .key__tr,
  .key__bl,
  .key__br {
    position: absolute;
    line-height: 1;
    font-size: 11px;
    font-weight: 500;
  }

  .key__tl { top: 4px;    left: 5px; }
  .key__tr { top: 4px;    right: 5px; }
  .key__bl { bottom: 4px; left: 5px; }
  .key__br { bottom: 4px; right: 5px; }

  /* ── Modifier key shared style ───────────────────────────────────────── */
  .key--mod {
    background:
      linear-gradient(
        180deg,
        color-mix(in srgb, var(--key-bg-top) 90%, var(--kbd-shell) 10%) 0%,
        color-mix(in srgb, var(--key-bg) 85%, var(--kbd-shell) 15%) 100%
      );
  }

  .key__mod-label {
    font-size: var(--key-mod-size);
    font-weight: 500;
    color: var(--key-mod-text);
    letter-spacing: 0.01em;
    white-space: nowrap;
    text-align: center;
    padding: 0 4px;
  }

  /* ── Command key icon ────────────────────────────────────────────────── */
  .key--cmd {
    flex-direction: column;
    gap: 1px;
  }

  .key__cmd-icon {
    font-size: 15px;
    line-height: 1;
    color: var(--key-text);
  }

  .key--cmd .key__mod-label {
    font-size: 8px;
  }

  /* ── Highlighted (different from Windows default) ────────────────────── */
  .key--different {
    border-color: var(--key-diff-border) !important;
    background:
      linear-gradient(
        180deg,
        color-mix(in srgb, var(--key-bg-top) 80%, var(--key-diff-border) 20%) 0%,
        color-mix(in srgb, var(--key-bg) 75%, var(--key-diff-border) 25%) 100%
      ) !important;
    outline-color: color-mix(in srgb, var(--key-diff-border) 40%, transparent 60%) !important;
  }

  .key--different .key__tl,
  .key--different .key__tr,
  .key--different .key__bl,
  .key--different .key__br {
    color: var(--key-diff-text);
    font-weight: 600;
  }

  /* ── Dead key indicator ──────────────────────────────────────────────── */
  .key--dead::after {
    content: "";
    position: absolute;
    bottom: 2px;
    left: 20%;
    right: 20%;
    height: 2px;
    border-radius: 1px;
    background: var(--key-dead-underline);
  }

  /* ── Accessibility: visually hidden but screen-reader accessible ──────── */
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0,0,0,0);
    white-space: nowrap;
  }

  /* ═══════════════════════════════════════════════════════════════════════
     KEY WIDTHS
     Using calc(N * var(--u) + (N-1) * var(--gap)) for multi-unit keys.
     1u = var(--u)
     ═══════════════════════════════════════════════════════════════════════ */

  /* Backspace / Delete — 1.5u */
  .key--backspace {
    width: calc(1.5 * var(--u) + 0.5 * var(--gap));
  }

  /* Tab — 1.5u */
  .key--tab {
    width: calc(1.5 * var(--u) + 0.5 * var(--gap));
    justify-content: flex-start;
    padding-left: 6px;
  }

  /* Caps Lock — 1.75u */
  .key--caps {
    width: calc(1.75 * var(--u) + 0.75 * var(--gap));
    justify-content: flex-start;
    padding-left: 6px;
  }

  /* Caps Lock LED dot */
  .key--caps::after {
    content: "";
    position: absolute;
    right: 7px;
    top: 50%;
    transform: translateY(-50%);
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: rgba(0,0,0,0.2);
    box-shadow: inset 0 1px 2px rgba(0,0,0,0.3);
  }

  /* ISO Enter — top stub occupies ~1.25u in the top row */
  .key--enter-iso-top {
    width: calc(1.25 * var(--u) + 0.25 * var(--gap));
    border-bottom-left-radius: 0;
    border-bottom-right-radius: 0;
    border-bottom: none;
    height: calc(var(--u) + 1px); /* extend 1px to overlap gap visually */
    /* Merge bottom visually with the row below */
    box-shadow: 2px 0 0 rgba(0,0,0,0.22), -1px 0 0 rgba(0,0,0,0.22);
    z-index: 1;
  }

  /* ISO Enter — bottom key spans ~1.75u and has the label */
  .key--enter-iso-bottom {
    width: calc(1.75 * var(--u) + 0.75 * var(--gap));
    border-top-right-radius: 0;
    justify-content: flex-start;
    padding-left: 8px;
  }

  /* ANSI Enter — 2.25u */
  .key--enter-ansi {
    width: calc(2.25 * var(--u) + 1.25 * var(--gap));
    justify-content: flex-start;
    padding-left: 8px;
  }

  /* Left Shift ISO — 1.25u */
  .key--lshift-iso {
    width: calc(1.25 * var(--u) + 0.25 * var(--gap));
    justify-content: flex-start;
    padding-left: 6px;
  }

  /* ISO extra key — 1u (standard) */
  .key--iso-extra {
    min-width: var(--u);
    width: var(--u);
  }

  /* Left Shift ANSI — 2.25u */
  .key--lshift-ansi {
    width: calc(2.25 * var(--u) + 1.25 * var(--gap));
    justify-content: flex-start;
    padding-left: 6px;
  }

  /* Right Shift — 2.75u */
  .key--rshift {
    width: calc(2.75 * var(--u) + 1.75 * var(--gap));
    justify-content: flex-end;
    padding-right: 6px;
  }

  /* Fn, Ctrl — 1.25u */
  .key--fn {
    width: calc(1.25 * var(--u) + 0.25 * var(--gap));
  }

  .key--ctrl {
    width: calc(1.25 * var(--u) + 0.25 * var(--gap));
  }

  /* Option/Alt — 1.25u */
  .key--alt {
    width: calc(1.25 * var(--u) + 0.25 * var(--gap));
  }

  /* Command — 1.5u */
  .key--cmd {
    width: calc(1.5 * var(--u) + 0.5 * var(--gap));
  }

  /* Space bar — 6.25u */
  .key--space {
    flex: 1;
    min-width: calc(6.25 * var(--u) + 5.25 * var(--gap));
  }

  /* ── Arrow cluster ───────────────────────────────────────────────────── */
  .kbd-arrows {
    display: grid;
    grid-template-columns: var(--u) var(--u) var(--u);
    grid-template-rows: calc(var(--u) * 0.55) calc(var(--u) * 0.55);
    gap: var(--gap);
    /* Up arrow center-top, left/down/right on bottom row */
  }

  .key--arrow {
    font-size: 9px;
    min-width: unset;
    width: var(--u);
    height: calc(var(--u) * 0.55);
    padding: 0;
    color: var(--key-mod-text);
  }

  /* Up arrow: col 2 of row 1 */
  .key--arrow:nth-child(1) {
    grid-column: 2;
    grid-row: 1;
  }
  /* Left arrow: col 1 of row 2 */
  .key--arrow:nth-child(2) {
    grid-column: 1;
    grid-row: 2;
  }
  /* Down arrow: col 2 of row 2 */
  .key--arrow:nth-child(3) {
    grid-column: 2;
    grid-row: 2;
  }
  /* Right arrow: col 3 of row 2 */
  .key--arrow:nth-child(4) {
    grid-column: 3;
    grid-row: 2;
  }
</style>
