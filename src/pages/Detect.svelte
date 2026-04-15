<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores";
  import { t } from "../lib/i18n";
  import type { DetectionKey, DetectionResult } from "../lib/types";

  let detectionKeys = $state<DetectionKey[]>([]);
  let currentStep = $state(0);
  let results = $state<DetectionResult[]>([]);
  let analyzing = $state(false);
  let detectedId = $state<string | null>(null);
  let detectedName = $state<string>("");
  let failed = $state(false);
  let waiting = $state(true);

  // Load detection keys from the first available layout (they share detection keys)
  onMount(async () => {
    try {
      const layoutIds = appState.layouts.map((l) => l.id);
      const keys = await invoke<DetectionKey[]>("get_detection_keys", { layoutIds });
      detectionKeys = keys;
      waiting = false;
    } catch (err) {
      console.error("Failed to get detection keys:", err);
      // Fallback: extract from layouts if command not available
      appState.error = String(err);
      waiting = false;
    }
  });

  let totalKeys = $derived(detectionKeys.length);
  let currentKey = $derived(detectionKeys[currentStep] ?? null);
  let progress = $derived(totalKeys > 0 ? ((currentStep) / totalKeys) * 100 : 0);

  function handleKeydown(event: KeyboardEvent) {
    if (analyzing || failed || detectedId || !currentKey) return;

    // Only capture if this is the expected event code
    if (event.code === currentKey.eventCode) {
      event.preventDefault();

      const result: DetectionResult = {
        eventCode: event.code,
        receivedChar: event.key,
      };
      results = [...results, result];

      if (currentStep + 1 < totalKeys) {
        currentStep += 1;
      } else {
        runDetection();
      }
    }
  }

  async function runDetection() {
    analyzing = true;
    try {
      const matchId = await invoke<string | null>("match_detection", {
        results: results,
      });
      if (matchId) {
        detectedId = matchId;
        appState.selectedLayoutId = matchId;
        // Find the name
        const layout = appState.layouts.find((l) => l.id === matchId);
        detectedName = layout?.name[appState.lang] ?? matchId;
      } else {
        failed = true;
      }
    } catch (err) {
      console.error("Detection matching failed:", err);
      failed = true;
    } finally {
      analyzing = false;
    }
  }

  function retry() {
    currentStep = 0;
    results = [];
    detectedId = null;
    detectedName = "";
    failed = false;
    analyzing = false;
  }

  function goPreview() {
    if (detectedId) {
      appState.page = "preview";
    }
  }

  function goSelect() {
    appState.page = "select";
  }

  function goBack() {
    appState.page = "welcome";
  }

  let keydownHandler: (e: KeyboardEvent) => void;

  onMount(() => {
    keydownHandler = handleKeydown;
    window.addEventListener("keydown", keydownHandler);
  });

  onDestroy(() => {
    if (keydownHandler) {
      window.removeEventListener("keydown", keydownHandler);
    }
  });
</script>

