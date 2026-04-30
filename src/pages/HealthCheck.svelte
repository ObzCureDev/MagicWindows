<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import KeyboardVisual from "../components/KeyboardVisual.svelte";
  import type { Layout, KeyStatus, HealthCheckSession, ModState } from "../lib/types";
  import { expectedCodepointFor, compareKeystroke } from "../lib/healthCheck";
  import { SCANCODE_BY_EVENT_CODE } from "../lib/scancode";

  // Page reads its target from appState. Settings is the only entry point
  // and is responsible for setting healthCheckTarget before navigating.
  let target = $derived(appState.healthCheckTarget);
  let layout = $state<Layout | null>(null);

  // If we land here without a target (deep-link / refresh), bounce to Settings.
  $effect(() => {
    if (!target) {
      appState.page = "settings";
    }
  });

  let session = $state<HealthCheckSession>({
    layoutId: "",
    klid: "",
    startedAt: new Date().toISOString(),
    results: [],
  });

  onMount(async () => {
    if (!target) return;
    session.layoutId = target.layoutId;
    session.klid = target.klid;
    try {
      // Same loader pattern used by Test.svelte / Preview.svelte / Detect.svelte.
      layout = await invoke<Layout>("get_layout", { id: target.layoutId });
    } catch (e) {
      console.error("Failed to load layout for health check:", e);
    }
  });

  let keyStatus = $derived.by(() => {
    const map: Record<string, KeyStatus> = {};
    for (const r of session.results) {
      const prev = map[r.scancode];
      if (r.status === "failed") map[r.scancode] = "failed";
      else if (prev !== "failed") map[r.scancode] = r.status;
    }
    return map;
  });

  let summary = $derived.by(() => {
    let passed = 0;
    let failed = 0;
    for (const v of Object.values(keyStatus)) {
      if (v === "passed") passed++;
      else if (v === "failed") failed++;
    }
    const total = layout ? Object.keys(layout.keys).length : 0;
    return { passed, failed, untested: Math.max(0, total - passed - failed) };
  });

  function onKeydown(e: KeyboardEvent) {
    if (!layout) return;
    const sc = SCANCODE_BY_EVENT_CODE[e.code];
    if (!sc) return;
    e.preventDefault();
    const mods: ModState = {
      shift: e.shiftKey,
      altgr: e.getModifierState("AltGraph"),
      capsLock: e.getModifierState("CapsLock"),
    };
    const expected = expectedCodepointFor(layout, sc, mods);
    if (expected === null) return;
    const received = e.key.length === 1 ? e.key : "";
    const status = compareKeystroke(expected, received);
    session.results = [
      ...session.results,
      {
        scancode: sc,
        modifiers: mods,
        expectedCodepoint: expected,
        receivedChar: received,
        status,
        at: Date.now() - new Date(session.startedAt).getTime(),
      },
    ];
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="health-check">
  {#if layout}
    <header class="hc-top">
      <div class="hc-top__text">
        <h1 class="hc-top__title">{t(appState.lang, "healthCheck.title")}</h1>
        <p class="hc-top__hint">{t(appState.lang, "healthCheck.subtitle")}</p>
      </div>
      <button class="btn btn-secondary btn-sm" onclick={() => (appState.page = "settings")}>
        {t(appState.lang, "healthCheck.back")}
      </button>
    </header>

    <div class="hc-kbd">
      <KeyboardVisual {layout} activeLayer="base" {keyStatus} />
    </div>

    <p class="hc-summary">
      {t(appState.lang, "healthCheck.summary")
        .replace("{passed}", String(summary.passed))
        .replace("{failed}", String(summary.failed))
        .replace("{untested}", String(summary.untested))}
    </p>
  {:else}
    <div class="hc-center"><div class="spinner"></div></div>
  {/if}
</div>

<style>
  .health-check {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px 24px 22px;
    box-sizing: border-box;
    height: 100%;
    overflow: hidden;
  }
  .hc-center {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .hc-top {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
    flex-shrink: 0;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--color-border);
  }
  .hc-top__text { min-width: 0; }
  .hc-top__title {
    margin: 0;
    font-family: var(--font-display);
    font-style: italic;
    font-weight: 400;
    font-variation-settings: "opsz" 144;
    font-size: 24px;
    line-height: 1.1;
    letter-spacing: -0.02em;
    color: var(--color-text-strong);
  }
  .hc-top__hint {
    margin: 4px 0 0;
    font-size: 13px;
    color: var(--color-text-secondary);
    line-height: 1.4;
    max-width: 540px;
  }
  .hc-kbd {
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    --u: 32px;
    --gap: 4px;
    --radius-key: 6px;
  }
  .hc-kbd :global(.kbd-body) {
    --u: 32px;
    --gap: 4px;
    --radius-key: 6px;
  }
  .hc-summary {
    margin: 0;
    flex-shrink: 0;
    text-align: center;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--color-text-secondary);
  }
</style>
