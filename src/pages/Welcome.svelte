<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores";
  import { t } from "../lib/i18n";
  import type { LayoutMeta } from "../lib/types";

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
  });

  function goDetect() {
    appState.page = "detect";
  }

  function goSelect() {
    appState.page = "select";
  }
</script>

<div class="page">
  <div class="page__content">
    <div class="welcome-logo">M</div>

    <div class="page__header">
      <h1 class="page__title">{t(appState.lang, "welcome.title")}</h1>
      <p class="page__subtitle">{t(appState.lang, "welcome.description")}</p>
    </div>

    {#if loading}
      <div class="spinner"></div>
      <p class="text-secondary">{t(appState.lang, "common.loading")}</p>
    {:else if appState.error}
      <div class="status status--error">{appState.error}</div>
    {:else}
      <div class="welcome-buttons">
        <button class="btn btn-primary btn-large" onclick={goDetect}>
          {t(appState.lang, "welcome.detectButton")}
        </button>
        <button class="btn btn-secondary btn-large" onclick={goSelect}>
          {t(appState.lang, "welcome.selectButton")}
        </button>
      </div>
    {/if}
  </div>
</div>