<div class="page">
  <div class="page__header">
    <h1 class="page__title">{t(appState.lang, "detect.title")}</h1>
    <p class="page__subtitle">{t(appState.lang, "detect.instruction")}</p>
  </div>

  <div class="page__content">
    {#if waiting}
      <div class="spinner"></div>
      <p class="text-secondary">{t(appState.lang, "common.loading")}</p>
    {:else if analyzing}
      <div class="spinner"></div>
      <p class="text-secondary">{t(appState.lang, "detect.analyzing")}</p>
    {:else if detectedId}
      <div class="status status--success">
        {t(appState.lang, "detect.detected", { name: detectedName })}
      </div>
      <div class="page__actions">
        <button class="btn btn-primary" onclick={goPreview}>
          {t(appState.lang, "detect.continue")}
        </button>
      </div>
    {:else if failed}
      <div class="status status--error">
        {t(appState.lang, "detect.notDetected")}
      </div>
      <div class="page__actions">
        <button class="btn btn-secondary" onclick={retry}>
          {t(appState.lang, "detect.tryAgain")}
        </button>
        <button class="btn btn-primary" onclick={goSelect}>
          {t(appState.lang, "detect.fallback")}
        </button>
      </div>
    {:else if currentKey}
      <div class="progress-bar">
        <div class="progress-bar__fill" style="width: {progress}%"></div>
      </div>
      <p class="text-secondary">
        {t(appState.lang, "detect.progress", {
          current: String(currentStep + 1),
          total: String(totalKeys),
        })}
      </p>

      <div class="detect-prompt">
        <p class="detect-prompt__text">{t(appState.lang, "detect.pressKey")}</p>
        <p class="detect-prompt__key-name">
          {currentKey.prompt[appState.lang] ?? currentKey.prompt["en"] ?? currentKey.eventCode}
        </p>
      </div>

      <!-- Simplified keyboard hint showing which area to press -->
      <div class="keyboard-container">
        <div class="keyboard-row">
          {#each ["Backquote", "Digit1", "Digit2", "Digit3", "Digit4", "Digit5", "Digit6", "Digit7", "Digit8", "Digit9", "Digit0", "Minus", "Equal"] as code}
            <div
              class="key"
              class:key--active={currentKey.eventCode === code}
            >
              <span class="key__label">
                {#if code === "Backquote"}`
                {:else if code.startsWith("Digit")}{code.slice(5)}
                {:else if code === "Minus"}-
                {:else if code === "Equal"}=
                {/if}
              </span>
            </div>
          {/each}
        </div>
        <div class="keyboard-row">
          <div class="key" style="min-width: 62px"><span class="key__label">Tab</span></div>
          {#each ["KeyQ", "KeyW", "KeyE", "KeyR", "KeyT", "KeyY", "KeyU", "KeyI", "KeyO", "KeyP", "BracketLeft", "BracketRight"] as code}
            <div
              class="key"
              class:key--active={currentKey.eventCode === code}
            >
              <span class="key__label">
                {code.startsWith("Key") ? code.slice(3) : code === "BracketLeft" ? "[" : "]"}
              </span>
            </div>
          {/each}
        </div>
        <div class="keyboard-row">
          <div class="key" style="min-width: 72px"><span class="key__label">Caps</span></div>
          {#each ["KeyA", "KeyS", "KeyD", "KeyF", "KeyG", "KeyH", "KeyJ", "KeyK", "KeyL", "Semicolon", "Quote", "Backslash"] as code}
            <div
              class="key"
              class:key--active={currentKey.eventCode === code}
            >
              <span class="key__label">
                {#if code.startsWith("Key")}{code.slice(3)}
                {:else if code === "Semicolon"};
                {:else if code === "Quote"}'
                {:else if code === "Backslash"}\
                {/if}
              </span>
            </div>
          {/each}
        </div>
        <div class="keyboard-row">
          <div class="key" style="min-width: 52px"><span class="key__label">Shift</span></div>
          {#each ["IntlBackslash", "KeyZ", "KeyX", "KeyC", "KeyV", "KeyB", "KeyN", "KeyM", "Comma", "Period", "Slash"] as code}
            <div
              class="key"
              class:key--active={currentKey.eventCode === code}
            >
              <span class="key__label">
                {#if code.startsWith("Key")}{code.slice(3)}
                {:else if code === "IntlBackslash"}{"<"}
                {:else if code === "Comma"},
                {:else if code === "Period"}.
                {:else if code === "Slash"}/
                {/if}
              </span>
            </div>
          {/each}
          <div class="key" style="min-width: 52px"><span class="key__label">Shift</span></div>
        </div>
      </div>
    {:else}
      <div class="status status--error">
        {t(appState.lang, "detect.notDetected")}
      </div>
    {/if}

    <div class="page__actions">
      <button class="btn btn-secondary" onclick={goBack}>
        {t(appState.lang, "detect.back")}
      </button>
    </div>
  </div>
</div>
