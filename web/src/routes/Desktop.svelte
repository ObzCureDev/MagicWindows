<script lang="ts">
  // No reactive state — purely static pitch page.
</script>

<header class="head">
  <a class="back" href="#/">&larr; All layouts</a>
  <h1>The desktop app</h1>
  <p class="lede">
    The web ZIP installs <strong>only the layout</strong>. The desktop app does the same
    install, plus a handful of features that need a native client.
  </p>
</header>

<section class="compare">
  <table>
    <thead>
      <tr>
        <th></th>
        <th>Web ZIP</th>
        <th>Desktop app</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>Install a layout</td>
        <td>&#10003;</td>
        <td>&#10003;</td>
      </tr>
      <tr>
        <td>Auto-detect your keyboard</td>
        <td>&mdash;</td>
        <td>&#10003;</td>
      </tr>
      <tr>
        <td>Mac-style modifier toggles (Cmd&harr;Ctrl, Caps&rarr;Ctrl)</td>
        <td>&mdash;</td>
        <td>&#10003;</td>
      </tr>
      <tr>
        <td>F12 / Eject remap (Calculator, Search, Mute&hellip;)</td>
        <td>&mdash;</td>
        <td>&#10003;</td>
      </tr>
      <tr>
        <td>Health check after install</td>
        <td>&mdash;</td>
        <td>&#10003;</td>
      </tr>
      <tr>
        <td>One-click uninstall UI</td>
        <td>&mdash;</td>
        <td>&#10003;</td>
      </tr>
    </tbody>
  </table>
</section>

<section class="dl">
  <a class="download-btn" href="https://github.com/ObzCureDev/MagicWindows/releases/latest" target="_blank" rel="noopener">
    Download the desktop app
  </a>
  <p class="dl-note">Latest release on GitHub. Windows 10/11, x64.</p>
</section>

<style>
  .head {
    max-width: 880px;
    margin: 2rem auto 2rem;
    padding: 0 1.5rem;
  }
  .back {
    color: #0066cc;
    text-decoration: none;
    font-size: 0.9rem;
  }
  .back:hover { text-decoration: underline; }
  .head h1 {
    font-size: 2rem;
    margin: 0.5rem 0 0.5rem;
  }
  .lede {
    color: #3a3a3c;
    margin: 0;
    line-height: 1.5;
  }

  .compare {
    max-width: 880px;
    margin: 1.5rem auto;
    padding: 0 1.5rem;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    background: #ffffff;
    border: 1px solid #e5e5ea;
    border-radius: 12px;
    overflow: hidden;
  }
  th, td {
    padding: 0.85rem 1rem;
    text-align: left;
    border-bottom: 1px solid #e5e5ea;
  }
  th {
    background: #f5f5f7;
    font-weight: 600;
    font-size: 0.9rem;
  }
  td:nth-child(2), td:nth-child(3),
  th:nth-child(2), th:nth-child(3) {
    text-align: center;
    width: 14%;
    font-weight: 500;
  }
  tr:last-child td { border-bottom: none; }

  .dl {
    max-width: 880px;
    margin: 2rem auto 4rem;
    padding: 0 1.5rem;
    text-align: center;
  }
  .download-btn {
    display: inline-block;
    padding: 0.85rem 1.75rem;
    background: #0066cc;
    color: #ffffff;
    border-radius: 999px;
    text-decoration: none;
    font-weight: 600;
    box-shadow: 0 4px 12px rgba(0,102,204,0.25);
    transition: transform 100ms ease, box-shadow 100ms ease;
  }
  .download-btn:hover {
    transform: translateY(-1px);
    box-shadow: 0 6px 16px rgba(0,102,204,0.32);
  }
  .dl-note {
    color: #8e8e93;
    font-size: 0.85rem;
    margin: 0.75rem 0 0;
  }

  /* ── Tablet ───────────────────────────────────────────────────────── */
  @media (max-width: 768px) {
    .head { padding: 0 1.25rem; margin-top: 1.5rem; }
    .head h1 { font-size: 1.6rem; }
    .lede { font-size: 0.95rem; }
    .compare { padding: 0 1.25rem; }
    th, td { padding: 0.65rem 0.75rem; font-size: 0.85rem; }
    th:first-child, td:first-child { font-size: 0.8rem; }
    .download-btn {
      padding: 0.75rem 1.4rem;
      font-size: 0.95rem;
    }
  }

  /* ── Mobile ───────────────────────────────────────────────────────── */
  @media (max-width: 480px) {
    .head h1 { font-size: 1.4rem; }
    .lede { font-size: 0.9rem; }

    /* Collapse table to a stack: one row per feature, web/desktop side-by-side */
    table, thead, tbody, tr, th, td { display: block; }
    thead { display: none; }
    tr {
      background: #ffffff;
      border: 1px solid #e5e5ea;
      border-radius: 12px;
      margin-bottom: 0.75rem;
      padding: 0.75rem;
    }
    tr:last-child td { border-bottom: 1px solid #e5e5ea; }
    table {
      border: none;
      border-radius: 0;
      background: transparent;
    }
    td {
      border: none !important;
      padding: 0.25rem 0;
      font-size: 0.9rem;
    }
    td:first-child {
      font-weight: 600;
      font-size: 0.95rem;
      margin-bottom: 0.4rem;
      padding-bottom: 0.4rem;
      border-bottom: 1px solid #f0f0f3 !important;
    }
    td:nth-child(2),
    td:nth-child(3) {
      display: inline-block;
      width: auto;
      text-align: left;
      font-weight: 500;
    }
    td:nth-child(2)::before { content: "Web ZIP: "; color: #8e8e93; font-weight: 400; margin-right: 0.4rem; }
    td:nth-child(3)::before { content: "Desktop: "; color: #8e8e93; font-weight: 400; margin-right: 0.4rem; margin-left: 1rem; }

    .download-btn {
      display: block;
      padding: 0.85rem 1rem;
    }
  }
</style>
