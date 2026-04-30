<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getVersion } from "@tauri-apps/api/app";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import { KeyboardVisual } from "@magicwindows/keyboard-visual";
  import type { Layout, KeyStatus, HealthCheckSession, ModState } from "../lib/types";
  import { expectedCodepointFor, compareKeystroke } from "../lib/healthCheck";
  import { SCANCODE_BY_EVENT_CODE } from "../lib/scancode";

  interface ControlKeyResult {
    name: string;
    vk: number;
    shift: boolean;
    passed: boolean;
    produced: string;
  }
  interface ControlKeyReport {
    klid: string;
    results: ControlKeyResult[];
    all_passed: boolean;
  }

  let controlReport = $state<ControlKeyReport | null>(null);
  let controlError = $state<string | null>(null);

  async function runControlKeyCheck() {
    if (!target) return;
    controlError = null;
    controlReport = null;
    try {
      controlReport = await invoke<ControlKeyReport>("health_check_control_keys", {
        klid: target.klid,
      });
    } catch (e) {
      controlError = String(e);
    }
  }

  // The Rust probe returns stable identifiers ("enter", "shift_enter", …)
  // and we localise them here so the FR UI doesn't end up with mixed-language
  // sentences. Falls back to the raw identifier when the i18n key is missing.
  function controlKeyLabel(name: string): string {
    return t(appState.lang, `healthCheck.controlKey.${name}`);
  }

  async function exportReport() {
    const appVersion = await getVersion().catch(() => "unknown");
    const payload = {
      appVersion,
      exportedAt: new Date().toISOString(),
      session,
      controlReport,
      controlError,
    };
    const blob = new Blob([JSON.stringify(payload, null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `magicwindows-health-${session.layoutId}-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

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

  // Initialize synchronously from the derived `target` so we never go through
  // an empty-string state. The `$effect` above bounces to Settings if
  // `target` is null, so the `?? ""` fallbacks are defence-in-depth only —
  // we intentionally capture the initial value here, not a reactive ref.
  /* svelte-ignore state_referenced_locally */
  let session = $state<HealthCheckSession>({
    layoutId: target?.layoutId ?? "",
    /* svelte-ignore state_referenced_locally */
    klid: target?.klid ?? "",
    startedAt: new Date().toISOString(),
    results: [],
  });

  onMount(async () => {
    if (!target) return;
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

    <section class="control-keys">
      <button class="btn btn-secondary btn-sm" onclick={runControlKeyCheck}>
        {t(appState.lang, "healthCheck.runControlKeys")}
      </button>

      {#if controlReport}
        {#if controlReport.all_passed}
          <p class="ok">{t(appState.lang, "healthCheck.controlKeysOk")}</p>
        {:else}
          <p class="fail">
            {t(appState.lang, "healthCheck.controlKeysFail").replace(
              "{keys}",
              controlReport.results
                .filter((r) => !r.passed)
                .map((r) => controlKeyLabel(r.name))
                .join(", ")
            )}
          </p>
        {/if}
        <ul class="control-results">
          {#each controlReport.results as r}
            <li class={r.passed ? "ok" : "fail"}>
              <strong>{controlKeyLabel(r.name)}</strong> — {r.passed ? "OK" : `produced ${r.produced}`}
            </li>
          {/each}
        </ul>
      {/if}

      {#if controlError}
        <p class="fail">{controlError}</p>
      {/if}
    </section>

    <div class="hc-actions">
      <button class="btn btn-secondary btn-sm" onclick={exportReport}>
        {t(appState.lang, "healthCheck.exportReport")}
      </button>
    </div>
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
  .control-keys {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: flex-start;
  }
  .control-keys .ok {
    margin: 0;
    color: var(--color-success);
    font-size: 13px;
  }
  .control-keys .fail {
    margin: 0;
    color: var(--color-danger);
    font-size: 13px;
  }
  .control-results {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-wrap: wrap;
    gap: 6px 12px;
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .control-results li.ok {
    color: var(--color-success);
  }
  .control-results li.fail {
    color: var(--color-danger);
  }
  .hc-actions {
    flex-shrink: 0;
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
