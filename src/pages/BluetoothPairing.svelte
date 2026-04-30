<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-shell";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import type { AppleKeyboardInfo } from "../lib/types";

  let polling = $state(false);
  let timedOut = $state(false);
  let detectedNow = $state(false);

  let intervalId: number | null = null;
  let timeoutId: number | null = null;

  const POLL_MS = 2000;
  const TIMEOUT_MS = 5 * 60 * 1000;

  function stopPolling() {
    if (intervalId !== null) { clearInterval(intervalId); intervalId = null; }
    if (timeoutId !== null) { clearTimeout(timeoutId); timeoutId = null; }
    polling = false;
  }

  async function probe(): Promise<boolean> {
    try {
      const kbds = await invoke<AppleKeyboardInfo[]>("enumerate_apple_keyboards");
      return kbds.length > 0;
    } catch {
      return false;
    }
  }

  async function startPolling() {
    polling = true;
    timedOut = false;
    detectedNow = false;
    timeoutId = window.setTimeout(() => {
      stopPolling();
      timedOut = true;
    }, TIMEOUT_MS);
    intervalId = window.setInterval(async () => {
      if (await probe()) {
        stopPolling();
        detectedNow = true;
        appState.appleKeyboardConnected = true;
        // Brief beat so user sees the success state, then advance.
        setTimeout(() => { appState.page = "detect"; }, 800);
      }
    }, POLL_MS);
  }

  async function openBluetoothSettings() {
    try {
      await open("ms-settings:bluetooth");
    } catch {
      // Some Tauri shell-plugin configs disallow ms-settings: — best effort.
    }
    if (!polling) await startPolling();
  }

  function skip() {
    stopPolling();
    appState.skippedPairing = true;
    appState.page = "welcome";
  }

  onMount(async () => {
    // Auto-start polling so the user doesn't have to click twice in the
    // happy path (open settings → pair → wait).
    await startPolling();
  });

  onDestroy(() => stopPolling());
</script>

<section class="page bt-pairing">
  <h1>{t(appState.lang, "bluetoothPairing.title")}</h1>
  <p>{t(appState.lang, "bluetoothPairing.subtitle")}</p>

  <div class="actions">
    <button class="btn btn-primary" onclick={openBluetoothSettings}>
      {t(appState.lang, "bluetoothPairing.openSettings")}
    </button>
    <button class="btn btn-secondary" onclick={skip}>
      {t(appState.lang, "bluetoothPairing.skip")}
    </button>
  </div>

  <p class="status">
    {#if detectedNow}
      <span class="ok">{t(appState.lang, "bluetoothPairing.detected")}</span>
    {:else if timedOut}
      <span class="warn">{t(appState.lang, "bluetoothPairing.timeout")}</span>
    {:else if polling}
      <span class="muted">{t(appState.lang, "bluetoothPairing.watching")}</span>
    {/if}
  </p>
</section>

<style>
  .bt-pairing { padding: 32px; max-width: 560px; margin: 0 auto; }
  .actions { display: flex; gap: 12px; margin: 24px 0; }
  .status { min-height: 1.4em; }
  .ok { color: var(--color-success); }
  .warn { color: var(--color-warning, var(--color-danger)); }
  .muted { color: var(--text-muted, #888); }
</style>
