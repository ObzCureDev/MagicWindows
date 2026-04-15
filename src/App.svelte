<script lang="ts">
  import { appState } from "./lib/stores";
  import { t } from "./lib/i18n";
  import Welcome from "./pages/Welcome.svelte";
  import Detect from "./pages/Detect.svelte";
  import Select from "./pages/Select.svelte";
  import Preview from "./pages/Preview.svelte";
  import Install from "./pages/Install.svelte";
  import Done from "./pages/Done.svelte";

  function setLang(lang: "en" | "fr") {
    appState.lang = lang;
  }
</script>

<div class="top-bar">
  <div class="top-bar__title">
    <span class="top-bar__title-icon">M</span>
    {t(appState.lang, "appTitle")}
  </div>
  <div class="lang-toggle">
    <button
      class="lang-toggle__btn"
      class:lang-toggle__btn--active={appState.lang === "fr"}
      onclick={() => setLang("fr")}
    >
      FR
    </button>
    <button
      class="lang-toggle__btn"
      class:lang-toggle__btn--active={appState.lang === "en"}
      onclick={() => setLang("en")}
    >
      EN
    </button>
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
{:else if appState.page === "done"}
  <Done />
{/if}
