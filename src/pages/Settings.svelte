<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import type { InstalledLayoutInfo, LayoutMeta } from "../lib/types";
  import ElevatedErrorPanel from "../components/ElevatedErrorPanel.svelte";

  // "kbdaplfr.dll" → "apple-fr-azerty". Derived from the bundled layout list
  // (loaded by Welcome) so adding a new layouts/*.json automatically lights
  // up its Health Check button — no manual map maintenance.
  // LayoutMeta.dllName is the bare basename (e.g. "kbdaplfr") while the
  // registry-side InstalledLayoutInfo.layoutFile carries the ".dll" suffix,
  // so we append it when building the map keys.
  const layoutIdByDll = $derived.by(() => {
    const map: Record<string, string> = {};
    for (const l of appState.layouts) {
      map[`${l.dllName.toLowerCase()}.dll`] = l.id;
    }
    return map;
  });

  function layoutIdFromDll(dllName: string): string {
    return layoutIdByDll[dllName.toLowerCase()] ?? "";
  }

  let allLayouts = $state<InstalledLayoutInfo[]>([]);
  let loading = $state(true);
  let listError = $state<string | null>(null);

  let removeError = $state<string | null>(null);
  let removeAttempt = $state(0);
  let lastRemoveTarget = $state<InstalledLayoutInfo | null>(null);

  let activeLayouts = $derived(allLayouts.filter((l) => l.isInUse));

  async function loadLayouts() {
    loading = true;
    listError = null;
    try {
      allLayouts = await invoke<InstalledLayoutInfo[]>("list_all_installed_layouts");
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

  onMount(async () => {
    // Settings is reachable directly from the top-bar gear icon, so the user
    // can land here without going through Welcome (which is where layouts are
    // normally fetched). When that happens, the layoutIdByDll derived map
    // would be empty and the Health Check button would silently disappear.
    if (appState.layouts.length === 0) {
      try {
        appState.layouts = await invoke<LayoutMeta[]>("list_layouts");
      } catch {
        // Silently ignore — layoutIdFromDll will return empty for all entries,
        // hiding the Health Check buttons. Acceptable failure mode.
      }
    }
    await loadLayouts();
  });
</script>

<div class="page settings">
  <div class="settings-topbar">
    <button class="btn btn-secondary btn-sm" onclick={goBack} aria-label={t(appState.lang, "common.back")}>
      <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <path d="M19 12H5M11 18l-6-6 6-6" />
      </svg>
      {t(appState.lang, "common.back")}
    </button>
  </div>

  <div class="page__header">
    <p class="settings__eyebrow">{t(appState.lang, "ui.manage")}</p>
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
    {:else if activeLayouts.length === 0}
      <p class="text-secondary text-center">{t(appState.lang, "settings.empty")}</p>
    {:else}
      <div class="layout-list">
        {#each activeLayouts as layout (layout.klid)}
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
              {#if layout.isMagicWindows && layoutIdFromDll(layout.layoutFile)}
                <button
                  class="btn btn-secondary btn-sm"
                  title={t(appState.lang, "settings.healthCheckHint")}
                  onclick={() => {
                    // Defense in depth: even though the surrounding {#if layoutIdFromDll(...)}
                    // hides this button for unknown DLLs, the click handler also bails out
                    // rather than navigate to HealthCheck with an empty layoutId.
                    const layoutId = layoutIdFromDll(layout.layoutFile);
                    if (!layoutId) return;
                    appState.healthCheckTarget = { layoutId, klid: layout.klid };
                    appState.page = "healthCheck";
                  }}
                >
                  {t(appState.lang, "settings.healthCheck")}
                </button>
              {/if}
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
  .settings__eyebrow {
    margin: 0;
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }
  .settings-topbar {
    width: 100%;
    max-width: 640px;
    margin: 0 auto 4px;
    display: flex;
    justify-content: flex-start;
  }
  .layout-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: 100%;
    max-width: 640px;
    margin: 0 auto;
  }
  .layout-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 14px 18px;
    background: var(--color-bg-card);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-xs);
    transition: border-color var(--transition-fast), background var(--transition-fast);
  }
  .layout-row:hover {
    border-color: var(--color-border-strong);
    background: var(--color-bg-card-hover);
  }
  .layout-row__main {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    flex: 1;
  }
  .layout-row__title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text);
  }
  .layout-row__meta {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--color-text-muted);
    display: flex;
    gap: 8px;
    align-items: center;
    letter-spacing: 0.02em;
  }
  .layout-row__meta code { font-size: 11px; }
  .layout-row__dot { opacity: 0.5; }
  .layout-row__badges {
    display: flex;
    gap: 6px;
    margin-top: 4px;
    flex-wrap: wrap;
  }
  .badge {
    display: inline-block;
    padding: 3px 9px;
    border-radius: var(--radius-pill);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    white-space: nowrap;
    border: 1px solid transparent;
  }
  .badge--magic {
    background: var(--color-accent-soft);
    color: var(--color-accent);
    border-color: var(--color-accent-ring);
  }
  .badge--system {
    background: var(--color-overlay);
    color: var(--color-text-secondary);
    border-color: var(--color-border);
  }
  .badge--in-use {
    background: var(--color-success-bg);
    color: var(--color-success);
    border-color: var(--color-success-border);
  }
  .layout-row__actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .info-icon {
    cursor: help;
    color: var(--color-text-muted);
    font-size: 1.05rem;
  }
</style>
