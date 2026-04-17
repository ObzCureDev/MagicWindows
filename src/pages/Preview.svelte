<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import type { Layout } from "../lib/types";
  import KeyboardVisual from "../components/KeyboardVisual.svelte";

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

  function goInstall() {
    appState.page = "install";
  }

  function goBack() {
    appState.page = "select";
  }
</script>

<div class="preview-page">
  {#if loading}
    <div class="spinner"></div>
  {:else if error}
    <div class="status status--error">{error}</div>
  {:else if layout}
    <!-- Compact top bar: title + actions inline ─────────────────────────── -->
    <header class="preview-top">
      <div class="preview-top__text">
        <h1 class="preview-top__title">
          {layout.name[appState.lang] ?? layout.name["en"] ?? layout.id}
        </h1>
        <p class="preview-top__hint">{t(appState.lang, "preview.instruction")}</p>
      </div>
      <div class="preview-top__actions">
        <button class="btn btn-secondary btn-sm" onclick={goBack}>
          {t(appState.lang, "preview.back")}
        </button>
        <button class="btn btn-primary" onclick={goInstall}>
          {t(appState.lang, "preview.installButton")}
        </button>
      </div>
    </header>

    <!-- Layer toggle -->
    <div class="layer-toggle" role="group" aria-label="Keyboard layer">
      <button
        class="layer-toggle__btn"
        class:layer-toggle__btn--active={activeLayer === "base"}
        onclick={() => (activeLayer = "base")}
      >{t(appState.lang, "preview.baseLayer")}</button>
      <button
        class="layer-toggle__btn"
        class:layer-toggle__btn--active={activeLayer === "shift"}
        onclick={() => (activeLayer = "shift")}
      >{t(appState.lang, "preview.shiftLayer")}</button>
      <button
        class="layer-toggle__btn"
        class:layer-toggle__btn--active={activeLayer === "altgr"}
        onclick={() => (activeLayer = "altgr")}
      >{t(appState.lang, "preview.altgrLayer")}</button>
      <button
        class="layer-toggle__btn"
        class:layer-toggle__btn--active={activeLayer === "altgrShift"}
        onclick={() => (activeLayer = "altgrShift")}
      >{t(appState.lang, "preview.altgrShiftLayer")}</button>
    </div>

    <!-- Keyboard visual (auto-scaled to fit) -->
    <div class="preview-kbd">
      <KeyboardVisual {layout} {activeLayer} />
    </div>

    <!-- Compact legend -->
    <div class="legend legend--compact">
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
</div>

<style>
  .preview-page {
    /* Fits in window without page scroll. Internal Kbd scales via wrapper below. */
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px 18px 14px;
    box-sizing: border-box;
    height: 100%;
    overflow: hidden;
  }
  .preview-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-shrink: 0;
  }
  .preview-top__text { min-width: 0; }
  .preview-top__title {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 600;
    line-height: 1.2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .preview-top__hint {
    margin: 2px 0 0;
    font-size: 0.78rem;
    color: var(--color-text-secondary);
    line-height: 1.3;
  }
  .preview-top__actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }
  /* Wrapper auto-scales the keyboard down so it never needs horizontal scroll. */
  .preview-kbd {
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    /* Override the component's internal --u for this page only — smaller key unit */
    --u: 32px;
    --gap: 4px;
    --radius-key: 6px;
  }
  .preview-kbd :global(.kbd-scaler) {
    overflow: visible;
    padding: 0;
  }
  .preview-kbd :global(.kbd-body) {
    --u: 32px;
    --gap: 4px;
    --radius-key: 6px;
  }
  .legend--compact {
    flex-shrink: 0;
    font-size: 0.78rem;
  }
  /* Tiny button variant just for the inline header */
  .btn-sm {
    padding: 6px 12px;
    font-size: 0.85rem;
  }
</style>
