<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";

  interface Props {
    error: string | null;
    onRetry: () => Promise<void>;
    operationName: string;
    context?: Record<string, unknown>;
    attemptCount: number;
  }

  let { error, onRetry, operationName, context = {}, attemptCount }: Props = $props();

  let submitting = $state(false);
  let sending = $state(false);

  function classifyError(msg: string): "uac" | "generic" {
    const lower = msg.toLowerCase();
    if (/admin|privilege|uac|access denied|cancell|cancelled|a annul|runas|start-process/.test(lower)) {
      return "uac";
    }
    return "generic";
  }

  async function handleRetry() {
    if (submitting) return;
    submitting = true;
    try {
      await onRetry();
    } finally {
      submitting = false;
    }
  }

  async function handleSendReport() {
    if (sending) return;
    sending = true;
    try {
      const diagnostics = await invoke<string>("collect_diagnostics");
      const ctxJson = Object.keys(context).length > 0
        ? `\n\n**Context:**\n\`\`\`json\n${JSON.stringify(context, null, 2)}\n\`\`\``
        : "";
      const intro = t(appState.lang, "bugReport.bodyIntro");
      const subject = t(appState.lang, "bugReport.subject", { op: operationName });
      const body = `${intro}\n\n${diagnostics}${ctxJson}\n\n--- Votre commentaire ---\n\n`;
      const url = `mailto:bug@mindvisionstudio.com?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body)}`;
      const { open } = await import("@tauri-apps/plugin-shell");
      await open(url);
    } catch (e) {
      console.error("Failed to open bug report:", e);
    } finally {
      sending = false;
    }
  }
</script>

{#if error}
  <div class="elevated-error" role="alert">
    <div class="elevated-error__header">
      <span class="elevated-error__icon" aria-hidden="true">&#9888;</span>
      <h3 class="elevated-error__title">{t(appState.lang, "elevatedError.title")}</h3>
    </div>
    <p class="elevated-error__hint">
      {classifyError(error) === "uac"
        ? t(appState.lang, "elevatedError.uacHint")
        : t(appState.lang, "elevatedError.generic")}
    </p>

    <div class="elevated-error__actions">
      <button class="btn btn-primary" onclick={handleRetry} disabled={submitting || sending}>
        {submitting ? t(appState.lang, "elevatedError.retrying") : t(appState.lang, "elevatedError.retry")}
      </button>
      {#if attemptCount >= 2}
        <button class="btn btn-secondary" onclick={handleSendReport} disabled={submitting || sending}>
          &#128231;&nbsp;{sending ? t(appState.lang, "elevatedError.sending") : t(appState.lang, "elevatedError.sendReport")}
        </button>
      {/if}
    </div>

    <details class="elevated-error__details">
      <summary>{t(appState.lang, "elevatedError.techDetails")}</summary>
      <pre class="elevated-error__raw">{error}</pre>
    </details>
  </div>
{/if}

<style>
  .elevated-error {
    width: 100%;
    max-width: 560px;
    margin: 12px auto;
    padding: 16px 18px;
    border: 1px solid var(--color-danger-border, rgba(220, 53, 69, 0.45));
    background: var(--color-danger-bg, rgba(220, 53, 69, 0.06));
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    box-sizing: border-box;
  }
  .elevated-error__header {
    display: flex;
    gap: 10px;
    align-items: center;
  }
  .elevated-error__icon {
    font-size: 1.4rem;
    color: var(--color-danger, #dc3545);
  }
  .elevated-error__title {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }
  .elevated-error__hint {
    margin: 0;
    font-size: 0.9rem;
    line-height: 1.4;
    color: var(--color-text-secondary);
  }
  .elevated-error__actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }
  .elevated-error__details {
    margin-top: 4px;
    font-size: 0.8rem;
  }
  .elevated-error__details > summary {
    cursor: pointer;
    color: var(--color-text-secondary);
  }
  .elevated-error__raw {
    margin: 8px 0 0;
    padding: 8px;
    background: var(--color-bg-elevated, rgba(0, 0, 0, 0.08));
    border-radius: 6px;
    font-size: 0.75rem;
    white-space: pre-wrap;
    word-break: break-all;
    max-height: 180px;
    overflow: auto;
  }
</style>
