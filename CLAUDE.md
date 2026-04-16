# CLAUDE.md — MagicWindows

Tauri v2 + Svelte 5 + Rust desktop app that installs Apple Magic Keyboard layouts on Windows, fixing keycap mismatches when using Apple keyboards on Windows.

## Build & Development Commands

```bash
npm install                # Install dependencies
npm run tauri dev          # Dev mode (Vite + Tauri together)
npm run tauri build        # Production build → src-tauri/target/release/bundle/ (MSI + NSIS)
npm run check              # TypeScript + Svelte type checking
npm run build              # Vite frontend build only

# Rust (run from src-tauri/ directory)
cargo test                 # Run Rust tests
cargo clippy               # Rust linting
cargo fmt                  # Rust formatting
```

## Architecture

- **Frontend**: Svelte 5 + TypeScript. Page-based routing via reactive state (`$state` rune in `src/lib/stores.ts`).
- **Backend**: Rust + Tauri v2. Handles keyboard detection, KLC file generation, and layout installation.
- **Layouts**: JSON files in `layouts/` validated against `layouts/schema.json`.
- **i18n**: Custom bilingual (EN/FR) system in `src/lib/i18n.ts`. All UI text must have both translations.
- **Theming**: System/Dark/Light mode via CSS custom properties on `:root[data-theme]`. Toggle in top bar cycles system → dark → light.
- **Page flow**: Welcome → Detect → Select → Preview → Install → Done (+ About via title bar)

## Project Structure

```
src/                       # Svelte 5 frontend
  pages/                   # Welcome, Detect, Select, Preview, Install, Done, About
  components/
    KeyboardVisual.svelte  # Reusable keyboard rendering component
  lib/
    stores.ts              # App state ($state rune) — includes theme, lang
    i18n.ts                # EN/FR translations
    types.ts               # TypeScript types (Page, Lang, Theme, Layout, etc.)
src-tauri/                 # Rust backend (Tauri v2)
  src/keyboard/            # Detection, KLC generation, installation logic
layouts/                   # Keyboard layout JSON definitions (6 layouts)
  schema.json              # Layout validation schema
  apple-fr-azerty.json     # French AZERTY
  apple-us-qwerty.json     # US QWERTY
  apple-uk-qwerty.json     # UK (British) QWERTY
  apple-de-qwertz.json     # German QWERTZ
  apple-es-qwerty.json     # Spanish QWERTY
  apple-it-qwerty.json     # Italian QWERTY
scripts/                   # PowerShell scripts (standalone, not used by app)
  Install-Layout.ps1       # Layout installation
  Uninstall-Layout.ps1     # Layout removal
```

## Key Conventions

- Layout JSON files must validate against `layouts/schema.json`
- DLL names: max 8 chars, lowercase alphanumeric
- Layout IDs follow pattern: `apple-{lang}-{type}` (e.g., `apple-fr-azerty`)
- All UI text must have both EN and FR translations in `src/lib/i18n.ts`
- Scancodes and Unicode codepoints in hex (e.g., `0040` for `@`)
- Dead keys marked with `@` suffix in key values (e.g., `005e@` for `^` as dead key)
- Use `-1` for "no character produced" in layout mappings
- Detection keys: 3-5 keys per layout that distinguish it from the Windows default

## Prerequisites

- Node.js 20+
- Rust 1.77+
- Tauri CLI (`@tauri-apps/cli` included in devDependencies)
