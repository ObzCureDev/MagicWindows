<script lang="ts">
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";
  import { open } from "@tauri-apps/plugin-shell";

  const version = "0.1.0";

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

<div class="page">
  <div class="page__header">
    <img src="/MagicWindows.png" alt="MagicWindows" class="welcome-logo" />
    <h1 class="page__title">{t(appState.lang, "about.title")}</h1>
    <p class="page__subtitle">
      {t(appState.lang, "about.version", { version })}
    </p>
  </div>

  <div class="page__content">
    <p class="text-secondary text-center" style="max-width: 420px;">
      {t(appState.lang, "about.description")}
    </p>

    <p class="text-muted" style="font-size: 13px;">
      {t(appState.lang, "about.license")}
    </p>

    <div class="page__actions">
      <button class="btn btn-secondary" onclick={goBack}>
        {t(appState.lang, "about.back")}
      </button>
      <button class="btn btn-primary" onclick={openGitHub}>
        {t(appState.lang, "about.github")}
      </button>
    </div>
  </div>
</div>
