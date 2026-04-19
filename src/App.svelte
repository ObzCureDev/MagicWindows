<script lang="ts">
  import { onMount } from "svelte";
  import { appState } from "./lib/stores.svelte";
  import { t } from "./lib/i18n";
  import type { Theme } from "./lib/types";
  import Welcome from "./pages/Welcome.svelte";
  import Detect from "./pages/Detect.svelte";
  import Select from "./pages/Select.svelte";
  import Preview from "./pages/Preview.svelte";
  import Install from "./pages/Install.svelte";
  import Test from "./pages/Test.svelte";
  import Done from "./pages/Done.svelte";
  import About from "./pages/About.svelte";
  import Modifiers from "./pages/Modifiers.svelte";
  import Settings from "./pages/Settings.svelte";

  function setLang(lang: "en" | "fr") {
    appState.lang = lang;
  }

  function cycleTheme() {
    const order: Theme[] = ["system", "dark", "light"];
    const idx = order.indexOf(appState.theme);
    appState.theme = order[(idx + 1) % order.length];
    applyTheme(appState.theme);
  }

  function applyTheme(theme: Theme) {
    if (theme === "system") {
      document.documentElement.removeAttribute("data-theme");
    } else {
      document.documentElement.setAttribute("data-theme", theme);
    }
  }

  function themeLabel(theme: Theme): string {
    if (theme === "light") return "Light theme";
    if (theme === "dark")  return "Dark theme";
    return "System theme";
  }

  // Apply on load
  applyTheme(appState.theme);

  onMount(async () => {
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check();
      if (update) {
        console.log("Update available:", update.version);
      }
    } catch {
      // Silently ignore: plugin unavailable in dev or update check failed
    }
  });
</script>

<div class="top-bar">
  <button class="top-bar__title" onclick={() => appState.page = "about"}>
    <img src="/MagicWindows.png" alt="" class="top-bar__title-icon" />
    <span class="top-bar__brand">{t(appState.lang, "appTitle")}</span>
  </button>

  <div class="top-bar__controls">
    <button
      class="theme-toggle"
      onclick={() => (appState.page = "settings")}
      title={t(appState.lang, "settings.topbarTitle")}
      aria-label={t(appState.lang, "settings.topbarTitle")}
    >
      <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <circle cx="12" cy="12" r="3" />
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
      </svg>
    </button>

    <button
      class="theme-toggle theme-toggle--cmd"
      onclick={() => (appState.page = "modifiers")}
      title={t(appState.lang, "modifiers.topbarTitle")}
      aria-label={t(appState.lang, "modifiers.topbarTitle")}
    >
      ⌘
    </button>

    <button
      class="theme-toggle"
      onclick={cycleTheme}
      title={themeLabel(appState.theme)}
      aria-label={themeLabel(appState.theme)}
    >
      {#if appState.theme === "light"}
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="12" cy="12" r="4" />
          <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
        </svg>
      {:else if appState.theme === "dark"}
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
        </svg>
      {:else}
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <rect x="3" y="4" width="18" height="14" rx="2" />
          <path d="M8 21h8M12 18v3" />
        </svg>
      {/if}
    </button>

    <div class="lang-toggle">
      <button
        class="lang-toggle__btn"
        class:lang-toggle__btn--active={appState.lang === "fr"}
        aria-pressed={appState.lang === "fr"}
        onclick={() => setLang("fr")}
      >
        FR
      </button>
      <button
        class="lang-toggle__btn"
        class:lang-toggle__btn--active={appState.lang === "en"}
        aria-pressed={appState.lang === "en"}
        onclick={() => setLang("en")}
      >
        EN
      </button>
    </div>
  </div>
</div>

{#if appState.page === "welcome"}
  <Welcome />
{:else if appState.page === "detect"}
  <Detect />
{:else if appState.page === "select"}
  <Select />
{:else if appState.page === "preview"}
  <Preview />
{:else if appState.page === "install"}
  <Install />
{:else if appState.page === "test"}
  <Test />
{:else if appState.page === "done"}
  <Done />
{:else if appState.page === "about"}
  <About />
{:else if appState.page === "modifiers"}
  <Modifiers />
{:else if appState.page === "settings"}
  <Settings />
{/if}

<style>
  .top-bar__brand {
    font-feature-settings: "ss01" on, "cv01" on;
  }
  .theme-toggle--cmd {
    font-size: 17px;
    font-family: -apple-system, "SF Pro Text", "Segoe UI Symbol", sans-serif;
    line-height: 1;
  }
</style>
