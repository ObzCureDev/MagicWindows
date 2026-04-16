<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores";
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
      <div class="layer-toggle" role="group" aria-label="Keyboard layer">
        <button
          class="layer-toggle__btn"
          class:layer-toggle__btn--active={activeLayer === "base"}
          aria-pressed={activeLayer === "base"}
          onclick={() => (activeLayer = "base")}
        >
          {t(appState.lang, "preview.baseLayer")}
        </button>
        <button
          class="layer-toggle__btn"
          class:layer-toggle__btn--active={activeLayer === "shift"}
          aria-pressed={activeLayer === "shift"}
          onclick={() => (activeLayer = "shift")}
        >
          {t(appState.lang, "preview.shiftLayer")}
        </button>
        <button
          class="layer-toggle__btn"
          class:layer-toggle__btn--active={activeLayer === "altgr"}
          aria-pressed={activeLayer === "altgr"}
          onclick={() => (activeLayer = "altgr")}
        >
          {t(appState.lang, "preview.altgrLayer")}
        </button>
        <button
          class="layer-toggle__btn"
          class:layer-toggle__btn--active={activeLayer === "altgrShift"}
          aria-pressed={activeLayer === "altgrShift"}
          onclick={() => (activeLayer = "altgrShift")}
        >
          {t(appState.lang, "preview.altgrShiftLayer")}
        </button>
      </div>

      <!-- Keyboard visual -->
      <KeyboardVisual {layout} {activeLayer} />

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
