<script lang="ts">
  import { appState } from "../lib/stores";
  import { t } from "../lib/i18n";

  let search = $state("");

  let filteredLayouts = $derived(
    appState.layouts.filter((layout) => {
      if (!search.trim()) return true;
      const q = search.toLowerCase();
      const name = (layout.name[appState.lang] ?? layout.name["en"] ?? "").toLowerCase();
      const desc = (layout.description[appState.lang] ?? layout.description["en"] ?? "").toLowerCase();
      const locale = layout.locale.toLowerCase();
      return name.includes(q) || desc.includes(q) || locale.includes(q);
    }),
  );

  function selectLayout(id: string) {
    appState.selectedLayoutId = id;
  }

  function goPreview() {
    if (appState.selectedLayoutId) {
      appState.page = "preview";
    }
  }

  function goBack() {
    appState.page = "welcome";
  }

  /**
   * Map a locale string like "fr-FR" to a short region flag label.
   */
  function localeFlag(locale: string): string {
    const parts = locale.split("-");
    const region = parts[1] ?? parts[0];
    // Use regional indicator symbols to get flag emoji
    const codePoints = [...region.toUpperCase()].map(
      (c) => 0x1f1e6 + c.charCodeAt(0) - 65,
    );
    return String.fromCodePoint(...codePoints);
  }
</script>

<div class="page">
  <div class="page__header">
    <h1 class="page__title">{t(appState.lang, "select.title")}</h1>
    <p class="page__subtitle">{t(appState.lang, "select.instruction")}</p>
  </div>

  <div class="page__content">
    <input
      class="search-input"
      type="text"
      placeholder={t(appState.lang, "select.searchPlaceholder")}
      bind:value={search}
    />

    <div class="layout-grid">
      {#if filteredLayouts.length === 0}
        <p class="text-secondary text-center">
          {t(appState.lang, "select.noResults")}
        </p>
      {:else}
        {#each filteredLayouts as layout (layout.id)}
          <div
            class="card"
            class:card--selected={appState.selectedLayoutId === layout.id}
            onclick={() => selectLayout(layout.id)}
            role="button"
            tabindex="0"
            onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") selectLayout(layout.id); }}
          >
            <div class="card__title">
              {localeFlag(layout.locale)}
              {layout.name[appState.lang] ?? layout.name["en"] ?? layout.id}
            </div>
            <div class="card__locale">{layout.locale}</div>
            <div class="card__description">
              {layout.description[appState.lang] ?? layout.description["en"] ?? ""}
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <div class="page__actions">
      <button class="btn btn-secondary" onclick={goBack}>
        {t(appState.lang, "select.back")}
      </button>
      <button
        class="btn btn-primary"
        disabled={!appState.selectedLayoutId}
        onclick={goPreview}
      >
        {t(appState.lang, "select.continue")}
      </button>
    </div>
  </div>
</div>
