<script lang="ts">
  import { layouts, layoutIds } from "../lib/layouts";
</script>

<header class="hero">
  <div class="hero-logo-wrap">
    <img class="hero-logo" src="/logo-256.png" alt="MagicWindows logo" width="120" height="120" />
  </div>
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
    margin: 3rem auto 2rem;
    padding: 0 1.25rem;
    text-align: center;
  }
  .hero-logo-wrap {
    display: inline-block;
    padding: 14px 18px 18px;
    margin: 0 auto 1.25rem;
    background: linear-gradient(135deg, #0066cc 0%, #003c7a 100%);
    border-radius: 26px;
    box-shadow:
      0 1px 0 rgba(255,255,255,0.18) inset,
      0 12px 30px rgba(0,102,204,0.28),
      0 4px 10px rgba(0,0,0,0.12);
  }
  .hero-logo {
    width: 120px;
    height: 120px;
    display: block;
    image-rendering: -webkit-optimize-contrast;
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
    padding: 0 1.25rem;
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

  /* ── Tablet ───────────────────────────────────────────────────────── */
  @media (max-width: 768px) {
    .hero {
      margin: 2rem auto 1.5rem;
    }
    .hero-logo-wrap {
      padding: 12px 14px 14px;
      margin-bottom: 1rem;
      border-radius: 22px;
    }
    .hero-logo {
      width: 96px;
      height: 96px;
    }
    .hero h1 {
      font-size: 2rem;
    }
    .lede {
      font-size: 1rem;
    }
    .grid {
      grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
      margin: 1.5rem auto 3rem;
    }
  }

  /* ── Mobile ───────────────────────────────────────────────────────── */
  @media (max-width: 480px) {
    .hero h1 {
      font-size: 1.65rem;
    }
    .hero-logo-wrap {
      padding: 10px 12px 12px;
      border-radius: 18px;
    }
    .hero-logo {
      width: 88px;
      height: 88px;
    }
    .lede {
      font-size: 0.95rem;
    }
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
