<script lang="ts">
  import { parseHash } from "./lib/router";
  import Home from "./routes/Home.svelte";
  import Preview from "./routes/Preview.svelte";
  import Desktop from "./routes/Desktop.svelte";

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
  {#if route.kind === "home"}
    <Home />
  {:else if route.kind === "preview"}
    <Preview layoutId={route.layoutId} />
  {:else if route.kind === "desktop"}
    <Desktop />
  {:else}
    <Home />
  {/if}
</div>

<style>
  .page {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }
</style>
