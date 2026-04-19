# Settings Page + Elevated Error Flow

**Date:** 2026-04-19
**Status:** Approved — ready for implementation plan
**Adds to:** layout install/uninstall, Done page, new top-bar Settings page

## Problem

Three separate gaps in the current install flow came up from user testing:

1. **No way to manage layouts from inside the app.** Once installed, the user has to open Windows Settings → Region → Language → Options to inspect or remove any keyboard layout. Microsoft's default layouts (e.g., `KBDFR.DLL` at KLID `0000040c`) sit next to ours in the Win+Space picker and invite confusion.

2. **No "tidy up" shortcut after install.** When we've just installed our `FR MagicKeyboard`, the user often wants to remove the Windows default `French (France)` layout so only ours is reachable. Today they must do this manually in Windows Settings.

3. **UAC cancellation produces raw red error output.** If the user closes the UAC prompt (or PowerShell window) mid-install, they see technical PowerShell stderr in red inside the app with no affordance to retry. There is no path to report the failure.

## Solution

Three coordinated additions:

- **New Settings page** (top-bar ⚙ icon) listing every installed keyboard layout with per-row remove buttons.
- **New post-install prompt on the Done page** offering to remove the Windows-default layout for the locale we just installed (one click, same backend).
- **New reusable `ElevatedErrorPanel` component** replacing the current raw error display for all elevated operations (install, uninstall, uninstall-by-klid, modifier scancode map writes). First failure shows a friendly message + **Retry**. Second failure adds **Send bug report** which opens the user's default email client via `mailto:` pre-filled with collected diagnostics.

No feature is conditional on the others; each can be shipped independently if needed.

## Section 1 — Settings page

### UI

- New `"settings"` value added to the `Page` union in [src/lib/types.ts](src/lib/types.ts).
- New icon button in the top-bar (between the ⌘ modifiers icon and the theme toggle), icon `⚙` (U+2699), `aria-label` translated via i18n.
- New file [src/pages/Settings.svelte](src/pages/Settings.svelte).

### Content

On mount, calls `list_all_keyboard_layouts()` and renders one card per layout:

- **Title:** `layoutText` (falls back to `klid` if empty)
- **Details line:** `layoutFile` · `klid`
- **Badges (inline):**
  - `MagicWindows` (blue) when `isMagicWindows === true`
  - `Système` (gray) when `klid` starts with `0000` (i.e., OEM/built-in)
  - `En usage` (green) when `isInUse === true`
- **Actions:**
  - `Retirer` button (red/danger)
  - `ℹ` icon next to Retirer — hover/click tooltip explaining how to re-enable a removed system layout via Windows Settings (`Paramètres Windows > Heure et langue > Langue > [langue] > Options > Ajouter un clavier`).

Empty state: if only MagicWindows layouts or none exist, show an informational line.

### Remove flow (Settings page)

- Clicking `Retirer` opens a `window.confirm` with the layout name.
- On confirm:
  - If `isMagicWindows === true` → call existing `uninstall_layout(id)` (cleans registry + removes the DLL from System32 + purges HKCU).
  - Else → call new `uninstall_layout_by_klid(klid)` (registry only, DLL in System32 untouched).
- Errors from either call flow through the `ElevatedErrorPanel` pinned at the top of the Settings page.
- After success, the list reloads automatically (re-call `list_all_keyboard_layouts()`).

## Section 2 — Done page "Remove Windows default"

### UI (addition to [src/pages/Done.svelte](src/pages/Done.svelte))

A new bloc rendered between the success message and the existing modifier-preset offer. Only appears when **both** of these are true:

- `list_all_keyboard_layouts()` includes a layout with `klid` starting with `0000` whose locale hex (last 4 chars) matches the locale of the layout we just installed (e.g., we installed FR → look for `0000040c`).
- That layout has **not yet been removed** in this session (tracked via local state `$state` var).

Content:

> **Layout Windows détecté** ℹ
>
> Windows a encore son layout français standard (`KBDFR.DLL`) installé. Le retirer évite toute confusion lors du switch via Win+Space.
>
> **[Retirer le layout Windows FR]**

The `ℹ` is the same tooltip as the Settings page (how to re-enable).

### Behaviour

- Click → `window.confirm` with the layout name.
- On confirm → `uninstall_layout_by_klid("0000040c")` (or whichever klid matches).
- Errors flow through an `ElevatedErrorPanel` specific to this operation.
- After success, the bloc disappears (sets local `removed = true`) and a small transient "Layout retiré ✓" message is shown.

