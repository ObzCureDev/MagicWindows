<script lang="ts">
  import { KeyboardVisual } from "@magicwindows/keyboard-visual";
  import { layouts, layoutIds } from "../lib/layouts";
  import { downloadsFor, ARCHES, ARCH_LABEL } from "../lib/manifest";
  import { navigate } from "../lib/router";

  interface Props {
    layoutId: string;
  }
  let { layoutId }: Props = $props();

  let activeLayer = $state<"base" | "shift" | "altgr" | "altgrShift">("base");

  const card = $derived(layouts[layoutId]);
  const downloads = $derived(card ? downloadsFor(layoutId) : {});
  const hasAny = $derived(ARCHES.some((a) => downloads[a]));

  $effect(() => {
    if (!card) navigate({ kind: "home" });
  });

  function pickLayout(id: string) {
    navigate({ kind: "preview", layoutId: id });
  }

  const layers = [
    { k: "base" as const, label: "Base" },
    { k: "shift" as const, label: "Shift" },
    { k: "altgr" as const, label: "AltGr" },
    { k: "altgrShift" as const, label: "AltGr+Shift" },
  ];
</script>

{#if card}
  <header class="head">
    <a class="back" href="#/">&larr; All layouts</a>
    <h1>{card.displayName}</h1>
    <p class="blurb">{card.blurb}</p>
  </header>

  <nav class="switcher" aria-label="Choose a layout">
    {#each layoutIds as id}
      <button
        type="button"
        class:active={id === layoutId}
        onclick={() => pickLayout(id)}
      >
        {layouts[id].displayName}
      </button>
    {/each}
  </nav>

  <div class="layer-toggle" role="group" aria-label="Modifier layer">
    {#each layers as opt}
      <button
        type="button"
        class:active={activeLayer === opt.k}
        onclick={() => (activeLayer = opt.k)}
      >
        {opt.label}
      </button>
    {/each}
  </div>

  <div class="preview-wrap">
    <KeyboardVisual layout={card.layout} {activeLayer} />
  </div>

  <div class="download-bar">
    {#if hasAny}
      <p class="dl-prompt">Pick the build that matches your PC:</p>
      <div class="dl-row">
        {#each ARCHES as arch}
          {@const dl = downloads[arch]}
          {#if dl}
            <a class="download-btn" href={dl.url} download>
              <span class="dl-arch">{ARCH_LABEL[arch]}</span>
              <span class="dl-size">{(dl.size / 1024).toFixed(0)} KB</span>
            </a>
          {:else}
            <span class="download-btn download-btn--missing" aria-disabled="true">
              <span class="dl-arch">{ARCH_LABEL[arch]}</span>
              <span class="dl-size">unavailable</span>
            </span>
          {/if}
        {/each}
      </div>
      <p class="dl-help">
        <strong>x64</strong> for most PCs (Intel / AMD).
        <strong>ARM64</strong> for Snapdragon / Surface Pro X / Copilot+ PCs.
      </p>
    {:else}
      <p class="dl-unavailable">Downloads temporarily unavailable. Please try again later.</p>
    {/if}
  </div>
{/if}

<style>
  .head {
    max-width: 1080px;
    margin: 2rem auto 1rem;
    padding: 0 1.5rem;
  }
  .back {
    color: #0066cc;
    text-decoration: none;
    font-size: 0.9rem;
  }
  .back:hover { text-decoration: underline; }
  .head h1 {
    font-size: 1.8rem;
    margin: 0.5rem 0 0.25rem;
  }
  .blurb {
    color: #6e6e73;
    margin: 0;
    line-height: 1.45;
  }

  .switcher {
    max-width: 1080px;
    margin: 1rem auto;
    padding: 0 1.5rem;
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .switcher button,
  .layer-toggle button {
    padding: 0.4rem 0.85rem;
    border-radius: 8px;
    border: 1px solid #d2d2d7;
    background: #ffffff;
    color: #1a1a1c;
    font-size: 0.85rem;
    cursor: pointer;
    transition: background 100ms ease, border-color 100ms ease;
  }
  .switcher button:hover,
  .layer-toggle button:hover { background: #f3f3f7; }
  .switcher button.active,
  .layer-toggle button.active {
    background: #0066cc;
    border-color: #0066cc;
    color: #ffffff;
  }

  .layer-toggle {
    max-width: 1080px;
    margin: 0.5rem auto 1rem;
    padding: 0 1.5rem;
    display: flex;
    gap: 0.5rem;
  }

  .preview-wrap {
    max-width: 1080px;
    margin: 1rem auto;
    padding: 0 1rem;
    display: flex;
    justify-content: center;
  }

  .download-bar {
    max-width: 1080px;
    margin: 1.5rem auto 4rem;
    padding: 0 1.5rem;
    text-align: center;
  }
  .dl-prompt {
    color: #1a1a1c;
    font-size: 0.95rem;
    margin: 0 0 0.75rem;
    font-weight: 500;
  }
  .dl-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    justify-content: center;
  }
  .download-btn {
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    gap: 0.15rem;
    padding: 0.7rem 1.4rem;
    background: #0066cc;
    color: #ffffff;
    border-radius: 14px;
    text-decoration: none;
    font-weight: 600;
    box-shadow: 0 4px 12px rgba(0,102,204,0.25);
    transition: transform 100ms ease, box-shadow 100ms ease;
    min-width: 140px;
  }
  .download-btn:hover {
    transform: translateY(-1px);
    box-shadow: 0 6px 16px rgba(0,102,204,0.32);
  }
  .download-btn--missing {
    background: #e5e5ea;
    color: #8e8e93;
    box-shadow: none;
    cursor: not-allowed;
  }
  .download-btn--missing:hover { transform: none; box-shadow: none; }
  .dl-arch {
    font-size: 1rem;
    line-height: 1.2;
  }
  .dl-size {
    font-size: 0.75rem;
    font-weight: 500;
    opacity: 0.85;
  }
  .dl-help {
    color: #6e6e73;
    font-size: 0.8rem;
    margin: 0.85rem 0 0;
    line-height: 1.5;
  }
  .dl-help strong {
    color: #1a1a1c;
    font-weight: 600;
  }
  .dl-unavailable {
    color: #b91c1c;
    font-size: 0.95rem;
  }

  /* ── Tablet ───────────────────────────────────────────────────────── */
  @media (max-width: 768px) {
    .head { padding: 0 1.25rem; margin-top: 1.5rem; }
    .head h1 { font-size: 1.5rem; }
    .switcher,
    .layer-toggle { padding: 0 1.25rem; }
    .switcher button,
    .layer-toggle button {
      padding: 0.35rem 0.7rem;
      font-size: 0.8rem;
    }
    .preview-wrap {
      padding: 0;
      overflow-x: auto;
      justify-content: flex-start;
      -webkit-overflow-scrolling: touch;
    }
    .preview-wrap > :global(*) {
      margin: 0 1rem;
    }
    .download-bar { padding: 0 1.25rem; margin-bottom: 3rem; }
    .download-btn {
      padding: 0.75rem 1.4rem;
      font-size: 0.95rem;
    }
  }

  /* ── Mobile ───────────────────────────────────────────────────────── */
  @media (max-width: 480px) {
    .head h1 { font-size: 1.3rem; }
    .blurb { font-size: 0.9rem; }
    .switcher button,
    .layer-toggle button {
      padding: 0.3rem 0.6rem;
      font-size: 0.75rem;
    }
    .dl-row {
      flex-direction: column;
      align-items: stretch;
    }
    .download-btn {
      min-width: 0;
      padding: 0.85rem 1rem;
    }
    .dl-help {
      font-size: 0.75rem;
    }
  }
</style>
