<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import type { InstalledLayoutInfo } from "../lib/types";
  import ElevatedErrorPanel from "../components/ElevatedErrorPanel.svelte";

  let layouts = $state<InstalledLayoutInfo[]>([]);
  let loading = $state(true);
  let listError = $state<string | null>(null);

  let removeError = $state<string | null>(null);
  let removeAttempt = $state(0);
  let lastRemoveTarget = $state<InstalledLayoutInfo | null>(null);

  async function loadLayouts() {
    loading = true;
    listError = null;
    try {
      layouts = await invoke<InstalledLayoutInfo[]>("list_all_installed_layouts");
    } catch (err) {
      console.error("Failed to list layouts:", err);
      listError = String(err);
    } finally {
      loading = false;
    }
  }

  async function doRemove(layout: InstalledLayoutInfo) {
    removeError = null;
    try {
      await invoke("uninstall_by_klid", { klid: layout.klid });
      await loadLayouts();
    } catch (err) {
      console.error("Remove failed:", err);
      removeError = String(err);
      removeAttempt += 1;
    }
  }

  async function retryLastRemove() {
    if (lastRemoveTarget) {
      await doRemove(lastRemoveTarget);
    }
  }

  async function requestRemove(layout: InstalledLayoutInfo) {
    const key = layout.isInUse ? "settings.confirmRemoveInUse" : "settings.confirmRemove";
    const msg = t(appState.lang, key, { name: layout.layoutText || layout.klid });
    if (!window.confirm(msg)) return;
    lastRemoveTarget = layout;
    removeAttempt = 0;
    await doRemove(layout);
  }

  function goBack() {
    appState.page = "welcome";
  }

  onMount(loadLayouts);
</script>

<div class="page">
  <div class="page__header">
    <h1 class="page__title">{t(appState.lang, "settings.title")}</h1>
    <p class="page__subtitle">{t(appState.lang, "settings.subtitle")}</p>
  </div>

  <div class="page__content">
    <ElevatedErrorPanel
      error={removeError}
      onRetry={retryLastRemove}
      operationName="uninstall_by_klid"
      context={{ klid: lastRemoveTarget?.klid, layoutFile: lastRemoveTarget?.layoutFile }}
      attemptCount={removeAttempt}
    />

    {#if loading}
      <div class="spinner"></div>
      <p class="text-secondary">{t(appState.lang, "settings.loading")}</p>
    {:else if listError}
      <div class="status status--error">{listError}</div>
    {:else if layouts.length === 0}
      <p class="text-secondary text-center">{t(appState.lang, "settings.empty")}</p>
    {:else}
      <div class="layout-list">
        {#each layouts as layout (layout.klid)}
          <div class="layout-row">
            <div class="layout-row__main">
              <div class="layout-row__title">{layout.layoutText || layout.klid}</div>
              <div class="layout-row__meta">
                <code>{layout.layoutFile}</code>
                <span class="layout-row__dot">·</span>
                <code>{layout.klid}</code>
              </div>
              <div class="layout-row__badges">
                {#if layout.isMagicWindows}
                  <span class="badge badge--magic">{t(appState.lang, "settings.badgeMagic")}</span>
                {:else if layout.klid.startsWith("0000")}
                  <span class="badge badge--system">{t(appState.lang, "settings.badgeSystem")}</span>
                {/if}
                {#if layout.isInUse}
                  <span class="badge badge--in-use">{t(appState.lang, "settings.badgeInUse")}</span>
                {/if}
              </div>
            </div>
            <div class="layout-row__actions">
              <button
                class="btn btn-danger btn-sm"
                onclick={() => requestRemove(layout)}
              >
                {t(appState.lang, "settings.remove")}
              </button>
              <span
                class="info-icon"
                title={t(appState.lang, "settings.reactivateInfo")}
                aria-label={t(appState.lang, "settings.reactivateInfo")}
              >&#9432;</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <div class="page__actions">
      <button class="btn btn-secondary" onclick={goBack}>
        {t(appState.lang, "common.back")}
      </button>
    </div>
  </div>
</div>

<style>
  .layout-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    max-width: 640px;
    margin: 0 auto;
  }
  .layout-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 14px;
    background: var(--color-bg-elevated, rgba(255,255,255,0.04));
    border: 1px solid var(--color-border, rgba(0,0,0,0.15));
    border-radius: 8px;
  }
  .layout-row__main { display: flex; flex-direction: column; gap: 4px; min-width: 0; flex: 1; }
  .layout-row__title { font-weight: 600; }
  .layout-row__meta { font-size: 0.78rem; color: var(--color-text-secondary); display: flex; gap: 6px; align-items: center; }
  .layout-row__meta code { font-size: 0.78rem; }
  .layout-row__dot { opacity: 0.5; }
  .layout-row__badges { display: flex; gap: 6px; margin-top: 2px; flex-wrap: wrap; }
  .badge {
    display: inline-block;
    padding: 2px 8px;
    border-radius: 999px;
    font-size: 0.7rem;
    font-weight: 500;
    white-space: nowrap;
  }
  .badge--magic { background: rgba(59, 130, 246, 0.15); color: #3b82f6; }
  .badge--system { background: rgba(156, 163, 175, 0.15); color: #6b7280; }
  .badge--in-use { background: rgba(34, 197, 94, 0.15); color: #22c55e; }
  .layout-row__actions { display: flex; align-items: center; gap: 8px; }
  .info-icon {
    cursor: help;
    color: var(--color-text-secondary);
    font-size: 1.05rem;
  }
  .btn-sm { padding: 4px 10px; font-size: 0.85rem; }
  .btn-danger { background: #dc3545; color: white; border-color: #dc3545; }
  .btn-danger:hover { background: #c82333; }
</style>
