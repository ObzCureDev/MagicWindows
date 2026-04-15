# Contributing to MagicWindows | Contribuer a MagicWindows

## Adding a New Apple Keyboard Layout | Ajouter une nouvelle disposition

### 1. Identify Your Layout

Determine which Apple keyboard layout you have. Check the keycaps on your physical keyboard and note the language/region.

### 2. Create the Layout JSON

Create a new file in `layouts/` following the naming convention `apple-{language}-{type}.json`. For example:
- `apple-fr-azerty.json` (French AZERTY)
- `apple-us-qwerty.json` (US QWERTY)
- `apple-de-qwertz.json` (German QWERTZ)

Use `layouts/apple-fr-azerty.json` as a template.

### 3. Map the Keys

For each key on your Apple keyboard:

1. Find the **scancode** (physical key position). Refer to the [scancode reference](https://kbdlayout.info/KBDFR/) for ISO keyboard layout scancodes.
2. Note the character printed on the keycap for each state:
   - **Base**: No modifier
   - **Shift**: With Shift held
   - **AltGr**: With Right Alt (= Apple Option key)
   - **AltGr+Shift**: With Right Alt + Shift

3. Convert each character to its Unicode hex codepoint. For example:
   - `@` = `0040`
   - `!` = `0021`
   - `e` = `00e9`
   - Use `-1` for "no character produced"

4. For dead keys (accent keys that modify the next keystroke), append `@` to the value. For example, `^` as a dead key = `005e@`.

### 4. Add Detection Keys

Choose 3-5 keys that are **distinctive** to your layout. These should be keys where the Apple character differs from the Windows default for that language. The auto-detection wizard will use these to identify keyboards.

### 5. Test

1. Run the app in development mode: `npm run tauri dev`
2. Verify your layout appears in the selection list
3. Check the visual preview matches your physical keyboard
4. If possible, test the actual installation on Windows

### 6. Submit

Open a pull request with:
- Your new `layouts/*.json` file
- A brief description of the keyboard model and language
- Photos of your keyboard (optional but helpful)

## Reporting Issues | Signaler un probleme

If a key produces the wrong character after installing a layout, please open an issue with:

| Field | Example |
|---|---|
| Layout used | `apple-fr-azerty` |
| Keyboard model | Magic Keyboard compact / with numpad |
| Key position | Row 1, key 8 (between 7 and 9) |
| Keycap label | `!` |
| Expected character | `!` (U+0021) |
| Actual character | `_` (U+005F) |

## Development Setup

```bash
# Prerequisites: Node.js 20+, Rust 1.77+

# Clone and install
git clone https://github.com/ObzCureDev/Magicwindows.git
cd Magicwindows
npm install

# Run in development
npm run tauri dev

# Build
npm run tauri build
```

## Code Style

- **Rust**: Follow standard Rust conventions (`cargo fmt`, `cargo clippy`)
- **TypeScript/Svelte**: Use the project's existing formatting
- **JSON layouts**: Follow the schema in `layouts/schema.json`
