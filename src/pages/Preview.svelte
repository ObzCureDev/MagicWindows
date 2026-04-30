<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import type { Layout } from "../lib/types";
  import { KeyboardVisual } from "@magicwindows/keyboard-visual";

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
    <div class="preview-center"><div class="spinner"></div></div>
  {:else if error}
    <div class="status status--error">{error}</div>
  {:else if layout}
    <header class="preview-top">
      <div class="preview-top__text">
        <p class="preview-top__eyebrow">{t(appState.lang, "ui.step", { n: "03" })}</p>
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
          <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M5 12h14M13 6l6 6-6 6" />
          </svg>
        </button>
      </div>
    </header>

    <div class="preview-controls">
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
    </div>

    <div class="preview-kbd">
      <KeyboardVisual {layout} {activeLayer} />
    </div>
  {/if}
</div>

<style>
  .preview-page {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px 24px 22px;
    box-sizing: border-box;
    height: 100%;
    overflow: hidden;
  }
  .preview-center {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .preview-top {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
    flex-shrink: 0;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--color-border);
  }
  .preview-top__text { min-width: 0; }
  .preview-top__eyebrow {
    margin: 0 0 4px;
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }
  .preview-top__title {
    margin: 0;
    font-family: var(--font-display);
    font-style: italic;
    font-weight: 400;
    font-variation-settings: "opsz" 144;
    font-size: 24px;
    line-height: 1.1;
    letter-spacing: -0.02em;
    color: var(--color-text-strong);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .preview-top__hint {
    margin: 4px 0 0;
    font-size: 13px;
    color: var(--color-text-secondary);
    line-height: 1.4;
    max-width: 480px;
  }
  .preview-top__actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }

  .preview-controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .preview-kbd {
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
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
    font-size: 12px;
  }
</style>
