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

  // "Remove Windows default" offer for the same locale we just installed.
  let windowsDefaultTarget = $state<InstalledLayoutInfo | null>(null);
  let windowsDefaultRemoved = $state(false);
  let removeWinError = $state<string | null>(null);
  let removeWinAttempt = $state(0);

  async function detectWindowsDefault() {
    if (!appState.selectedLayoutId) return;
    try {
      // Get the localeId (8 hex chars, e.g. "0000040c") of the layout we just installed.
      const installedLayout = await invoke<Layout>("get_layout", { id: appState.selectedLayoutId });
      const localeHex = installedLayout.localeId.slice(-4).toLowerCase();

      // Find the system default layout (klid starts with "0000") matching that locale.
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

  // Preset → ModifierToggles mapping. Mac shortcuts swaps both sides of
  // Cmd ↔ Ctrl so Cmd+C/V/X behaves like macOS. Windows-strict keeps Ctrl
  // at its Windows position but swaps Option ↔ Cmd so the physical key
  // order matches a PC keyboard (Ctrl-Win-Alt from the spacebar outward).
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

<div class="page">
  <div class="page__content">
    <div class="checkmark">&#10003;</div>

    <div class="page__header">
      <h1 class="page__title">{t(appState.lang, "done.title")}</h1>
      <p class="page__subtitle">{t(appState.lang, "done.congratulations")}</p>
    </div>

    <div class="status status--info" style="max-width: 460px;">
      {t(appState.lang, "done.switchInfo")}
    </div>

    <p class="text-secondary text-center" style="max-width: 460px;">
      {t(appState.lang, "done.instructions")}
    </p>

    <!-- ── Optional: remove the Windows default layout for this locale ── -->
    {#if windowsDefaultTarget}
      <div class="win-default-offer">
        <h2 class="win-default-offer__title">
          <span>{t(appState.lang, "done.removeWindowsDefault", { locale: windowsDefaultTarget.layoutText })}</span>
          <span
            class="info-icon"
            title={t(appState.lang, "settings.reactivateInfo")}
            aria-label={t(appState.lang, "settings.reactivateInfo")}
          >&#9432;</span>
        </h2>
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
      </div>
    {:else if windowsDefaultRemoved}
      <div class="status status--success" style="max-width: 460px;">
        {t(appState.lang, "done.windowsDefaultRemoved")}
      </div>
    {/if}

    <!-- ── Optional Mac-style modifier remap ─────────────────────────── -->
    <div class="mod-offer">
      <h2 class="mod-offer__title">{t(appState.lang, "done.modOfferTitle")}</h2>
      <p class="mod-offer__hint">{t(appState.lang, "done.modOfferHint")}</p>

      <label class="mod-opt">
        <input type="radio" name="mod-preset" value="none" bind:group={selectedPreset} disabled={applied} />
        <div class="mod-opt__text">
          <span class="mod-opt__label">{t(appState.lang, "done.modNone")}</span>
        </div>
      </label>
      <label class="mod-opt">
        <input type="radio" name="mod-preset" value="macShortcuts" bind:group={selectedPreset} disabled={applied} />
        <div class="mod-opt__text">
          <span class="mod-opt__label">{t(appState.lang, "done.modMac")}</span>
          <span class="mod-opt__desc">{t(appState.lang, "done.modMacDesc")}</span>
        </div>
      </label>
      <label class="mod-opt">
        <input type="radio" name="mod-preset" value="winStrict" bind:group={selectedPreset} disabled={applied} />
        <div class="mod-opt__text">
          <span class="mod-opt__label">{t(appState.lang, "done.modWin")}</span>
          <span class="mod-opt__desc">{t(appState.lang, "done.modWinDesc")}</span>
        </div>
      </label>

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
    </div>

    <div class="page__actions">
      <button class="btn btn-secondary" onclick={openSettings}>
        {t(appState.lang, "install.openSettings")}
      </button>
      <button class="btn btn-primary" onclick={close}>
        {t(appState.lang, "done.close")}
      </button>
    </div>

    <div class="mt-4">
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
  .win-default-offer {
    width: 100%;
    max-width: 480px;
    margin: 12px auto 4px;
    padding: 14px 16px;
    box-sizing: border-box;
    background: var(--color-bg-elevated, rgba(255,255,255,0.04));
    border: 1px solid var(--color-border, rgba(0,0,0,0.15));
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .win-default-offer__title {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 600;
    line-height: 1.35;
    display: flex;
    gap: 6px;
    align-items: flex-start;
  }
  .info-icon {
    cursor: help;
    color: var(--color-text-secondary);
    flex-shrink: 0;
  }
  .mod-offer {
    width: 100%;
    max-width: 480px;
    margin: 12px auto 4px;
    padding: 14px 16px;
    box-sizing: border-box;
    background: var(--color-bg-elevated, rgba(255,255,255,0.04));
    border: 1px solid var(--color-border, rgba(0,0,0,0.15));
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .mod-offer__title {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
  }
  .mod-offer__hint {
    margin: 0 0 4px;
    font-size: 0.78rem;
    color: var(--color-text-secondary);
    line-height: 1.35;
  }
  .mod-opt {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    cursor: pointer;
    padding: 6px 4px;
    border-radius: 6px;
  }
  .mod-opt:hover { background: rgba(127,127,127,0.06); }
  .mod-opt input[type="radio"] { margin-top: 3px; }
  .mod-opt__text { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .mod-opt__label {
    font-size: 0.88rem;
    font-weight: 500;
  }
  .mod-opt__desc {
    font-size: 0.75rem;
    color: var(--color-text-secondary);
    line-height: 1.3;
  }
  .mod-offer__apply {
    align-self: flex-start;
    margin-top: 4px;
  }
</style>
