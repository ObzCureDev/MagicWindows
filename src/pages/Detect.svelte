<script lang="ts">
  import { onMount } from "svelte";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import {
    pickBestQuestion,
    applyResponse,
    isExpectedPress,
    layoutsWithChar,
  } from "../lib/detection";
  import catalogueJson from "../lib/detection-catalogue.generated.json";
  import type { DetectionCatalogue, DetectionCharEntry } from "../lib/types";

  const catalogue = catalogueJson as DetectionCatalogue;
  const MAX_QUESTIONS = 3;
  const MAX_WRONG_PER_QUESTION = 3;
  const MODIFIER_CODES = new Set([
    "ShiftLeft", "ShiftRight",
    "AltLeft", "AltRight",
    "ControlLeft", "ControlRight",
    "MetaLeft", "MetaRight",
    "CapsLock", "Fn", "FnLock",
  ]);

  let candidates = $state<string[]>(appState.layouts.map((l) => l.id));
  let questionsAsked = $state(0);
  let wrongPresses = $state(0);
  let currentChar = $state<DetectionCharEntry | null>(null);
  let detectedId = $state<string | null>(null);
  let failed = $state(false);
  let showWrongToast = $state(false);
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  function nextQuestion() {
    if (candidates.length === 1) {
      finishSuccess(candidates[0]);
      return;
    }
    if (questionsAsked >= MAX_QUESTIONS) {
      finishFailure();
      return;
    }
    const q = pickBestQuestion(catalogue, candidates);
    if (!q) {
      finishFailure();
      return;
    }
    currentChar = q;
    wrongPresses = 0;
  }

  function finishSuccess(id: string) {
    detectedId = id;
    appState.selectedLayoutId = id;
  }

  function finishFailure() {
    failed = true;
    appState.detectionFailedMessage = t(appState.lang, "detect.failedBanner");
    appState.page = "select";
  }

  function flashWrongToast() {
    showWrongToast = true;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      showWrongToast = false;
      toastTimer = null;
    }, 2500);
  }

  function handleKeydown(event: KeyboardEvent) {
    if (!currentChar || detectedId || failed) return;
    if (event.repeat) return;
    if (MODIFIER_CODES.has(event.code)) return;
    if (event.code === "Escape") {
      appState.page = "welcome";
      return;
    }
    event.preventDefault();

    if (isExpectedPress(currentChar, candidates, event.code)) {
      const next = applyResponse(currentChar, candidates, { kind: "key_pressed", eventCode: event.code });
      candidates = next;
      questionsAsked += 1;
      currentChar = null;
      nextQuestion();
    } else {
      wrongPresses += 1;
      flashWrongToast();
    }
  }

  function clickNoKey() {
    if (!currentChar) return;
    const next = applyResponse(currentChar, candidates, { kind: "no_such_key" });
    candidates = next;
    questionsAsked += 1;
    currentChar = null;
    nextQuestion();
  }

  function pickManually() {
    appState.page = "select";
  }

  function goBack() {
    appState.page = "welcome";
  }

  function detectedName(id: string): string {
    const layout = appState.layouts.find((l) => l.id === id);
    return layout?.name[appState.lang] ?? id;
  }

  function goPreview() {
    appState.page = "preview";
  }

  // Helpers for the conditional "I don't have this key" button
  let showNoKeyButton = $derived(
    !!currentChar &&
    layoutsWithChar(currentChar, candidates).length < candidates.length
  );

  let progressPct = $derived((questionsAsked / MAX_QUESTIONS) * 100);

  onMount(() => {
    nextQuestion();
    window.addEventListener("keydown", handleKeydown);
    return () => {
      window.removeEventListener("keydown", handleKeydown);
      if (toastTimer) clearTimeout(toastTimer);
    };
  });
</script>

