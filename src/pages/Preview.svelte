<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores";
  import { t } from "../lib/i18n";
  import type { Layout } from "../lib/types";

  let layout = $state<Layout | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let activeLayer = $state<"base" | "shift" | "altgr" | "altgrShift">("base");

  onMount(async () => {
    if (!appState.selectedLayoutId) {
      appState.page = "select";
      return;
    }
    try {
      const data = await invoke<Layout>("get_layout", {
        id: appState.selectedLayoutId,
      });
      layout = data;
    } catch (err) {
      console.error("Failed to load layout:", err);
      error = String(err);
    } finally {
      loading = false;
    }
  });

  /**
   * Convert a hex codepoint string (e.g. "0041") to a display character.
   * Handles dead keys (suffixed with "@") and the "-1" (no output) sentinel.
   */
  function hexToChar(hex: string): string {
    if (!hex || hex === "-1") return "";
    // Dead key indicator
    const isDead = hex.endsWith("@");
    const clean = isDead ? hex.slice(0, -1) : hex;
    const cp = parseInt(clean, 16);
    if (isNaN(cp) || cp === 0) return "";
    const ch = String.fromCodePoint(cp);
    return isDead ? ch + "\u0332" : ch; // underline for dead keys
  }

  /**
   * Get the character label for a given scancode and the current layer.
   */
  function getKeyLabel(scancode: string): string {
    if (!layout) return "";
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

  /**
   * Check whether a key has a value different from a "standard" assumption.
   * Since we don't ship a Windows default reference, we mark any key that
   * has AltGr or AltGr+Shift output (most Windows layouts have none).
   * For base/shift we just check if the key has data at all (they always differ
   * on Apple layouts).
   */
  function isDifferent(scancode: string): boolean {
    if (!layout) return false;
    const mapping = layout.keys[scancode];
    if (!mapping) return false;
    // For AltGr layers, any non-empty value is "different" from default Windows
    if (activeLayer === "altgr") return mapping.altgr !== "-1";
    if (activeLayer === "altgrShift") return mapping.altgrShift !== "-1";
    // For base/shift — we consider the whole layout "different", but
    // specifically highlight keys where the character is unusual (not simple ASCII a-z/0-9)
    const val = activeLayer === "base" ? mapping.base : mapping.shift;
    if (!val || val === "-1") return false;
    const clean = val.endsWith("@") ? val.slice(0, -1) : val;
    const cp = parseInt(clean, 16);
    if (isNaN(cp)) return false;
    // Characters above basic ASCII are typically Apple-specific
    return cp > 0x7e || val.endsWith("@");
  }

  // Scancode-based physical keyboard rows (ISO layout, matching the JSON keys)
  const numberRow = ["29", "02", "03", "04", "05", "06", "07", "08", "09", "0a", "0b", "0c", "0d"];
  const topRow    = ["10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "1a", "1b"];
  const homeRow   = ["1e", "1f", "20", "21", "22", "23", "24", "25", "26", "27", "28", "2b"];
  const bottomRow = ["56", "2c", "2d", "2e", "2f", "30", "31", "32", "33", "34", "35"];
  const spaceRow  = ["39"];

  function goInstall() {
    appState.page = "install";
  }

  function goBack() {
    // Go back to select or detect depending on where we came from
    appState.page = "select";
  }
</script>

<div class="page">
  <div class="page__header">
    <h1 class="page__title">{t(appState.lang, "preview.title")}</h1>
    <p class="page__subtitle">
      {#if layout}
        {layout.name[appState.lang] ?? layout.name["en"] ?? layout.id}
        &mdash;
      {/if}
      {t(appState.lang, "preview.instruction")}
    </p>
  </div>

  <div class="page__content">
    {#if loading}
      <div class="spinner"></div>
      <p class="text-secondary">{t(appState.lang, "common.loading")}</p>
    {:else if error}
      <div class="status status--error">{error}</div>
    {:else if layout}
      <!-- Layer toggle -->
      <div class="layer-toggle">
        <button
          class="layer-toggle__btn"
          class:layer-toggle__btn--active={activeLayer === "base"}
          onclick={() => (activeLayer = "base")}
        >
          {t(appState.lang, "preview.baseLayer")}
        </button>
        <button
          class="layer-toggle__btn"
          class:layer-toggle__btn--active={activeLayer === "shift"}
          onclick={() => (activeLayer = "shift")}
        >
          {t(appState.lang, "preview.shiftLayer")}
        </button>
        <button
          class="layer-toggle__btn"
          class:layer-toggle__btn--active={activeLayer === "altgr"}
          onclick={() => (activeLayer = "altgr")}
        >
          {t(appState.lang, "preview.altgrLayer")}
        </button>
        <button
          class="layer-toggle__btn"
          class:layer-toggle__btn--active={activeLayer === "altgrShift"}
          onclick={() => (activeLayer = "altgrShift")}
        >
          {t(appState.lang, "preview.altgrShiftLayer")}
        </button>
      </div>

      <!-- Keyboard visual -->
      <div class="keyboard-container">
        <!-- Number row -->
        <div class="keyboard-row">
          {#each numberRow as sc}
            <div
              class="key"
              class:key--different={isDifferent(sc)}
            >
              <span class="key__label">{getKeyLabel(sc)}</span>
            </div>
          {/each}
        </div>

        <!-- Top row (QWERTY/AZERTY row) -->
        <div class="keyboard-row">
          <div class="key" style="min-width: 62px">
            <span class="key__label">Tab</span>
          </div>
          {#each topRow as sc}
            <div
              class="key"
              class:key--different={isDifferent(sc)}
            >
              <span class="key__label">{getKeyLabel(sc)}</span>
            </div>
          {/each}
        </div>

        <!-- Home row -->
        <div class="keyboard-row">
          <div class="key" style="min-width: 72px">
            <span class="key__label">Caps</span>
          </div>
          {#each homeRow as sc}
            <div
              class="key"
              class:key--different={isDifferent(sc)}
            >
              <span class="key__label">{getKeyLabel(sc)}</span>
            </div>
          {/each}
        </div>

        <!-- Bottom row -->
        <div class="keyboard-row">
          <div class="key" style="min-width: 52px">
            <span class="key__label">Shift</span>
          </div>
          {#each bottomRow as sc}
            <div
              class="key"
              class:key--different={isDifferent(sc)}
            >
              <span class="key__label">{getKeyLabel(sc)}</span>
            </div>
          {/each}
          <div class="key" style="min-width: 52px">
            <span class="key__label">Shift</span>
          </div>
        </div>

        <!-- Space row -->
        <div class="keyboard-row" style="justify-content: center;">
          {#each spaceRow as sc}
            <div
              class="key"
              class:key--different={isDifferent(sc)}
              style="min-width: 280px;"
            >
              <span class="key__label">{getKeyLabel(sc) || "Space"}</span>
            </div>
          {/each}
        </div>
      </div>

      <!-- Legend -->
      <div class="legend">
        <div class="legend__item">
          <div class="legend__swatch legend__swatch--different"></div>
          {t(appState.lang, "preview.different")}
        </div>
        <div class="legend__item">
          <div class="legend__swatch legend__swatch--same"></div>
          {t(appState.lang, "preview.same")}
        </div>
      </div>
    {/if}

    <div class="page__actions">
      <button class="btn btn-secondary" onclick={goBack}>
        {t(appState.lang, "preview.back")}
      </button>
      {#if layout}
        <button class="btn btn-primary" onclick={goInstall}>
          {t(appState.lang, "preview.installButton")}
        </button>
      {/if}
    </div>
  </div>
</div>
