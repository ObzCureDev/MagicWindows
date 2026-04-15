# MagicWindows

**Install Apple Magic Keyboard layouts on Windows** | **Installer les dispositions Apple Magic Keyboard sur Windows**

[![Build](https://github.com/ObzCureDev/Magicwindows/actions/workflows/build.yml/badge.svg)](https://github.com/ObzCureDev/Magicwindows/actions/workflows/build.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

---

## The Problem | Le Probleme

When you use an **Apple Magic Keyboard** (AZERTY, QWERTY, etc.) on **Windows**, the symbols printed on the keycaps don't match what appears on screen. For example, on a French Magic Keyboard, pressing the key labeled `!` produces `_` instead.

This happens because Apple and Windows use different keyboard layouts for the same language. MagicWindows fixes this by installing a custom Windows keyboard layout that matches your Apple keycaps exactly.

---

**Quand vous utilisez un Apple Magic Keyboard** (AZERTY, QWERTY, etc.) **sur Windows**, les symboles imprimes sur les touches ne correspondent pas a ce qui s'affiche a l'ecran. Par exemple, sur un Magic Keyboard francais, appuyer sur la touche `!` produit `_`.

Cela se produit car Apple et Windows utilisent des dispositions clavier differentes pour la meme langue. MagicWindows corrige cela en installant une disposition clavier Windows personnalisee qui correspond exactement aux inscriptions de votre Magic Keyboard.

## Features | Fonctionnalites

- **Auto-detection**: The app detects which Apple keyboard you have by asking you to press a few keys
- **Multiple layouts**: Support for various Apple keyboard layouts (French AZERTY, and more coming soon)
- **Native Windows integration**: Installs as a standard Windows keyboard layout — selectable in Settings alongside other keyboards
- **Multi-keyboard friendly**: Use Win+Space to switch between your Apple layout and other keyboards
- **Visual preview**: See exactly what each key will produce before installing
- **Open source**: Fully transparent, community-driven

---

- **Auto-detection** : L'application detecte votre clavier Apple en vous demandant d'appuyer sur quelques touches
- **Plusieurs dispositions** : Support de differentes dispositions Apple (AZERTY francais, et d'autres a venir)
- **Integration native Windows** : S'installe comme un clavier Windows standard, selectionnable dans les Parametres
- **Multi-clavier** : Utilisez Win+Espace pour basculer entre la disposition Apple et vos autres claviers
- **Apercu visuel** : Voyez exactement ce que chaque touche produira avant l'installation
- **Open source** : Entierement transparent, communautaire

## Quick Start | Demarrage rapide

### Download | Telecharger

Download the latest installer from [Releases](https://github.com/ObzCureDev/Magicwindows/releases).

Telechargez le dernier installeur depuis [Releases](https://github.com/ObzCureDev/Magicwindows/releases).

### Install | Installer

1. Run the MagicWindows installer
2. Choose "Auto-detect" or select your keyboard model manually
3. Preview the layout and click "Install"
4. Go to **Settings > Time & Language > Language & Region > French > Keyboard**
5. Your new layout "French - Apple Magic Keyboard (AZERTY)" is now available
6. Use **Win+Space** to switch between keyboards

---

1. Lancez l'installeur MagicWindows
2. Choisissez "Auto-detection" ou selectionnez votre modele de clavier manuellement
3. Visualisez la disposition et cliquez sur "Installer"
4. Allez dans **Parametres > Heure et langue > Langue et region > Francais > Clavier**
5. Votre nouvelle disposition "Francais - Apple Magic Keyboard (AZERTY)" est disponible
6. Utilisez **Win+Espace** pour basculer entre les claviers

## Key Differences (French AZERTY) | Differences principales

Here are the main keys that differ between Apple's French AZERTY and Windows' default French layout:

| Key Position | Apple Keycap | Windows Default | Fixed by MagicWindows |
|---|---|---|---|
| Left of `1` | `@` / `#` | `2` | `@` / `#` |
| Between `5` and `7` | `\u00a7` / `6` | `-` / `6` | `\u00a7` / `6` |
| Between `7` and `9` | `!` / `8` | `_` / `8` | `!` / `8` |
| Right of `0` (2nd) | `-` / `+` | `=` / `+` | `-` / `+` |
| Right of `^` | `$` / `*` | `$` / `\u00a3` | `$` / `*` |

Plus many AltGr (Option) layer differences for special characters like `{`, `}`, `|`, `\`, `~`, etc.

## Supported Layouts | Dispositions supportees

| Layout | Status |
|---|---|
| French AZERTY (Apple Magic Keyboard) | Available |
| US QWERTY (Apple Magic Keyboard) | Planned |
| UK QWERTY (Apple Magic Keyboard) | Planned |
| German QWERTZ (Apple Magic Keyboard) | Planned |

Want to add your layout? See [Contributing](#contributing).

## How It Works | Comment ca marche

MagicWindows creates a standard Windows keyboard layout (`.klc` file compiled into a DLL) that maps each physical key to the character printed on your Apple keyboard's keycaps. The layout is registered in Windows just like any built-in keyboard, so you can select it in Settings and switch to it with Win+Space.

### Important Notes | Notes importantes

- **Function keys (F1-F12)**: MagicWindows handles character layout only. Function key behavior (F1-F12 vs. brightness/volume) is controlled by the keyboard firmware. Use the `fn` key on your Magic Keyboard to toggle, or see [Magic Utilities](https://magicutilities.net) for more control.
- **Compact vs Full-size**: Both Magic Keyboard variants use the same character layout. A single installation works for both.
- **Admin rights required**: Installing a keyboard layout requires administrator privileges (the DLL is copied to System32).

---

- **Touches de fonction (F1-F12)** : MagicWindows gere uniquement la disposition des caracteres. Le comportement des touches de fonction est controle par le firmware du clavier. Utilisez la touche `fn` pour basculer.
- **Compact vs Pleine taille** : Les deux variantes utilisent la meme disposition. Une seule installation suffit.
- **Droits administrateur** : L'installation necessite les droits administrateur.

## Build from Source | Compiler depuis les sources

### Prerequisites | Prerequis

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) 1.77+
- [Tauri CLI](https://tauri.app/start/create-project/) (`npm install -D @tauri-apps/cli`)

### Development | Developpement

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

The production build creates installers in `src-tauri/target/release/bundle/`.

## Project Structure | Structure du projet

```
MagicWindows/
\u251c\u2500\u2500 src/                  # Svelte frontend
\u2502   \u251c\u2500\u2500 pages/            # App pages (Welcome, Detect, Select, Preview, Install)
\u2502   \u251c\u2500\u2500 components/       # Reusable components (KeyboardVisual)
\u2502   \u2514\u2500\u2500 lib/              # Types, i18n, state management
\u251c\u2500\u2500 src-tauri/            # Rust backend (Tauri)
\u2502   \u2514\u2500\u2500 src/keyboard/     # Detection, KLC generation, installation
\u251c\u2500\u2500 layouts/              # Keyboard layout definitions (JSON)
\u251c\u2500\u2500 scripts/              # PowerShell install/uninstall scripts
\u2514\u2500\u2500 .github/workflows/    # CI/CD
```

## Contributing

Contributions are welcome! Here's how you can help:

### Adding a New Layout | Ajouter une disposition

1. Create a new JSON file in `layouts/` following the schema in `layouts/schema.json`
2. Use an existing layout (like `apple-fr-azerty.json`) as a template
3. Map each physical key's scancode to the correct character from your Apple keyboard
4. Add 3-5 detection keys (keys that are distinctive to your layout)
5. Submit a pull request

### Reporting a Key Mismatch | Signaler une erreur

If a key doesn't produce the expected character, [open an issue](https://github.com/ObzCureDev/Magicwindows/issues/new) with:
- Your keyboard model (compact or full-size)
- The key you pressed (physical position)
- What character you expected (printed on keycap)
- What character was produced

## Tech Stack

- **Frontend**: [Svelte 5](https://svelte.dev/) + TypeScript
- **Backend**: [Rust](https://www.rust-lang.org/) + [Tauri 2](https://tauri.app/)
- **Layout format**: Microsoft KLC (Keyboard Layout Creator)
- **Installer**: MSI / NSIS via Tauri bundler

## License

[Apache License 2.0](LICENSE)

---

Made with care by [ObzCure](https://github.com/ObzCureDev)
