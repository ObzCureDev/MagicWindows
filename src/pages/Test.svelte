<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import type { Layout } from "../lib/types";
  import { KeyboardVisual } from "@magicwindows/keyboard-visual";

  let layout = $state<Layout | null>(null);
  let text = $state("");
  let lastCode = $state("");
  let lastKey = $state("");
  let lastMods = $state("");

  function onKey(e: KeyboardEvent) {
    lastCode = e.code;
    lastKey = e.key;
    const mods = [];
    if (e.shiftKey) mods.push("shift");
    if (e.ctrlKey)  mods.push("ctrl");
    if (e.altKey)   mods.push("alt");
    if (e.metaKey)  mods.push("meta");
    lastMods = mods.length ? mods.join("+") : "—";
  }

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
      <p class="test-top__eyebrow">{t(appState.lang, "ui.step", { n: "04" })}</p>
      <h1 class="test-top__title">{t(appState.lang, "test.title")}</h1>
      <p class="test-top__hint">{t(appState.lang, "test.hint")}</p>
    </div>
    <button class="btn btn-primary" onclick={goDone}>
      {t(appState.lang, "test.continue")}
      <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M5 12h14M13 6l6 6-6 6" />
      </svg>
    </button>
  </header>

  <!-- svelte-ignore a11y_autofocus -->
  <textarea
    class="test-input"
    placeholder={t(appState.lang, "test.placeholder")}
    bind:value={text}
    onkeydown={onKey}
    autofocus
  ></textarea>

  {#if lastCode}
    <p class="test-debug">
      <span class="test-debug__chip">code <b>{lastCode}</b></span>
      <span class="test-debug__chip">key <b>{lastKey}</b></span>
      <span class="test-debug__chip">mods <b>{lastMods}</b></span>
    </p>
  {/if}

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
    gap: 12px;
    padding: 18px 24px 22px;
    box-sizing: border-box;
    height: 100%;
    overflow: hidden;
  }
  .test-top {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
    flex-shrink: 0;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--color-border);
  }
  .test-top__text { min-width: 0; }
  .test-top__eyebrow {
    margin: 0 0 4px;
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }
  .test-top__title {
    margin: 0;
    font-family: var(--font-display);
    font-style: italic;
    font-weight: 400;
    font-variation-settings: "opsz" 144;
    font-size: 24px;
    line-height: 1.1;
    letter-spacing: -0.02em;
    color: var(--color-text-strong);
  }
  .test-top__hint {
    margin: 4px 0 0;
    font-size: 13px;
    color: var(--color-text-secondary);
    line-height: 1.4;
  }
  .test-input {
    flex-shrink: 0;
    width: 100%;
    min-height: 90px;
    padding: 14px 16px;
    box-sizing: border-box;
    font-family: var(--font-mono);
    font-size: 14px;
    line-height: 1.5;
    color: var(--color-text);
    background: var(--color-bg-card);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    resize: vertical;
    box-shadow: var(--shadow-xs);
    transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
  }
  .test-input::placeholder { color: var(--color-text-muted); }
  .test-input:focus {
    outline: none;
    border-color: var(--color-accent);
    box-shadow: 0 0 0 3px var(--color-accent-soft);
  }
  .test-debug {
    flex-shrink: 0;
    margin: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--color-text-secondary);
  }
  .test-debug__chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 8px;
    background: var(--color-overlay);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-pill);
  }
  .test-debug__chip b {
    color: var(--color-text);
    font-weight: 600;
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
