<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import type { Layout } from "../lib/types";
  import KeyboardVisual from "../components/KeyboardVisual.svelte";

  let layout = $state<Layout | null>(null);
  let text = $state("");

  onMount(async () => {
    if (!appState.selectedLayoutId) {
      appState.page = "done";
      return;
    }
    try {
      layout = await invoke<Layout>("get_layout", { id: appState.selectedLayoutId });
    } catch (e) {
      console.error("Failed to load layout for test:", e);
    }
  });

  function goDone() {
    appState.page = "done";
  }
</script>

<div class="test-page">
  <header class="test-top">
    <div class="test-top__text">
      <h1 class="test-top__title">{t(appState.lang, "test.title")}</h1>
      <p class="test-top__hint">{t(appState.lang, "test.hint")}</p>
    </div>
    <button class="btn btn-primary" onclick={goDone}>
      {t(appState.lang, "test.continue")}
    </button>
  </header>

  <!-- svelte-ignore a11y_autofocus -->
  <textarea
    class="test-input"
    placeholder={t(appState.lang, "test.placeholder")}
    bind:value={text}
    autofocus
  ></textarea>

  {#if layout}
    <div class="test-kbd">
      <KeyboardVisual {layout} activeLayer="base" />
    </div>
  {/if}
</div>

<style>
  .test-page {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px 18px 14px;
    box-sizing: border-box;
    height: 100%;
    overflow: hidden;
  }
  .test-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-shrink: 0;
  }
  .test-top__text { min-width: 0; }
  .test-top__title {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 600;
    line-height: 1.2;
  }
  .test-top__hint {
    margin: 2px 0 0;
    font-size: 0.78rem;
    color: var(--color-text-secondary);
    line-height: 1.3;
  }
  .test-input {
    flex-shrink: 0;
    width: 100%;
    min-height: 90px;
    padding: 10px 12px;
    box-sizing: border-box;
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", Helvetica, Arial, sans-serif;
    font-size: 1rem;
    color: var(--color-text);
    background: var(--color-bg-elevated, rgba(255,255,255,0.04));
    border: 1px solid var(--color-border, rgba(0,0,0,0.15));
    border-radius: 8px;
    resize: vertical;
  }
  .test-input:focus {
    outline: 2px solid var(--color-primary, #2865d4);
    outline-offset: -1px;
  }
  .test-kbd {
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
  .test-kbd :global(.kbd-body) {
    --u: 32px;
    --gap: 4px;
    --radius-key: 6px;
  }
</style>
