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
      await invoke("install_layout", { id: appState.selectedLayoutId });
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

<div class="page">
  <div class="page__header">
    <h1 class="page__title">{t(appState.lang, "install.title")}</h1>
  </div>

  <div class="page__content">
    {#if installing}
      <div class="spinner"></div>
      <p class="text-secondary">{t(appState.lang, "install.installing")}</p>
    {:else if success}
      <div class="status status--success">
        {t(appState.lang, "install.success")}
      </div>

      <p class="text-secondary text-center" style="max-width: 420px;">
        {t(appState.lang, "done.instructions")}
      </p>

      <div class="page__actions">
        <button class="btn btn-secondary" onclick={openSettings}>
          {t(appState.lang, "install.openSettings")}
        </button>
        <button class="btn btn-primary" onclick={goDone}>
          {t(appState.lang, "install.done")}
        </button>
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