<div class="page">
  <div class="page__header">
    <h1 class="page__title">{t(appState.lang, "detect.title")}</h1>
  </div>

  <div class="page__content">
    {#if detectedId}
      <div class="status status--success">
        {t(appState.lang, "detect.detected", { name: detectedName(detectedId) })}
      </div>
      <div class="page__actions">
        <button class="btn btn-primary" onclick={goPreview}>
          {t(appState.lang, "detect.continue")}
        </button>
      </div>

      <div class="mk-body mk-body--neutral" aria-hidden="true">
        <div class="mk-row mk-row--fn">
          <div class="mk-key mk-key--fn-esc"><span class="mk-lbl-sm">esc</span></div>
          {#each Array(12) as _, i}
            <div class="mk-key mk-key--fn"><span class="mk-lbl-fn">F{i + 1}</span></div>
          {/each}
          <div class="mk-touchid"></div>
        </div>
        <div class="mk-row">
          {#each Array(13) as _}<div class="mk-key"></div>{/each}
          <div class="mk-key mk-key--delete"><span class="mk-lbl-sm">delete</span></div>
        </div>
        <div class="mk-row">
          <div class="mk-key mk-key--tab"><span class="mk-lbl-sm">tab</span></div>
          {#each Array(12) as _}<div class="mk-key"></div>{/each}
          <div class="mk-key mk-key--enter-top"></div>
        </div>
        <div class="mk-row">
          <div class="mk-key mk-key--caps"><span class="mk-lbl-sm">caps lock</span></div>
          {#each Array(12) as _}<div class="mk-key"></div>{/each}
          <div class="mk-key mk-key--enter-bot"><span class="mk-lbl-sm">return</span></div>
        </div>
        <div class="mk-row">
          <div class="mk-key mk-key--lshift"><span class="mk-lbl-sm">shift</span></div>
          {#each Array(11) as _}<div class="mk-key"></div>{/each}
          <div class="mk-key mk-key--rshift"><span class="mk-lbl-sm">shift</span></div>
        </div>
        <div class="mk-row mk-row--bottom">
          <div class="mk-key mk-key--mod1"><span class="mk-lbl-xs">fn</span></div>
          <div class="mk-key mk-key--mod1"><span class="mk-lbl-xs">control</span></div>
          <div class="mk-key mk-key--mod1"><span class="mk-lbl-xs">option</span></div>
          <div class="mk-key mk-key--cmd"><span class="mk-lbl-xs">command</span></div>
          <div class="mk-key mk-key--space"></div>
          <div class="mk-key mk-key--cmd"><span class="mk-lbl-xs">command</span></div>
          <div class="mk-key mk-key--mod1"><span class="mk-lbl-xs">option</span></div>
          <div class="mk-arrows">
            <div class="mk-arrow mk-arrow--l"><span class="mk-arrow-glyph">◀</span></div>
            <div class="mk-arrow-stack">
              <div class="mk-arrow mk-arrow--h"><span class="mk-arrow-glyph">▲</span></div>
              <div class="mk-arrow mk-arrow--h"><span class="mk-arrow-glyph">▼</span></div>
            </div>
            <div class="mk-arrow mk-arrow--l"><span class="mk-arrow-glyph">▶</span></div>
          </div>
        </div>
      </div>
    {:else if currentChar}
      <div
        class="progress-bar"
        role="progressbar"
        aria-valuenow={Math.round(progressPct)}
        aria-valuemin={0}
        aria-valuemax={100}
      >
        <div class="progress-bar__fill" style="width: {progressPct}%"></div>
      </div>

      {#if showWrongToast}
        <div class="status status--error" role="alert">
          {t(appState.lang, "detect.wrongKey")}
        </div>
      {/if}

      {#if wrongPresses >= MAX_WRONG_PER_QUESTION}
        <div class="status status--info" role="status">
          {t(appState.lang, "detect.wrongKeyHelp")}
        </div>
      {/if}

      <div class="detect-prompt">
        <p class="detect-prompt__text">{t(appState.lang, "detect.charPrompt")}</p>
        <div class="detect-prompt__char">{currentChar.char}</div>
        <p class="text-secondary">{t(appState.lang, "detect.charHint")}</p>
      </div>

      <div class="page__actions">
        {#if showNoKeyButton}
          <button class="btn btn-secondary" onclick={clickNoKey}>
            {t(appState.lang, "detect.noKey")}
          </button>
        {/if}
        <button class="btn btn-secondary" onclick={pickManually}>
          {t(appState.lang, "detect.manual")}
        </button>
        <button class="btn btn-secondary" onclick={goBack}>
          {t(appState.lang, "detect.back")}
        </button>
      </div>
    {:else}
      <div class="spinner"></div>
    {/if}
  </div>
</div>

<style>
  .detect-prompt {
    text-align: center;
    margin: 2rem auto;
    max-width: 480px;
  }
  .detect-prompt__text {
    font-size: 1rem;
    color: var(--color-text-secondary);
    margin-bottom: 1.5rem;
  }
  .detect-prompt__char {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 120px;
    height: 120px;
    margin: 0 auto 1.5rem;
    font-size: 64px;
    font-weight: 400;
    color: var(--color-text);
    background: linear-gradient(180deg, #ffffff 0%, #f0f0f4 100%);
    border-radius: 18px;
    border: 1px solid rgba(0,0,0,0.08);
    box-shadow:
      0 0.5px 0 rgba(255,255,255,0.95) inset,
      0 4px 14px rgba(0,0,0,0.10);
  }
  :root[data-theme="dark"] .detect-prompt__char,
  :root:not([data-theme="light"]) .detect-prompt__char {
    background: linear-gradient(180deg, #5a5a5e 0%, #4a4a4c 100%);
    border-color: rgba(0,0,0,0.45);
    color: #f5f5f7;
  }
</style>
