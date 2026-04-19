<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import ElevatedErrorPanel from "../components/ElevatedErrorPanel.svelte";

  let installing = $state(true);
  let success = $state(false);
  let error = $state<string | null>(null);
  let attemptCount = $state(0);

  async function runInstall() {
    if (!appState.selectedLayoutId) {
      appState.page = "select";
      return;
    }
    installing = true;
    appState.installing = true;
    error = null;
    try {
      const elapsedMs = await invoke<number>("install_layout", {
        id: appState.selectedLayoutId,
      });
      appState.lastInstallMs = elapsedMs;
      success = true;
    } catch (err) {
      console.error("Installation failed:", err);
      error = String(err);
      attemptCount += 1;
    } finally {
      installing = false;
      appState.installing = false;
    }
  }

  async function openSettings() {
    try {
      const { open } = await import("@tauri-apps/plugin-shell");
      await open("ms-settings:regionlanguage");
    } catch (e) {
      console.error("Could not open settings:", e);
    }
  }

  function goDone() {
    appState.page = "test";
  }

  function goBack() {
    appState.page = "preview";
  }

  onMount(runInstall);
</script>

<div class="page install">
  <div class="page__content">
    {#if installing}
      <div class="install-progress">
        <div class="install-progress__ring">
          <div class="spinner"></div>
        </div>
        <p class="install-progress__eyebrow">{t(appState.lang, "ui.installing")}</p>
        <h1 class="install-progress__title">{t(appState.lang, "install.installing")}</h1>
        <p class="install-progress__hint">{t(appState.lang, "ui.installHint")}</p>
      </div>
    {:else if success}
      <div class="install-success">
        <div class="checkmark">&#10003;</div>
        <h1 class="page__title">{t(appState.lang, "install.success")}</h1>

        {#if appState.lastInstallMs !== null}
          <p class="install-success__elapsed">
            {t(appState.lang, "install.elapsed", {
              seconds: (appState.lastInstallMs / 1000).toFixed(1),
            })}
          </p>
        {/if}

        <p class="page__subtitle">
          {t(appState.lang, "done.instructions")}
        </p>

        <div class="page__actions">
          <button class="btn btn-secondary" onclick={openSettings}>
            {t(appState.lang, "install.openSettings")}
          </button>
          <button class="btn btn-primary" onclick={goDone}>
            {t(appState.lang, "install.done")}
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M5 12h14M13 6l6 6-6 6" />
            </svg>
          </button>
        </div>
      </div>
    {:else if error}
      <ElevatedErrorPanel
        {error}
        onRetry={runInstall}
        operationName="install_layout"
        context={{ layoutId: appState.selectedLayoutId }}
        {attemptCount}
      />

      <div class="page__actions">
        <button class="btn btn-secondary" onclick={goBack}>
          {t(appState.lang, "common.back")}
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .install-progress {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    padding: 24px 0;
  }
  .install-progress__ring {
    width: 96px;
    height: 96px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: var(--color-accent-soft);
    box-shadow:
      0 0 0 1px var(--color-accent-ring),
      0 0 0 12px color-mix(in srgb, var(--color-accent) 5%, transparent),
      0 12px 30px var(--color-accent-glow);
    animation: ringPulse 2.4s ease-in-out infinite;
  }
  @keyframes ringPulse {
    0%, 100% { transform: scale(1); }
    50%      { transform: scale(1.04); }
  }
  .install-progress__ring :global(.spinner) {
    width: 44px;
    height: 44px;
    border-width: 3px;
  }
  .install-progress__eyebrow {
    margin: 8px 0 0;
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }
  .install-progress__title {
    margin: 0;
    font-family: var(--font-display);
    font-style: italic;
    font-weight: 400;
    font-variation-settings: "opsz" 144;
    font-size: 32px;
    line-height: 1.1;
    letter-spacing: -0.02em;
    color: var(--color-text-strong);
    text-align: center;
  }
  .install-progress__hint {
    margin: 0;
    font-size: 13px;
    color: var(--color-text-muted);
  }

  .install-success {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
  }
  .install-success__elapsed {
    margin: -4px 0 0;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--color-text-muted);
    letter-spacing: 0.04em;
  }
</style>
