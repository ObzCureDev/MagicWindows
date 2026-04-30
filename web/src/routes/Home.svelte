<script lang="ts">
  import { layouts, layoutIds } from "../lib/layouts";
</script>

<header class="hero">
  <h1>Apple keyboard layouts on Windows</h1>
  <p class="lede">
    Plug your Apple Magic Keyboard into a Windows PC and the printed keys don't match
    what you type. <strong>@</strong>, <strong>#</strong>, accented letters, even
    <strong>Shift+Enter</strong> — Windows ignores half the keycap labels.
  </p>
  <p class="lede">
    Download the matching layout for your keyboard, double-click the installer,
    restart. Done.
  </p>
  <a class="cta-secondary" href="#/desktop">Want auto-detect &amp; advanced options? &rarr;</a>
</header>

<section class="grid">
  {#each layoutIds as id}
    {@const card = layouts[id]}
    <a class="card" href={`#/preview/${id}`}>
      <h2>{card.displayName}</h2>
      <p>{card.blurb}</p>
      <span class="card-cta">Preview &amp; download &rarr;</span>
    </a>
  {/each}
</section>

<style>
  .hero {
    max-width: 720px;
    margin: 4rem auto 2rem;
    padding: 0 1.5rem;
    text-align: center;
  }
  .hero h1 {
    font-size: 2.5rem;
    margin: 0 0 1rem;
    line-height: 1.1;
  }
  .lede {
    font-size: 1.1rem;
    color: #3a3a3c;
    margin: 0 0 1rem;
  }
  .cta-secondary {
    display: inline-block;
    margin-top: 1rem;
    color: #0066cc;
    text-decoration: none;
    font-size: 0.95rem;
  }
  .cta-secondary:hover { text-decoration: underline; }

  .grid {
    max-width: 1080px;
    margin: 2rem auto 4rem;
    padding: 0 1.5rem;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
  }
  .card {
    background: #ffffff;
    border: 1px solid #e5e5ea;
    border-radius: 12px;
    padding: 1.25rem;
    text-decoration: none;
    color: inherit;
    transition: transform 120ms ease, box-shadow 120ms ease;
  }
  .card:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(0,0,0,0.06);
  }
  .card h2 {
    margin: 0 0 0.5rem;
    font-size: 1.15rem;
  }
  .card p {
    color: #6e6e73;
    font-size: 0.9rem;
    margin: 0 0 0.75rem;
    line-height: 1.45;
  }
  .card-cta {
    color: #0066cc;
    font-size: 0.9rem;
    font-weight: 500;
  }
</style>
