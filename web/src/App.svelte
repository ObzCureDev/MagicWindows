<script lang="ts">
  import { parseHash } from "./lib/router";
  import Home from "./routes/Home.svelte";
  import Preview from "./routes/Preview.svelte";
  import Desktop from "./routes/Desktop.svelte";
  import { manifest } from "./lib/manifest";
  import { i18n, t } from "./lib/i18n.svelte";

  let route = $state(parseHash(window.location.hash));

  function onHashChange() {
    route = parseHash(window.location.hash);
  }

  $effect(() => {
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  });

  // Sync the <html lang> attribute on first render so screen readers + search
  // engines see the right initial language. The setter in i18n updates it on
  // every subsequent toggle.
  $effect(() => {
    document.documentElement.lang = i18n.lang;
  });
</script>

<div class="page">
  <main class="page-main">
    {#if route.kind === "home"}
      <Home />
    {:else if route.kind === "preview"}
      <Preview layoutId={route.layoutId} />
    {:else if route.kind === "desktop"}
      <Desktop />
    {:else}
      <Home />
    {/if}
  </main>

  <footer class="site-footer">
    <p class="site-footer__line">
      {t("footer.madeBy")}
      <a href="https://mindvisionstudio.com" target="_blank" rel="noopener">MindVision Studio</a>
      <span class="dot" aria-hidden="true">·</span>
      <a href="https://github.com/ObzCureDev/MagicWindows" target="_blank" rel="noopener">{t("footer.github")}</a>
      <span class="dot" aria-hidden="true">·</span>
      <a href="https://github.com/ObzCureDev/MagicWindows/issues" target="_blank" rel="noopener">{t("footer.bug")}</a>
      <span class="dot" aria-hidden="true">·</span>
      <span class="version">v{manifest.version}</span>
      <span class="dot" aria-hidden="true">·</span>
      <span class="lang-toggle" role="group" aria-label={t("footer.langToggleAriaLabel")}>
        <button
          type="button"
          class="lang-btn"
          class:active={i18n.lang === "en"}
          aria-pressed={i18n.lang === "en"}
          onclick={() => (i18n.lang = "en")}
        >EN</button>
        <span class="lang-sep" aria-hidden="true">/</span>
        <button
          type="button"
          class="lang-btn"
          class:active={i18n.lang === "fr"}
          aria-pressed={i18n.lang === "fr"}
          onclick={() => (i18n.lang = "fr")}
        >FR</button>
      </span>
    </p>
    <p class="site-footer__sub">{t("footer.disclaimer")}</p>
  </footer>
</div>

<style>
  .page {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }
  .page-main {
    flex: 1 0 auto;
  }

  .site-footer {
    flex-shrink: 0;
    padding: 2rem 1.5rem 2.5rem;
    text-align: center;
    border-top: 1px solid #e5e5ea;
    background: #f5f5f7;
    color: #6e6e73;
    font-size: 0.85rem;
    line-height: 1.6;
  }
  .site-footer__line {
    margin: 0 0 0.5rem;
  }
  .site-footer__line a {
    color: #1a1a1c;
    text-decoration: none;
    font-weight: 500;
  }
  .site-footer__line a:hover { color: #0066cc; text-decoration: underline; }
  .dot {
    margin: 0 0.4rem;
    color: #c7c7cc;
  }
  .version {
    font-family: "SF Mono", Menlo, Consolas, monospace;
    font-size: 0.78rem;
  }

  .lang-toggle {
    display: inline-flex;
    align-items: center;
    gap: 0.15rem;
    padding: 0.1rem 0.3rem;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.04);
  }
  .lang-btn {
    background: transparent;
    border: none;
    padding: 0.1rem 0.35rem;
    font: inherit;
    font-size: 0.75rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    color: #8e8e93;
    cursor: pointer;
    border-radius: 4px;
    transition: color 100ms ease;
  }
  .lang-btn:hover { color: #1a1a1c; }
  .lang-btn.active {
    color: #0066cc;
    background: #ffffff;
    box-shadow: 0 1px 2px rgba(0,0,0,0.06);
  }
  .lang-sep {
    color: #c7c7cc;
    font-size: 0.7rem;
  }
  .site-footer__sub {
    max-width: 640px;
    margin: 0 auto;
    font-size: 0.72rem;
    color: #8e8e93;
    line-height: 1.5;
  }

  @media (max-width: 480px) {
    .site-footer { padding: 1.5rem 1.25rem 2rem; font-size: 0.8rem; }
    .site-footer__line a { display: inline-block; }
    .site-footer__sub { font-size: 0.7rem; }
  }
</style>
