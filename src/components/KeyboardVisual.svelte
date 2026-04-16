<script lang="ts">
  import type { Layout } from "../lib/types";

  interface Props {
    layout: Layout;
    activeLayer: "base" | "shift" | "altgr" | "altgrShift";
  }

  let { layout, activeLayer }: Props = $props();

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
      case "base":
        return hexToChar(mapping.base);
      case "shift":
        return hexToChar(mapping.shift);
      case "altgr":
        return hexToChar(mapping.altgr);
      case "altgrShift":
        return hexToChar(mapping.altgrShift);
    }
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

  const numberRow = ["29", "02", "03", "04", "05", "06", "07", "08", "09", "0a", "0b", "0c", "0d"];
  const topRow    = ["10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "1a", "1b"];
  const homeRow   = ["1e", "1f", "20", "21", "22", "23", "24", "25", "26", "27", "28", "2b"];
  const bottomRow = ["56", "2c", "2d", "2e", "2f", "30", "31", "32", "33", "34", "35"];
  const spaceRow  = ["39"];
</script>

<div class="keyboard-container">
  <div class="keyboard-row">
    {#each numberRow as sc}
      <div class="key" class:key--different={isDifferent(sc)}>
        <span class="key__label">{getKeyLabel(sc)}</span>
      </div>
    {/each}
  </div>

  <div class="keyboard-row">
    <div class="key" style="min-width: 62px">
      <span class="key__label">Tab</span>
    </div>
    {#each topRow as sc}
      <div class="key" class:key--different={isDifferent(sc)}>
        <span class="key__label">{getKeyLabel(sc)}</span>
      </div>
    {/each}
  </div>

  <div class="keyboard-row">
    <div class="key" style="min-width: 72px">
      <span class="key__label">Caps</span>
    </div>
    {#each homeRow as sc}
      <div class="key" class:key--different={isDifferent(sc)}>
        <span class="key__label">{getKeyLabel(sc)}</span>
      </div>
    {/each}
  </div>

  <div class="keyboard-row">
    <div class="key" style="min-width: 52px">
      <span class="key__label">Shift</span>
    </div>
    {#each bottomRow as sc}
      <div class="key" class:key--different={isDifferent(sc)}>
        <span class="key__label">{getKeyLabel(sc)}</span>
      </div>
    {/each}
    <div class="key" style="min-width: 52px">
      <span class="key__label">Shift</span>
    </div>
  </div>

  <div class="keyboard-row" style="justify-content: center;">
    {#each spaceRow as sc}
      <div class="key" class:key--different={isDifferent(sc)} style="min-width: 280px;">
        <span class="key__label">{getKeyLabel(sc) || "Space"}</span>
      </div>
    {/each}
  </div>
</div>
