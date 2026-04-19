<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import type { InstalledLayoutInfo, Layout, ModifierToggles } from "../lib/types";
  import ElevatedErrorPanel from "../components/ElevatedErrorPanel.svelte";

  type ModPreset = "none" | "macShortcuts" | "winStrict";

  let selectedPreset = $state<ModPreset>("none");
  let applying = $state(false);
  let applied = $state(false);
  let modError = $state<string | null>(null);
  let modAttempt = $state(0);
  let uninstallError = $state<string | null>(null);
  let uninstallAttempt = $state(0);

  let windowsDefaultTarget = $state<InstalledLayoutInfo | null>(null);
  let windowsDefaultRemoved = $state(false);
  let removeWinError = $state<string | null>(null);
  let removeWinAttempt = $state(0);

  async function detectWindowsDefault() {
    if (!appState.selectedLayoutId) return;
    try {
      const installedLayout = await invoke<Layout>("get_layout", { id: appState.selectedLayoutId });
      const localeHex = installedLayout.localeId.slice(-4).toLowerCase();
      const allLayouts = await invoke<InstalledLayoutInfo[]>("list_all_installed_layouts");
      windowsDefaultTarget = allLayouts.find(
        (l) => l.klid.startsWith("0000") && l.klid.slice(-4).toLowerCase() === localeHex,
      ) ?? null;
    } catch (err) {
      console.error("detectWindowsDefault failed:", err);
    }
  }

  async function removeWindowsDefault() {
    if (!windowsDefaultTarget) return;
    const msg = t(appState.lang, "settings.confirmRemove", {
      name: windowsDefaultTarget.layoutText || windowsDefaultTarget.klid,
    });
    if (!window.confirm(msg)) return;
    removeWinError = null;
    try {
      await invoke("uninstall_by_klid", { klid: windowsDefaultTarget.klid });
      windowsDefaultRemoved = true;
      windowsDefaultTarget = null;
    } catch (err) {
      console.error("Remove Windows default failed:", err);
      removeWinError = String(err);
      removeWinAttempt += 1;
    }
  }

  async function retryRemoveWindowsDefault() {
    await removeWindowsDefault();
  }

  onMount(detectWindowsDefault);

  function presetToToggles(p: ModPreset): ModifierToggles {
    switch (p) {
      case "macShortcuts":
        return { swapCmdCtrlLeft: true, swapCmdCtrlRight: true, capsToCtrl: false, swapOptionCmd: false };
      case "winStrict":
        return { swapCmdCtrlLeft: false, swapCmdCtrlRight: false, capsToCtrl: false, swapOptionCmd: true };
      default:
        return { swapCmdCtrlLeft: false, swapCmdCtrlRight: false, capsToCtrl: false, swapOptionCmd: false };
    }
  }

  async function applyPreset() {
    if (selectedPreset === "none") return;
    applying = true;
    modError = null;
    try {
      const toggles = presetToToggles(selectedPreset);
      await invoke("write_scancode_map", { toggles });
      applied = true;
    } catch (err) {
      console.error("write_scancode_map failed:", err);
      modError = String(err);
      modAttempt += 1;
    } finally {
      applying = false;
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

  async function uninstall() {
    if (!appState.selectedLayoutId) return;
    uninstallError = null;
    try {
      await invoke("uninstall_layout", { id: appState.selectedLayoutId });
      appState.selectedLayoutId = null;
      appState.page = "welcome";
    } catch (err) {
      console.error("Uninstall failed:", err);
      uninstallError = String(err);
      uninstallAttempt += 1;
    }
  }

  async function close() {
    try {
      await invoke("quit_app");
    } catch (e) {
      console.error("quit_app failed:", e);
      window.close();
    }
  }
</script>

<div class="page done">
  <div class="page__content">
    <div class="checkmark">&#10003;</div>

    <div class="done__hero">
      <p class="done__eyebrow">{t(appState.lang, "ui.allSet")}</p>
      <h1 class="page__title">{t(appState.lang, "done.title")}</h1>
      <p class="done__lead">{t(appState.lang, "done.congratulations")}</p>
    </div>

    <div class="status status--info">
      {t(appState.lang, "done.switchInfo")}
    </div>

    <p class="done__instructions">
      {t(appState.lang, "done.instructions")}
    </p>

    {#if windowsDefaultTarget}
      <section class="panel">
        <header class="panel__header">
          <h2 class="panel__title">
            {t(appState.lang, "done.removeWindowsDefault", { locale: windowsDefaultTarget.layoutText })}
          </h2>
          <span
            class="info-icon"
            title={t(appState.lang, "settings.reactivateInfo")}
            aria-label={t(appState.lang, "settings.reactivateInfo")}
          >&#9432;</span>
        </header>
        <button class="btn btn-secondary" onclick={removeWindowsDefault}>
          {t(appState.lang, "done.removeWindowsDefaultBtn", { locale: windowsDefaultTarget.layoutText })}
        </button>
        <ElevatedErrorPanel
          error={removeWinError}
          onRetry={retryRemoveWindowsDefault}
          operationName="uninstall_by_klid"
          context={{ klid: windowsDefaultTarget.klid }}
          attemptCount={removeWinAttempt}
        />
      </section>
    {:else if windowsDefaultRemoved}
      <div class="status status--success">
        {t(appState.lang, "done.windowsDefaultRemoved")}
      </div>
    {/if}

    <section class="panel">
      <header class="panel__header panel__header--col">
        <h2 class="panel__title">{t(appState.lang, "done.modOfferTitle")}</h2>
        <p class="panel__hint">{t(appState.lang, "done.modOfferHint")}</p>
      </header>

      <div class="mod-list">
        <label class="mod-opt" class:mod-opt--active={selectedPreset === "none"}>
          <input type="radio" name="mod-preset" value="none" bind:group={selectedPreset} disabled={applied} />
          <div class="mod-opt__text">
            <span class="mod-opt__label">{t(appState.lang, "done.modNone")}</span>
          </div>
        </label>
        <label class="mod-opt" class:mod-opt--active={selectedPreset === "macShortcuts"}>
          <input type="radio" name="mod-preset" value="macShortcuts" bind:group={selectedPreset} disabled={applied} />
          <div class="mod-opt__text">
            <span class="mod-opt__label">{t(appState.lang, "done.modMac")}</span>
            <span class="mod-opt__desc">{t(appState.lang, "done.modMacDesc")}</span>
          </div>
        </label>
        <label class="mod-opt" class:mod-opt--active={selectedPreset === "winStrict"}>
          <input type="radio" name="mod-preset" value="winStrict" bind:group={selectedPreset} disabled={applied} />
          <div class="mod-opt__text">
            <span class="mod-opt__label">{t(appState.lang, "done.modWin")}</span>
            <span class="mod-opt__desc">{t(appState.lang, "done.modWinDesc")}</span>
          </div>
        </label>
      </div>

      <ElevatedErrorPanel
        error={modError}
        onRetry={applyPreset}
        operationName="write_scancode_map"
        context={{ preset: selectedPreset }}
        attemptCount={modAttempt}
      />

      {#if applied}
        <div class="status status--success">{t(appState.lang, "done.modApplied")}</div>
      {:else if selectedPreset !== "none"}
        <button class="btn btn-primary mod-offer__apply" onclick={applyPreset} disabled={applying}>
          {applying ? t(appState.lang, "done.modApplying") : t(appState.lang, "done.modApply")}
        </button>
      {/if}
    </section>

    <div class="page__actions">
      <button class="btn btn-secondary" onclick={openSettings}>
        {t(appState.lang, "install.openSettings")}
      </button>
      <button class="btn btn-primary" onclick={close}>
        {t(appState.lang, "done.close")}
      </button>
    </div>

    <div class="done__danger">
      <ElevatedErrorPanel
        error={uninstallError}
        onRetry={uninstall}
        operationName="uninstall_layout"
        context={{ layoutId: appState.selectedLayoutId }}
        attemptCount={uninstallAttempt}
      />
      <button class="btn btn-danger" onclick={uninstall}>
        {t(appState.lang, "done.uninstall")}
      </button>
    </div>
  </div>
</div>

<style>
  .done__hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }
  .done__eyebrow {
    margin: 0;
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--color-success);
  }
  .done__lead {
    margin: 4px 0 0;
    font-size: 15px;
    line-height: 1.55;
    color: var(--color-text-secondary);
    text-align: center;
    max-width: 480px;
    text-wrap: balance;
  }
  .done__instructions {
    margin: 0;
    text-align: center;
    color: var(--color-text-secondary);
    font-size: 13px;
    line-height: 1.5;
    max-width: 480px;
  }

  .panel {
    width: 100%;
    max-width: 520px;
    margin: 4px auto;
    padding: 16px 18px;
    background: var(--color-bg-card);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xs);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .panel__header {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    justify-content: space-between;
  }
  .panel__header--col { flex-direction: column; gap: 4px; }
  .panel__title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text);
    line-height: 1.4;
  }
  .panel__hint {
    margin: 0;
    font-size: 12px;
    color: var(--color-text-muted);
    line-height: 1.4;
  }
  .info-icon {
    cursor: help;
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .mod-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .mod-opt {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    cursor: pointer;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    border: 1px solid transparent;
    transition: all var(--transition-fast);
  }
  .mod-opt:hover {
    background: var(--color-overlay);
    border-color: var(--color-border);
  }
  .mod-opt--active {
    background: var(--color-accent-soft);
    border-color: var(--color-accent-ring);
  }
  .mod-opt input[type="radio"] {
    margin-top: 3px;
    accent-color: var(--color-accent);
  }
  .mod-opt__text { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .mod-opt__label { font-size: 13px; font-weight: 600; color: var(--color-text); }
  .mod-opt__desc { font-size: 12px; color: var(--color-text-secondary); line-height: 1.4; }
  .mod-offer__apply {
    align-self: flex-start;
    margin-top: 4px;
  }

  .done__danger {
    margin-top: 16px;
    padding-top: 16px;
    border-top: 1px dashed var(--color-border);
    width: 100%;
    max-width: 520px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }
</style>
