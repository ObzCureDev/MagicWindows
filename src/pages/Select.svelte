<script lang="ts">
  import { onMount } from "svelte";
  import { appState } from "../lib/stores.svelte";
  import { t } from "../lib/i18n";

  let search = $state("");

  let filteredLayouts = $derived(
    appState.layouts
      .filter((layout) => {
        if (!search.trim()) return true;
        const q = search.toLowerCase();
        const name = (layout.name[appState.lang] ?? layout.name["en"] ?? "").toLowerCase();
        const desc = (layout.description[appState.lang] ?? layout.description["en"] ?? "").toLowerCase();
        const locale = layout.locale.toLowerCase();
        return name.includes(q) || desc.includes(q) || locale.includes(q);
      })
      .sort((a, b) => {
        const aLangMatch = a.locale.startsWith(appState.lang);
        const bLangMatch = b.locale.startsWith(appState.lang);
        if (aLangMatch && !bLangMatch) return -1;
        if (!aLangMatch && bLangMatch) return 1;
        return 0;
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

  function localeFlag(locale: string): string {
    const parts = locale.split("-");
    const region = parts[1] ?? parts[0];
    if (!region || region.length !== 2) return "";
    const upper = region.toUpperCase();
    if (upper < "AA" || upper > "ZZ") return "";
    const codePoints = [...upper].map(
      (c) => 0x1f1e6 + c.charCodeAt(0) - 65,
    );
    return String.fromCodePoint(...codePoints);
  }

  onMount(() => {
    return () => {
      appState.detectionFailedMessage = null;
    };
  });
</script>

<div class="page select">
  <div class="page__header">
    <p class="select__eyebrow">{t(appState.lang, "ui.step", { n: "02" })}</p>
    <h1 class="page__title">{t(appState.lang, "select.title")}</h1>
    <p class="page__subtitle">{t(appState.lang, "select.instruction")}</p>
  </div>

  <div class="page__content">
    {#if appState.detectionFailedMessage}
      <div class="status status--info" role="status">
        {appState.detectionFailedMessage}
      </div>
    {/if}

    <div class="search-wrap">
      <svg class="search-wrap__icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
        <circle cx="11" cy="11" r="7" />
        <path d="m20 20-3.5-3.5" />
      </svg>
      <input
        class="search-input search-input--padded"
        type="text"
        placeholder={t(appState.lang, "select.searchPlaceholder")}
        aria-label={t(appState.lang, "select.searchPlaceholder")}
        bind:value={search}
      />
    </div>

    <div class="layout-grid">
      {#if filteredLayouts.length === 0}
        <p class="text-secondary text-center">
          {t(appState.lang, "select.noResults")}
        </p>
      {:else}
        {#each filteredLayouts as layout (layout.id)}
          <div
            class="card layout-card"
            class:card--selected={appState.selectedLayoutId === layout.id}
            onclick={() => selectLayout(layout.id)}
            role="button"
            tabindex="0"
            onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") selectLayout(layout.id); }}
          >
            <div class="layout-card__flag">{localeFlag(layout.locale)}</div>
            <div class="layout-card__body">
              <div class="card__title">
                {layout.name[appState.lang] ?? layout.name["en"] ?? layout.id}
              </div>
              <div class="card__locale">{layout.locale}</div>
              <div class="card__description">
                {layout.description[appState.lang] ?? layout.description["en"] ?? ""}
              </div>
            </div>
            {#if appState.selectedLayoutId === layout.id}
              <div class="layout-card__check" aria-hidden="true">
                <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round">
                  <path d="m5 12 5 5L20 7" />
                </svg>
              </div>
            {/if}
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
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M5 12h14M13 6l6 6-6 6" />
        </svg>
      </button>
    </div>
  </div>
</div>

<style>
  .select__eyebrow {
    margin: 0;
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: var(--color-text-muted);
  }

  .search-wrap {
    position: relative;
    width: 100%;
    max-width: 420px;
  }
  .search-wrap__icon {
    position: absolute;
    left: 14px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--color-text-muted);
    pointer-events: none;
  }
  :global(.search-input--padded) {
    padding-left: 38px;
  }

  .layout-card {
    display: grid;
    grid-template-columns: 56px 1fr auto;
    align-items: center;
    gap: 16px;
    padding: 14px 18px;
  }
  .layout-card__flag {
    width: 56px;
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 32px;
    background: var(--color-overlay);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    flex-shrink: 0;
  }
  .layout-card__body {
    min-width: 0;
  }
  .layout-card__check {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: var(--color-accent);
    color: #fff;
    box-shadow: 0 4px 12px var(--color-accent-glow);
    flex-shrink: 0;
  }
</style>
