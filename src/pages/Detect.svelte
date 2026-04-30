<script lang="ts">
  import { onMount } from "svelte";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import { invoke } from "@tauri-apps/api/core";
  import {
    pickBestQuestion,
    applyResponse,
    isExpectedPress,
    layoutsWithChar,
  } from "../lib/detection";
  import catalogueJson from "../lib/detection-catalogue.generated.json";
  import type { DetectionCatalogue, DetectionCharEntry, Layout } from "../lib/types";
  import { KeyboardVisual } from "@magicwindows/keyboard-visual";

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
  let detectedLayout = $state<Layout | null>(null);
  let failed = $state(false);
  let showWrongToast = $state(false);
  let toastTimer: ReturnType<typeof setTimeout> | null = null;
  let askedChars = $state(new Set<string>());

  function pickQuestion(): DetectionCharEntry | null {
    if (questionsAsked === 0 && askedChars.size === 0) {
      const at = catalogue.characters.find((c) => c.char === "@");
      if (at) return at;
    }
    const filtered: DetectionCatalogue = {
      ...catalogue,
      characters: catalogue.characters.filter((c) => !askedChars.has(c.char)),
    };
    return pickBestQuestion(filtered, candidates);
  }

  function nextQuestion() {
    if (candidates.length === 1) {
      finishSuccess(candidates[0]);
      return;
    }
    if (questionsAsked >= MAX_QUESTIONS) {
      finishFailure();
      return;
    }
    const q = pickQuestion();
    if (!q) {
      finishFailure();
      return;
    }
    currentChar = q;
    wrongPresses = 0;
  }

  async function finishSuccess(id: string) {
    detectedId = id;
    appState.selectedLayoutId = id;
    try {
      detectedLayout = await invoke<Layout>("get_layout", { id });
    } catch (e) {
      console.error("Failed to load detected layout:", e);
    }
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
      askedChars.add(currentChar.char);
      askedChars = askedChars;
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
    const someAbsent = layoutsWithChar(currentChar, candidates).length < candidates.length;
    if (someAbsent) {
      candidates = applyResponse(currentChar, candidates, { kind: "no_such_key" });
      questionsAsked += 1;
    }
    askedChars.add(currentChar.char);
    askedChars = askedChars;
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

  let showNoKeyButton = $derived(!!currentChar);

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

<div class="page detect">
  {#if detectedId}
    <div class="detect-success">
      <div class="status status--success">
        <strong>{t(appState.lang, "detect.detected", { name: detectedName(detectedId) })}</strong>
      </div>
      <button class="btn btn-primary btn-large" onclick={goPreview}>
        {t(appState.lang, "detect.continue")}
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M5 12h14M13 6l6 6-6 6" />
        </svg>
      </button>
    </div>

    {#if detectedLayout}
      <div class="detect-kbd">
        <KeyboardVisual layout={detectedLayout} activeLayer="base" />
      </div>
    {/if}
  {:else if currentChar}
    <div class="detect__header">
      <p class="detect__eyebrow">
        {t(appState.lang, "detect.title")}
      </p>
      <p class="detect__progress-label">
        {t(appState.lang, "detect.progress", {
          current: String(questionsAsked + 1),
          total: String(MAX_QUESTIONS),
        })}
      </p>
      <div
        class="progress-bar"
        role="progressbar"
        aria-valuenow={Math.round(progressPct)}
        aria-valuemin={0}
        aria-valuemax={100}
      >
        <div class="progress-bar__fill" style="width: {progressPct}%"></div>
      </div>
    </div>

    <div class="detect-prompt">
      <p class="detect-prompt__text">{t(appState.lang, "detect.charPrompt")}</p>
      <div class="detect-prompt__char-wrap">
        <div class="detect-prompt__char">{currentChar.char}</div>
      </div>
      <p class="detect-prompt__hint">{t(appState.lang, "detect.charHint")}</p>
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

<style>
  .detect {
    gap: 18px;
  }
  .detect__header {
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    width: 100%;
    max-width: 480px;
  }
  .detect__eyebrow {
    margin: 0;
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }
  .detect__progress-label {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 12px;
    letter-spacing: 0.04em;
    color: var(--color-text-secondary);
  }

  .detect-success {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 14px;
    margin-bottom: 20px;
  }
  .detect-kbd {
    display: flex;
    justify-content: center;
    --u: 32px;
    --gap: 4px;
    --radius-key: 6px;
  }
  .detect-kbd :global(.kbd-body) {
    --u: 32px;
    --gap: 4px;
    --radius-key: 6px;
  }

  /* Prompt centerpiece — dramatic char card */
  .detect-prompt {
    text-align: center;
    margin: 4px auto 0;
    padding: 26px 32px 28px;
    max-width: 540px;
    width: 100%;
    background:
      radial-gradient(ellipse at top, color-mix(in srgb, var(--color-accent) 6%, transparent), transparent 60%),
      var(--color-bg-card);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-md), inset 0 1px 0 rgba(255, 255, 255, 0.04);
    position: relative;
    overflow: hidden;
  }
  .detect-prompt::before {
    content: "";
    position: absolute;
    inset: 0 0 auto 0;
    height: 1px;
    background: linear-gradient(90deg, transparent, var(--color-accent-ring), transparent);
    opacity: 0.6;
  }
  .detect-prompt__text {
    font-size: 14px;
    color: var(--color-text-secondary);
    margin: 0 0 18px;
    letter-spacing: -0.005em;
  }
  .detect-prompt__char-wrap {
    display: flex;
    justify-content: center;
    margin: 0 auto 18px;
  }
  .detect-prompt__char {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 132px;
    height: 132px;
    font-family: var(--font-mono);
    font-size: 76px;
    font-weight: 500;
    color: var(--color-text);
    background: linear-gradient(180deg, #ffffff 0%, #ececf0 100%);
    border-radius: 22px;
    border: 1px solid rgba(0,0,0,0.10);
    box-shadow:
      0 0.5px 0 rgba(255,255,255,0.95) inset,
      0 -1px 0 rgba(0,0,0,0.04) inset,
      0 8px 22px rgba(0,0,0,0.12),
      0 2px 4px rgba(0,0,0,0.06);
    animation: charPop 380ms var(--ease-out) both;
  }
  @keyframes charPop {
    from { opacity: 0; transform: scale(0.85); }
    to   { opacity: 1; transform: scale(1);    }
  }
  :root[data-theme="dark"] .detect-prompt__char,
  :root:not([data-theme="light"]) .detect-prompt__char {
    background: linear-gradient(180deg, #5a5a5e 0%, #3f3f44 100%);
    border-color: rgba(0,0,0,0.50);
    color: #f5f5f7;
    box-shadow:
      0 0.5px 0 rgba(255,255,255,0.10) inset,
      0 -1px 0 rgba(0,0,0,0.30) inset,
      0 12px 28px rgba(0,0,0,0.55),
      0 2px 4px rgba(0,0,0,0.30);
  }
  .detect-prompt__hint {
    font-size: 13px;
    color: var(--color-text-muted);
    margin: 0;
    line-height: 1.5;
    max-width: 380px;
    margin-left: auto;
    margin-right: auto;
  }
</style>
