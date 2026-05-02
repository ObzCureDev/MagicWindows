<script lang="ts">
  import { parseHash } from "./lib/router";
  import Home from "./routes/Home.svelte";
  import Preview from "./routes/Preview.svelte";
  import Desktop from "./routes/Desktop.svelte";
  import { manifest } from "./lib/manifest";

  let route = $state(parseHash(window.location.hash));

  function onHashChange() {
    route = parseHash(window.location.hash);
  }

  $effect(() => {
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
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
      Made by
      <a href="https://mindvisionstudio.com" target="_blank" rel="noopener">MindVision Studio</a>
      <span class="dot" aria-hidden="true">·</span>
      <a href="https://github.com/ObzCureDev/MagicWindows" target="_blank" rel="noopener">GitHub</a>
      <span class="dot" aria-hidden="true">·</span>
      <a href="https://github.com/ObzCureDev/MagicWindows/issues" target="_blank" rel="noopener">Report a bug</a>
      <span class="dot" aria-hidden="true">·</span>
      <span class="version">v{manifest.version}</span>
    </p>
    <p class="site-footer__sub">
      MagicWindows is open source (MIT). Apple, Magic Keyboard, and macOS are
      trademarks of Apple Inc. Windows is a trademark of Microsoft Corporation.
      This project is not affiliated with either.
    </p>
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
