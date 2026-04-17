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

  function themeIcon(theme: Theme): string {
    if (theme === "light") return "\u2600"; // sun
    if (theme === "dark") return "\u263E";  // moon
    return "\u25D1"; // half circle = system
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
    {t(appState.lang, "appTitle")}
  </button>
  <div class="top-bar__controls">
    <button
      class="theme-toggle"
      onclick={() => (appState.page = "modifiers")}
      title={t(appState.lang, "modifiers.topbarTitle")}
      aria-label={t(appState.lang, "modifiers.topbarTitle")}
    >
      ⌘
    </button>
    <button
      class="theme-toggle"
      onclick={cycleTheme}
      title={appState.theme === "system" ? "System theme" : appState.theme === "dark" ? "Dark" : "Light"}
    >
      {themeIcon(appState.theme)}
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
{/if}