## Section 3 — `ElevatedErrorPanel` component

### File

[src/components/ElevatedErrorPanel.svelte](src/components/ElevatedErrorPanel.svelte)

### Props

```ts
{
  error: string | null;                // raw PS error / stderr; null hides the panel
  onRetry: () => Promise<void>;        // parent's retry callback
  operationName: string;               // e.g. "install_layout" — used in mailto subject
  context: Record<string, any>;        // free-form JSON-serialisable; included in bug report
  attemptCount: number;                // owned by the parent; panel renders Send bug report when >= 2
}
```

### Internal state

- `submitting: boolean` — true while `onRetry` is in flight; disables buttons.
- `sending: boolean` — true while the mailto handler is running (collecting diagnostics then opening the client).

`attemptCount` is a prop, not internal state. The parent increments it on each failed attempt so the same panel instance can be reused across multiple retries with the same instance state.

### Rendered UI

- ⚠ icon + title: `Une erreur est survenue` / `Something went wrong` (i18n).
- Friendly sub-text, heuristic:
  - If `error` matches (case-insensitive) `admin | privilege | uac | access denied | cancelled` → "MagicWindows n'a pas reçu les droits administrateur. Cela peut arriver si vous avez fermé la fenêtre UAC ou cliqué Non."
  - Else → "Une erreur inattendue s'est produite. Réessayez, et si le problème persiste, envoyez-nous un rapport."
- Always: button `[Réessayer]` (disabled while `submitting`).
- Only when `attemptCount >= 2`: button `[Envoyer un rapport de bug 📧]` (disabled while `sending`).
- `<details>` collapsible labelled "Détails techniques" / "Technical details", containing the raw `error` string in a `<pre>`.

### Retry flow

- Clicking `[Réessayer]` sets `submitting = true`, awaits `onRetry()`.
- If `onRetry` throws or the parent re-sets `error` to non-null, `attemptCount` is incremented by the parent. (See "Parent contract" below.)
- If `onRetry` succeeds, the parent sets `error = null` which hides the panel.

### Bug-report flow

- Clicking `[Envoyer un rapport de bug]`:
  1. Sets `sending = true`.
  2. Calls Tauri command `collect_diagnostics()` → `String` (Markdown block).
  3. Builds `mailto:` URL:
     - `to = bug@mindvisionstudio.com`
     - `subject = [MagicWindows Bug] {operationName} failed`
     - `body = <bodyIntro from i18n> + "\n\n" + diagnostics + "\n\n" + "--- Votre commentaire ---\n"`
  4. URL-encodes subject & body, opens via `@tauri-apps/plugin-shell` → `open()`.
  5. Sets `sending = false`.
- The user's default email client opens with a pre-filled draft. **The user must click Send manually** — we never send on their behalf.

### Parent contract

Pages using the component own the retry logic. Pattern (example for Install):

```ts
let error = $state<string | null>(null);
let attemptCount = $state(0);

async function runInstall() {
  try {
    await invoke("install_layout", { id });
    error = null;
  } catch (e) {
    error = String(e);
    attemptCount += 1;
  }
}

async function handleRetry() {
  await runInstall();
}
```

