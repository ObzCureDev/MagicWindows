<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import type { LayoutMeta, AppleKeyboardInfo } from "../lib/types";

  let loading = $state(true);

  onMount(async () => {
    try {
      const layouts = await invoke<LayoutMeta[]>("list_layouts");
      appState.layouts = layouts;
    } catch (err) {
      console.error("Failed to load layouts:", err);
      appState.error = String(err);
    } finally {
      loading = false;
    }

    // Apple HID probe — surfaces the BT pairing flow if no Apple keyboard is
    // present. Skipped if the user previously chose "I'll pair later".
    if (appState.skippedPairing) return;
    try {
      const kbds = await invoke<AppleKeyboardInfo[]>("enumerate_apple_keyboards");
      appState.appleKeyboardConnected = kbds.length > 0;
      if (!appState.appleKeyboardConnected) {
        appState.page = "bluetoothPairing";
      }
    } catch {
      // Silently fall through — never block the existing flow.
      appState.appleKeyboardConnected = null;
    }
  });

  function goDetect() {
    appState.page = "detect";
  }

  function goSelect() {
    appState.page = "select";
  }
</script>

<div class="page welcome">
  <div class="welcome__content">
    <img src="/MagicWindows.png" alt="MagicWindows" class="welcome-logo" />

    <p class="welcome__eyebrow">{t(appState.lang, "appSubtitle")}</p>

    <h1 class="welcome__title">
      {t(appState.lang, "welcome.title")}
    </h1>

    <p class="welcome__lead">
      {t(appState.lang, "welcome.description")}
    </p>

    {#if loading}
      <div class="welcome__loading">
        <div class="spinner" role="status" aria-label={t(appState.lang, "common.loading")}></div>
      </div>
    {:else if appState.error}
      <div class="status status--error">{appState.error}</div>
    {:else}
      <div class="welcome-buttons">
        <button class="btn btn-primary btn-large" onclick={goDetect}>
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <circle cx="12" cy="12" r="10" />
            <path d="M12 6v6l4 2" />
          </svg>
          {t(appState.lang, "welcome.detectButton")}
        </button>
        <button class="btn btn-secondary btn-large" onclick={goSelect}>
          {t(appState.lang, "welcome.selectButton")}
        </button>
      </div>

      <div class="welcome__footnote">
        <span class="dot"></span>
        <span>{t(appState.lang, "ui.footnote")}</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .welcome {
    padding-top: 24px;
  }
  .welcome__content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    max-width: 580px;
    text-align: center;
  }
  .welcome__eyebrow {
    margin: 8px 0 0;
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--color-text-muted);
    font-family: var(--font-sans);
  }
  .welcome__title {
    margin: 4px 0 6px;
    font-family: var(--font-display);
    font-style: italic;
    font-weight: 400;
    font-variation-settings: "opsz" 144, "SOFT" 50;
    font-size: clamp(40px, 7vw, 56px);
    line-height: 1.0;
    letter-spacing: -0.03em;
    color: var(--color-text-strong);
    text-wrap: balance;
  }
  .welcome__lead {
    margin: 0 auto 12px;
    max-width: 480px;
    font-size: 15px;
    line-height: 1.55;
    color: var(--color-text-secondary);
    text-wrap: balance;
  }
  .welcome__loading {
    margin-top: 12px;
  }
  .welcome__footnote {
    margin-top: 14px;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--color-text-muted);
    letter-spacing: 0.02em;
  }
  .welcome__footnote .dot {
    width: 6px; height: 6px;
    border-radius: 50%;
    background: var(--color-success);
    box-shadow: 0 0 8px var(--color-success);
  }
</style>
