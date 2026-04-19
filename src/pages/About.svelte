<script lang="ts">
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import { open } from "@tauri-apps/plugin-shell";

  const version = "0.1.3";

  async function openGitHub() {
    try {
      await open("https://github.com/ObzCureDev/MagicWindows");
    } catch {
      // Fallback: won't work in dev but ok
    }
  }

  function goBack() {
    appState.page = "welcome";
  }
</script>

<div class="page about">
  <div class="about__content">
    <img src="/MagicWindows.png" alt="MagicWindows" class="welcome-logo" />

    <p class="about__eyebrow">{t(appState.lang, "ui.about")}</p>
    <h1 class="page__title">{t(appState.lang, "about.title")}</h1>
    <p class="about__version">
      {t(appState.lang, "about.version", { version })}
    </p>

    <p class="about__description">
      {t(appState.lang, "about.description")}
    </p>

    <div class="about__meta">
      <span class="about__chip">{t(appState.lang, "about.license")}</span>
      <span class="about__chip about__chip--muted">Tauri v2 · Svelte 5 · Rust</span>
    </div>

    <div class="page__actions">
      <button class="btn btn-secondary" onclick={goBack}>
        {t(appState.lang, "about.back")}
      </button>
      <button class="btn btn-primary" onclick={openGitHub}>
        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor" aria-hidden="true">
          <path d="M12 .5C5.65.5.5 5.65.5 12c0 5.08 3.29 9.39 7.86 10.91.58.1.79-.25.79-.56v-2.06c-3.2.7-3.87-1.36-3.87-1.36-.52-1.32-1.27-1.67-1.27-1.67-1.04-.71.08-.7.08-.7 1.15.08 1.76 1.18 1.76 1.18 1.02 1.76 2.69 1.25 3.34.95.1-.74.4-1.25.73-1.54-2.55-.29-5.24-1.28-5.24-5.69 0-1.26.45-2.29 1.18-3.1-.12-.29-.51-1.46.11-3.05 0 0 .96-.31 3.15 1.18.91-.25 1.89-.38 2.86-.39.97.01 1.95.14 2.86.39 2.18-1.49 3.14-1.18 3.14-1.18.62 1.59.23 2.76.11 3.05.74.81 1.18 1.84 1.18 3.1 0 4.42-2.7 5.39-5.27 5.68.41.36.78 1.06.78 2.14v3.17c0 .31.21.67.8.55C20.21 21.39 23.5 17.08 23.5 12 23.5 5.65 18.35.5 12 .5z"/>
        </svg>
        {t(appState.lang, "about.github")}
      </button>
    </div>
  </div>
</div>

<style>
  .about__content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    text-align: center;
    max-width: 480px;
  }
  .about__eyebrow {
    margin: 8px 0 0;
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }
  .about__version {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--color-text-muted);
    letter-spacing: 0.04em;
  }
  .about__description {
    margin: 8px 0 0;
    font-size: 14px;
    line-height: 1.6;
    color: var(--color-text-secondary);
    text-wrap: balance;
  }
  .about__meta {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    justify-content: center;
    margin-top: 8px;
  }
  .about__chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 12px;
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--color-text-secondary);
    background: var(--color-overlay);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-pill);
    letter-spacing: 0.04em;
  }
  .about__chip--muted { color: var(--color-text-muted); }
</style>