The panel is rendered with `{error}`, `onRetry={handleRetry}`, and `attemptCount={attemptCount}`. Keeping `attemptCount` on the parent lets a single page instance multiple panels (e.g. Done page's modifier-apply + remove-Windows-default) with independent counters.

### Replaces

The existing raw error displays in:
- [src/pages/Install.svelte](src/pages/Install.svelte)
- The modifier-apply error in [src/pages/Done.svelte](src/pages/Done.svelte)
- The uninstall error in [src/pages/Done.svelte](src/pages/Done.svelte)

And is used fresh in:
- [src/pages/Settings.svelte](src/pages/Settings.svelte) (for per-layout removes)
- [src/pages/Done.svelte](src/pages/Done.svelte) (for the new "remove Windows default" bloc)

## Section 4 — Backend: Tauri commands

### `list_all_keyboard_layouts() -> Vec<InstalledLayoutInfo>`

- Non-elevated. Reads `HKLM\SYSTEM\CurrentControlSet\Control\Keyboard Layouts` subkeys (world-readable).
- For each subkey, reads `Layout File` and `Layout Text` values.
- Checks `HKCU\Keyboard Layout\Preload` for `isInUse`.
- Returns a `Vec<InstalledLayoutInfo>`:

```rust
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstalledLayoutInfo {
    pub klid: String,              // subkey name, e.g. "0000040c"
    pub layout_file: String,       // e.g. "KBDFR.DLL" or "kbdaplfr.dll"
    pub layout_text: String,       // human-readable, empty string if registry value missing
    pub is_magic_windows: bool,    // layout_file.to_lowercase().starts_with("kbdapl")
    pub is_in_use: bool,           // found in HKCU Preload
}
```

### `uninstall_layout_by_klid(klid: String) -> Result<(), String>`

- Elevated via the same `run_elevated_ps` helper already in [src-tauri/src/keyboard/install.rs](src-tauri/src/keyboard/install.rs).
- PowerShell script:
  - Validates `klid` format (8 hex chars).
  - Removes `HKLM\SYSTEM\CurrentControlSet\Control\Keyboard Layouts\{klid}` (registry subkey only; DLL in System32 is never touched).
  - Writes `LastUninstalledKLIDs = {klid}` to `HKLM\SOFTWARE\MagicWindows`.
- After the elevated step, runs the existing unprivileged `purge_hkcu_after_uninstall` helper which reads the marker and cleans HKCU\Preload + InputMethodTips.

Refactor note: extract the PS body of the existing `uninstall_layout` registry-removal step into a shared helper so both commands (ours + by-klid) reuse the same logic. Keep `uninstall_layout(id)` doing the extra `Remove-Item` for System32 DLL.

### `collect_diagnostics() -> String`

Non-elevated. Returns a Markdown block:

```
## MagicWindows Bug Report

**App version:** {cargo pkg version}
**OS:** {cmd /c ver output, trimmed}
**Active layout (HKL):** 0x{hex of GetKeyboardLayout}

**Installed MagicWindows layouts:**
- {klid} / {layout_file} / "{layout_text}"
- ...

**All installed keyboard layouts:**
- {klid} / {layout_file} / "{layout_text}"
- ...

**Scancode Map (modifiers):**
{output of read_scancode_map or "(none)"}

**Last elevated PS transcript (tail 100 lines):**
```
(paste from %LOCALAPPDATA%\MagicWindows\layouts\install_transcript.txt or similar if present)
```
```

Collected via existing plumbing:
- App version: `env!("CARGO_PKG_VERSION")`.
- OS: `Command::new("cmd").args(["/c", "ver"])`.
- Active HKL: `GetKeyboardLayout(0)` via `windows` crate or PowerShell fallback.
- Layouts: reuse `list_all_keyboard_layouts()`.
- Scancode Map: reuse `read_scancode_map()`.
- PS transcript: read the most recent `*_transcript.txt` from `get_install_dir()`, last 100 lines.

If any collector fails, include `*(collector failed: {error})*` instead of the section — never fail the whole diagnostics call.

## Section 5 — Frontend plumbing

### New i18n keys in [src/lib/i18n.ts](src/lib/i18n.ts)

| Key | EN | FR |
|-----|----|----|
| `settings.topbarTitle` | Settings | Paramètres |
| `settings.title` | Installed keyboard layouts | Layouts installés |
| `settings.subtitle` | Review and remove any keyboard layout on your system. | Vérifiez et retirez n'importe quel layout clavier de votre système. |
| `settings.badgeMagic` | MagicWindows | MagicWindows |
| `settings.badgeSystem` | System | Système |
| `settings.badgeInUse` | In use | En usage |
| `settings.remove` | Remove | Retirer |
| `settings.reactivateInfo` | To re-enable: Windows Settings → Time & language → Language → [language] → Options → Add a keyboard. | Pour réactiver : Paramètres Windows → Heure et langue → Langue → [langue] → Options → Ajouter un clavier. |
| `settings.confirmRemove` | Remove "{name}"? | Retirer « {name} » ? |
| `settings.empty` | No keyboard layouts found. | Aucun layout clavier trouvé. |
| `done.removeWindowsDefault` | Windows still has its default {locale} layout installed. Removing it prevents confusion when switching via Win+Space. | Windows a encore son layout {locale} standard installé. Le retirer évite toute confusion lors du changement via Win+Space. |
| `done.removeWindowsDefaultBtn` | Remove the Windows default {locale} layout | Retirer le layout Windows {locale} par défaut |
| `done.windowsDefaultRemoved` | Windows layout removed ✓ | Layout Windows retiré ✓ |
| `elevatedError.title` | Something went wrong | Une erreur est survenue |
| `elevatedError.uacHint` | MagicWindows didn't receive administrator rights. This can happen if you closed the UAC prompt or clicked No. | MagicWindows n'a pas reçu les droits administrateur. Cela peut arriver si vous avez fermé la fenêtre UAC ou cliqué Non. |
| `elevatedError.generic` | An unexpected error occurred. Retry, and if the problem persists, send us a report. | Une erreur inattendue s'est produite. Réessayez, et si le problème persiste, envoyez-nous un rapport. |
| `elevatedError.retry` | Retry | Réessayer |
| `elevatedError.sendReport` | Send bug report | Envoyer un rapport de bug |
| `elevatedError.techDetails` | Technical details | Détails techniques |
| `bugReport.subject` | [MagicWindows Bug] {op} failed | [MagicWindows Bug] {op} failed |
| `bugReport.bodyIntro` | Hello, an error occurred while running MagicWindows. Full diagnostics below — feel free to add context above. | Bonjour, une erreur est survenue lors de l'utilisation de MagicWindows. Diagnostics complets ci-dessous — n'hésitez pas à ajouter du contexte au-dessus. |

### New top-bar button in [src/App.svelte](src/App.svelte)

Added between the ⌘ modifiers button and the theme toggle:

```html
<button
  class="theme-toggle"
  onclick={() => (appState.page = "settings")}
  title={t(appState.lang, "settings.topbarTitle")}
  aria-label={t(appState.lang, "settings.topbarTitle")}
>⚙</button>
```

And a `{:else if appState.page === "settings"}<Settings />` branch in the router.

## Edge cases

- **Removing the active layout.** If the user removes the layout currently active in their session (HKCU Preload has it), Windows will fall back to the next-in-list. If there's no next-in-list, Windows restores en-US by default. The Settings page shows a confirm dialog with an extra line when `isInUse === true`: *"Ce layout est actuellement utilisé par votre session."*
- **Removing the only installed layout.** Same fallback applies. We don't special-case this further — Windows handles it.
- **Protected system DLLs.** We never attempt `Remove-Item` on anything in `System32`. Removing Microsoft-owned registry entries for a built-in layout is fully reversible from Windows Settings.
- **No email client configured.** `mailto:` opens whatever Windows has registered. On fresh Windows 11 installs this can be "no app associated". We accept this limitation for v1 — v2 may add a HTTP webhook fallback.
- **Very large PS transcripts.** Truncate to last 100 lines (~10 KB) before inserting into the mailto body to stay under URL length limits. If truncated, prepend `*(transcript truncated — full log at {path})*`.
- **Concurrent elevated operations.** The ElevatedErrorPanel does not currently guard against two retries firing in parallel from different callers; parents are responsible for disabling their trigger while `submitting === true`. The existing install flow already does this via `installing` boolean.
- **`collect_diagnostics` failure.** Never throws. If individual sections fail, inline an `*(collector failed: ...)*` marker. Mailto still opens so the user can at least report.

## Testing

### Rust unit tests (in [src-tauri/src/keyboard/install.rs](src-tauri/src/keyboard/install.rs))

- `list_all_keyboard_layouts_returns_at_least_one` — sanity check against a real Windows registry. (Keep behind `#[cfg(target_os = "windows")]`.)
- `is_magic_windows_detection` — unit test of the `layout_file.to_lowercase().starts_with("kbdapl")` classifier with mocked data.

### Manual QA checklist

1. Fresh install of our FR layout → Done page shows the "Retirer Windows FR" bloc → click → UAC approve → bloc disappears, "Layout retiré ✓" shows briefly.
2. Same flow but cancel UAC → ElevatedErrorPanel appears with Retry only. Click Retry, cancel again → Retry + Send bug report appear. Click Send bug report → default email client opens with pre-filled subject and body.
3. Open Settings page from top-bar → every installed layout is listed with correct badges. Remove a MagicWindows layout → works, DLL disappears from System32. Remove a system layout → works, DLL stays in System32. Verify `ℹ` tooltip content.
4. After removing the system FR layout via Settings, reopen Done of a fresh install → the bloc should NOT appear (already-removed).

## Out of scope

- HTTP bug-report submission (v2).
- Language-specific reactivation instructions (we show the Windows Settings path verbatim; Windows localises its own UI).
- "Restore" button for removed system layouts (one-way via Windows Settings is fine).
- Multi-select delete in the Settings page (YAGNI).
- Scancode Map removal from the Settings page (already handled on the Modifiers page).
