# MagicWindows

**Install Apple Magic Keyboard layouts on Windows** | **Installer les dispositions Apple Magic Keyboard sur Windows**

[![Build](https://github.com/ObzCureDev/Magicwindows/actions/workflows/build.yml/badge.svg)](https://github.com/ObzCureDev/Magicwindows/actions/workflows/build.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

---

## The Problem

When you use an **Apple Magic Keyboard** (AZERTY, QWERTY, etc.) on **Windows**, the symbols printed on the keycaps don't match what appears on screen. For example, on a French Magic Keyboard, pressing the key labeled `!` produces `_` instead.

This happens because Apple and Windows use different keyboard layouts for the same language. MagicWindows fixes this by installing a custom Windows keyboard layout that matches your Apple keycaps exactly.

> **Le problème** : Quand vous utilisez un **Apple Magic Keyboard** sur **Windows**, les symboles imprimés sur les touches ne correspondent pas à ce qui s'affiche à l'écran. Par exemple, sur un Magic Keyboard français, appuyer sur la touche `!` produit `_`. MagicWindows corrige cela en installant une disposition clavier Windows personnalisée qui correspond exactement aux inscriptions de votre Magic Keyboard.

## Features | Fonctionnalités

- **Auto-detection** — The app detects which Apple keyboard you have by asking you to press a few keys
- **Multiple layouts** — Support for various Apple keyboard layouts (French AZERTY, and more coming soon)
- **Native Windows integration** — Installs as a standard Windows keyboard layout, selectable in Settings alongside other keyboards
- **Multi-keyboard friendly** — Use Win+Space to switch between your Apple layout and other keyboards
- **Visual preview** — See exactly what each key will produce before installing
- **Bilingual UI** — Full French and English interface
- **Open source** — Fully transparent, community-driven

## Quick Start | Démarrage rapide

### Download | Télécharger

Download the latest installer from [**Releases**](https://github.com/ObzCureDev/Magicwindows/releases).

### Install | Installer

1. Run the MagicWindows installer | *Lancez l'installeur MagicWindows*
2. Choose "Auto-detect" or select your keyboard manually | *Choisissez "Auto-détection" ou sélectionnez manuellement*
3. Preview the layout and click "Install" | *Visualisez la disposition et cliquez sur "Installer"*
4. Go to **Settings > Time & Language > Language & Region > Keyboard** | *Paramètres > Heure et langue > Langue et région > Clavier*
5. Your new layout is now available | *Votre nouvelle disposition est disponible*
6. Use **Win+Space** to switch between keyboards | *Utilisez Win+Espace pour basculer*

## Key Differences (French AZERTY)

Here are the main keys that differ between Apple's French AZERTY and Windows' default French layout:

| Key Position | Apple Keycap | Windows Default | Fixed by MagicWindows |
|---|---|---|---|
| Left of `1` | `@` / `#` | `²` | `@` / `#` |
| Between `5` and `7` | `§` / `6` | `-` / `6` | `§` / `6` |
| Between `7` and `9` | `!` / `8` | `_` / `8` | `!` / `8` |
| Right of `°` | `-` / `_` | `=` / `+` | `-` / `_` |
| Right of `^` | `$` / `*` | `$` / `£` | `$` / `*` |
| Right of `M` | `ù` / `%` | `ù` / `%` | `ù` / `%` |
| Right of `ù` | `` ` `` / `£` | `*` / `µ` | `` ` `` / `£` |

Plus many AltGr (Option) layer differences for special characters like `{`, `}`, `|`, `\`, `~`, `€`, etc.

## Supported Layouts | Dispositions supportées

| Layout | Status |
|---|---|
| French AZERTY (Apple Magic Keyboard) | Available |
| US QWERTY (Apple Magic Keyboard) | Planned |
| UK QWERTY (Apple Magic Keyboard) | Planned |
| German QWERTZ (Apple Magic Keyboard) | Planned |

Want to add your layout? See [Contributing](#contributing).

## How It Works

MagicWindows creates a standard Windows keyboard layout (`.klc` file compiled into a DLL) that maps each physical key to the character printed on your Apple keyboard's keycaps. The layout is registered in Windows just like any built-in keyboard, so you can select it in Settings and switch to it with Win+Space.

### Important Notes

- **Function keys (F1-F12)** — MagicWindows handles character layout only. Function key behavior (F1-F12 vs. brightness/volume) is controlled by the keyboard firmware. Use the `fn` key on your Magic Keyboard to toggle, or see [Magic Utilities](https://magicutilities.net) for more control.
- **Compact vs Full-size** — Both Magic Keyboard variants use the same character layout. A single installation works for both.
- **Admin rights required** — Installing a keyboard layout requires administrator privileges (the DLL is copied to System32).

## Build from Source | Compiler depuis les sources

### Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) 1.77+
- [Tauri CLI](https://tauri.app/start/create-project/) (`npm install -D @tauri-apps/cli`)

### Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

The production build creates installers in `src-tauri/target/release/bundle/`.

## Project Structure

```
MagicWindows/
├── src/                  # Svelte 5 frontend
│   ├── pages/            # App pages (Welcome, Detect, Select, Preview, Install)
│   ├── components/       # Reusable components (KeyboardVisual)
│   └── lib/              # Types, i18n, state management
├── src-tauri/            # Rust backend (Tauri v2)
│   └── src/keyboard/     # Detection, KLC generation, installation
├── layouts/              # Keyboard layout definitions (JSON)
├── scripts/              # PowerShell install/uninstall scripts
└── .github/workflows/    # CI/CD
```

## Contributing

Contributions are welcome! Here's how you can help:

### Adding a New Layout

1. Create a new JSON file in `layouts/` following the schema in `layouts/schema.json`
2. Use an existing layout (like `apple-fr-azerty.json`) as a template
3. Map each physical key's scancode to the correct character from your Apple keyboard
4. Add 3-5 detection keys (keys that are distinctive to your layout)
5. Submit a pull request

See [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) for detailed instructions.

### Reporting a Key Mismatch

If a key doesn't produce the expected character, [open an issue](https://github.com/ObzCureDev/Magicwindows/issues/new) with:
- Your keyboard model (compact or full-size)
- The key you pressed (physical position)
- What character you expected (printed on keycap)
- What character was produced

## Tech Stack

| Component | Technology |
|---|---|
| Frontend | [Svelte 5](https://svelte.dev/) + TypeScript |
| Backend | [Rust](https://www.rust-lang.org/) + [Tauri 2](https://tauri.app/) |
| Layout format | Microsoft KLC (Keyboard Layout Creator) |
| Installer | MSI / NSIS via Tauri bundler |
| CI/CD | GitHub Actions |

## License

[Apache License 2.0](LICENSE)

---

Made with care by [ObzCure](https://github.com/ObzCureDev)
