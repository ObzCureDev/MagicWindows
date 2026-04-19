<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import type { ModifierState, ModifierToggles, RawScancodePair } from "../lib/types";

  let modState = $state<ModifierState | null>(null);
  let loading = $state(true);
  let toggles = $state<ModifierToggles>({
    swapCmdCtrlLeft: false,
    swapCmdCtrlRight: false,
    capsToCtrl: false,
    swapOptionCmd: false,
  });
  let phase = $state<"select" | "preview" | "applying">("select");
  let error = $state<string | null>(null);
  let success = $state(false);
  let showExternalDetails = $state(false);

  let bothSides = $derived(toggles.swapCmdCtrlLeft && toggles.swapCmdCtrlRight);
  let cmdCtrlActive = $derived(toggles.swapCmdCtrlLeft || toggles.swapCmdCtrlRight);

  async function load() {
    loading = true;
    error = null;
    try {
      const s = await invoke<ModifierState>("read_scancode_map");
      modState = s;
      toggles = { ...s.current };
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function setBothSides(v: boolean) {
    toggles.swapCmdCtrlLeft  = v;
    toggles.swapCmdCtrlRight = v;
    if (v) toggles.swapOptionCmd = false;
  }

  function setSwapOptionCmd(v: boolean) {
    toggles.swapOptionCmd = v;
    if (v) {
      toggles.swapCmdCtrlLeft  = false;
      toggles.swapCmdCtrlRight = false;
    }
  }

  function toPreview() {
    error = null;
    phase = "preview";
  }

  async function apply() {
    phase = "applying";
    error = null;
    try {
      await invoke("write_scancode_map", { toggles });
      success = true;
      await load();
      phase = "select";
    } catch (e) {
      error = String(e);
      phase = "preview";
    }
  }

  async function disableAll() {
    phase = "applying";
    error = null;
    try {
      await invoke("clear_scancode_map");
      success = true;
      await load();
      phase = "select";
    } catch (e) {
      error = String(e);
      phase = "select";
    }
  }

  function back() {
    appState.page = "welcome";
  }

  function pairsForCurrent(): RawScancodePair[] {
    return modState?.rawEntries ?? [];
  }
  function describePair(p: RawScancodePair): string {
    const labels: Record<string, string> = {
      "1D00": "LCtrl",
      "1DE0": "RCtrl",
      "5BE0": "LWin (Cmd)",
      "5CE0": "RWin (Cmd)",
      "3800": "LAlt (Option)",
      "38E0": "RAlt (Option)",
      "3A00": "CapsLock",
    };
    const o = labels[p.oldCode] ?? p.oldCode;
    const n = labels[p.newCode] ?? p.newCode;
    return `${o} → ${n}`;
  }

  function pair(newCode: string, oldCode: string): RawScancodePair {
    return { newCode, oldCode };
  }
  function previewPairs(t: ModifierToggles): RawScancodePair[] {
    const r: RawScancodePair[] = [];
    if (t.swapCmdCtrlLeft) {
      r.push(pair("1D00", "5BE0"));
      r.push(pair("5BE0", "1D00"));
    }
    if (t.swapCmdCtrlRight) {
      r.push(pair("1DE0", "5CE0"));
      r.push(pair("5CE0", "1DE0"));
    }
    if (t.capsToCtrl) r.push(pair("1D00", "3A00"));
    if (t.swapOptionCmd) {
      r.push(pair("3800", "5BE0"));
      r.push(pair("5BE0", "3800"));
      r.push(pair("38E0", "5CE0"));
      r.push(pair("5CE0", "38E0"));
    }
    return r;
  }

  onMount(load);
</script>

<div class="page modifiers">
  <div class="page__header">
    <p class="modifiers__eyebrow">{t(appState.lang, "ui.system")}</p>
    <h1 class="page__title">{t(appState.lang, "modifiers.title")}</h1>
    <p class="page__subtitle">{t(appState.lang, "modifiers.description")}</p>
  </div>

  <div class="page__content">
    {#if loading}
      <div class="spinner"></div>
    {:else if phase === "select"}
      {#if modState?.hasExternalMappings}
        <div class="status status--warning" role="alert">
          {t(appState.lang, "modifiers.externalWarning")}
          <button class="link" onclick={() => (showExternalDetails = !showExternalDetails)}>
            {t(appState.lang, "modifiers.externalDetails")}
          </button>
          {#if showExternalDetails && modState}
            <ul class="raw-pairs">
              {#each modState.rawEntries as p}
                <li><code>{describePair(p)}</code></li>
              {/each}
            </ul>
          {/if}
        </div>
      {/if}

      {#if success}
        <div class="status status--success">{t(appState.lang, "modifiers.applied")}</div>
      {/if}

      <div class="toggle-list">
        <label class="toggle-row" class:toggle-row--active={bothSides}>
          <input type="checkbox"
                 checked={bothSides}
                 disabled={toggles.swapOptionCmd}
                 onchange={(e) => setBothSides((e.currentTarget as HTMLInputElement).checked)} />
          <div class="toggle-text">
            <div class="toggle-label">{t(appState.lang, "modifiers.toggleSwapBoth")}</div>
            <div class="toggle-hint">{t(appState.lang, "modifiers.toggleSwapBothHint")}</div>
          </div>
        </label>

        <label class="toggle-row" class:toggle-row--active={toggles.swapCmdCtrlLeft}>
          <input type="checkbox"
                 bind:checked={toggles.swapCmdCtrlLeft}
                 disabled={toggles.swapOptionCmd} />
          <div class="toggle-text">
            <div class="toggle-label">{t(appState.lang, "modifiers.toggleSwapLeft")}</div>
          </div>
        </label>

        <label class="toggle-row" class:toggle-row--active={toggles.swapCmdCtrlRight}>
          <input type="checkbox"
                 bind:checked={toggles.swapCmdCtrlRight}
                 disabled={toggles.swapOptionCmd} />
          <div class="toggle-text">
            <div class="toggle-label">{t(appState.lang, "modifiers.toggleSwapRight")}</div>
          </div>
        </label>

        <label class="toggle-row" class:toggle-row--active={toggles.capsToCtrl}>
          <input type="checkbox" bind:checked={toggles.capsToCtrl} />
          <div class="toggle-text">
            <div class="toggle-label">{t(appState.lang, "modifiers.toggleCaps")}</div>
            <div class="toggle-hint">{t(appState.lang, "modifiers.toggleCapsHint")}</div>
          </div>
        </label>

        <label class="toggle-row" class:toggle-row--active={toggles.swapOptionCmd}>
          <input type="checkbox"
                 checked={toggles.swapOptionCmd}
                 disabled={cmdCtrlActive}
                 onchange={(e) => setSwapOptionCmd((e.currentTarget as HTMLInputElement).checked)} />
          <div class="toggle-text">
            <div class="toggle-label">{t(appState.lang, "modifiers.toggleOptionCmd")}</div>
            <div class="toggle-hint">{t(appState.lang, "modifiers.toggleOptionCmdHint")}</div>
          </div>
        </label>
      </div>

      {#if error}<div class="status status--error">{error}</div>{/if}

      <div class="page__actions">
        <button class="btn btn-primary" onclick={toPreview}>
          {t(appState.lang, "modifiers.preview")}
        </button>
        <button class="btn btn-danger" onclick={disableAll}>
          {t(appState.lang, "modifiers.disableAll")}
        </button>
        <button class="btn btn-secondary" onclick={back}>
          {t(appState.lang, "modifiers.back")}
        </button>
      </div>

    {:else if phase === "preview" || phase === "applying"}
      <h2 class="modifiers__preview-title">{t(appState.lang, "modifiers.previewTitle")}</h2>

      <div class="preview-grid">
        <div class="preview-col">
          <h3 class="preview-col__title">{t(appState.lang, "modifiers.previewBefore")}</h3>
          {#if pairsForCurrent().length === 0}
            <p class="preview-col__empty">{t(appState.lang, "modifiers.previewNoChange")}</p>
          {:else}
            <ul class="raw-pairs">
              {#each pairsForCurrent() as p}<li><code>{describePair(p)}</code></li>{/each}
            </ul>
          {/if}
        </div>

        <div class="preview-col preview-col--after">
          <h3 class="preview-col__title">{t(appState.lang, "modifiers.previewAfter")}</h3>
          {#if previewPairs(toggles).length === 0}
            <p class="preview-col__empty">{t(appState.lang, "modifiers.previewNoChange")}</p>
          {:else}
            <ul class="raw-pairs">
              {#each previewPairs(toggles) as p}<li><code>{describePair(p)}</code></li>{/each}
            </ul>
          {/if}
        </div>
      </div>

      <div class="status status--warning">{t(appState.lang, "modifiers.rebootWarning")}</div>

      {#if error}<div class="status status--error">{error}</div>{/if}

      <div class="page__actions">
        <button class="btn btn-primary"
                disabled={phase === "applying"}
                onclick={apply}>
          {phase === "applying" ? t(appState.lang, "modifiers.applying") : t(appState.lang, "modifiers.apply")}
        </button>
        <button class="btn btn-secondary"
                disabled={phase === "applying"}
                onclick={() => (phase = "select")}>
          {t(appState.lang, "modifiers.cancel")}
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .modifiers__eyebrow {
    margin: 0;
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .toggle-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    max-width: 580px;
    margin: 4px auto;
  }
  .toggle-row {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 12px 14px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-card);
    cursor: pointer;
    transition: all var(--transition-fast);
    box-shadow: var(--shadow-xs);
  }
  .toggle-row:hover {
    border-color: var(--color-border-strong);
    background: var(--color-bg-card-hover);
  }
  .toggle-row--active {
    background: var(--color-accent-soft);
    border-color: var(--color-accent-ring);
    box-shadow: 0 0 0 1px var(--color-accent-ring);
  }
  .toggle-row input[type="checkbox"] {
    margin-top: 3px;
    width: 16px;
    height: 16px;
    accent-color: var(--color-accent);
    cursor: pointer;
  }
  .toggle-text {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .toggle-label {
    font-weight: 600;
    font-size: 14px;
    color: var(--color-text);
  }
  .toggle-hint {
    font-size: 12px;
    color: var(--color-text-secondary);
    line-height: 1.4;
  }

  .modifiers__preview-title {
    margin: 0 0 8px;
    font-family: var(--font-display);
    font-style: italic;
    font-weight: 400;
    font-variation-settings: "opsz" 144;
    font-size: 22px;
    color: var(--color-text-strong);
  }
  .preview-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
    width: 100%;
    max-width: 720px;
    margin: 0 auto;
  }
  .preview-col {
    padding: 14px 16px;
    background: var(--color-bg-card);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
  }
  .preview-col--after {
    background: var(--color-accent-soft);
    border-color: var(--color-accent-ring);
  }
  .preview-col__title {
    margin: 0 0 8px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }
  .preview-col__empty {
    margin: 0;
    font-size: 12px;
    color: var(--color-text-muted);
  }
  .raw-pairs {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .raw-pairs li {
    padding: 0;
  }
  .raw-pairs code {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--color-text);
  }
  .link {
    background: none;
    border: none;
    color: var(--color-accent);
    text-decoration: underline;
    cursor: pointer;
    padding: 0;
    margin-left: 8px;
    font: inherit;
  }
</style>
