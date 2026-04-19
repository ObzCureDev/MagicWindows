# Troubleshooting | Dépannage

A practical guide for diagnosing keyboard issues after installing a MagicWindows layout. Most reports fall into three buckets: stale DLL cache, a third-party app doing its own key handling, or a real layout bug. The 3-test triage below tells you which in ~90 seconds.

## 3-Test Triage | Triage en 3 tests

Apply these in order. The first two use Windows-native apps (they route keys through `TranslateMessage`, which is the well-trodden path). The third re-tests in the app where you saw the bug.

### Test 1 — `cmd.exe` or PowerShell

Open a new `cmd.exe` or PowerShell window (not embedded in another app). Type a few letters, press `Enter`. Expected: new prompt line. If you see a stray character (e.g. `m`) → the DLL is actually broken. Skip to [DLL-level diagnostics](#dll-level-diagnostics).

### Test 2 — Notepad

Open Notepad. Type text, press `Enter`. Expected: line break. Same criterion — a stray character means the DLL is broken.

### Test 3 — The app where you saw the bug

Reproduce the original symptom.

### Reading the results

| Test 1 | Test 2 | Test 3 | Diagnosis |
|--------|--------|--------|-----------|
| OK     | OK     | KO     | **Third-party app bug** — the app handles keys itself (common in Chromium/Electron integrated terminals) and mis-routes the FR/DE/… layout. Not a MagicWindows bug. See [Known third-party quirks](#known-third-party-quirks). |
| KO     | KO     | KO     | **Real layout-level bug.** Most often stale DLL cache — follow [Stale DLL cache](#stale-dll-cache-reinstall--reload-session). If it persists after a clean reinstall + session reload, it's a genuine codegen or JSON mapping bug — open an issue with the triage output. |
| KO     | OK     | any    | Unusual — suggests a console-specific codepage issue. Include `chcp` output in the issue. |

## Stale DLL cache — reinstall + reload session

Windows keyboard layout DLLs are loaded by `winlogon` when your session starts. Overwriting the DLL in `System32` does **not** refresh the loaded copy — you need a session reload.

1. Launch MagicWindows → Settings (gear icon, top bar) → **Remove** the installed layout.
2. Return to the wizard → reinstall from scratch. This re-copies the latest DLL to `System32\kbdapl*.dll`.
3. **Log off and log back in** (not just restart the target app, not just a reboot of the app — Windows caches the DLL for the entire user session). A full Windows restart also works.
4. Re-run the 3 tests.

If the bug persists after this, it is not a cache issue.

## Known third-party quirks

Apps that implement their own keyboard handling on top of the OS routinely break on non-US layouts. Common culprits:

- **Chromium/Electron integrated terminals** (Antigravity, Cursor, older VSCode builds, some Slack-huddle chat inputs). Symptoms: `Enter` produces a letter, dead keys don't compose, AltGr combinations miss. These apps often use the raw [`event.code`](https://developer.mozilla.org/docs/Web/API/UI_Events/Keyboard_event_code_values) (which is physical-position based, assumes US QWERTY) rather than `event.key` or letting Windows route via `TranslateMessage`. They are the bug; we cannot fix them from the layout side without breaking Windows-native apps.
- **Remote Desktop / Citrix / RDP clients.** The client often enforces its own layout client-side. Switch the remote session's input method to the MagicWindows layout on the *server*, not via the client wrapper.
- **Some Java / Swing apps**. They read scancodes directly and assume a US physical layout. Workaround: none from the layout side.

When reporting to a third-party app, include: *"Installed Apple FR AZERTY via MagicWindows (custom `kbdaplfr.dll`). Enter produces the wrong character in the integrated terminal, but works correctly in `cmd.exe`, Notepad, and the chat input of the same app. This proves the layout DLL is OS-valid — the app's key handling is bypassing Windows routing."* That framing prevents them from blaming your custom layout.

## DLL-level diagnostics

If Tests 1 and 2 both fail, the DLL itself is suspect.

### Is the installed DLL the one I just built?

```powershell
Get-ChildItem "C:\Windows\System32\kbdapl*.dll" | Select Name, Length, LastWriteTime
Get-ChildItem "target\kbd_dlls\kbdapl*.dll"      | Select Name, Length, LastWriteTime
```

The `LastWriteTime` should match within a few seconds of your last `cargo build`. If the `System32` copy is older, you're running a pre-refactor DLL — reinstall.

### Does the generated C source match the codegen?

```bash
cd src-tauri
cargo test -p klc-codegen
```

All tests are golden-output tests that regenerate the C source per layout and assert key invariants (`aVkToWch2` table present, `VK_RETURN` mapped to `{0x000D, 0x000A}`, etc.). A failing test here means the codegen itself drifted.

### Inspect what's in the current Scancode Map

Mac-style modifier remap writes a [`Scancode Map`](https://learn.microsoft.com/windows-hardware/drivers/kernel/scan-codes) to `HKLM\System\CurrentControlSet\Control\Keyboard Layout`. Rule it out:

```powershell
$b = (Get-ItemProperty -Path 'HKLM:\System\CurrentControlSet\Control\Keyboard Layout' `
        -Name 'Scancode Map' -ErrorAction SilentlyContinue).'Scancode Map'
if ($b) { ($b | ForEach-Object { '{0:X2}' -f $_ }) -join ' ' } else { 'No Scancode Map set.' }
```

The first 8 bytes are zeroed header, next 4 bytes are the entry count (little-endian), then 4-byte entries of `[new_scancode(2) old_scancode(2)]`. The MagicWindows-written maps only touch modifier scancodes (`0x1D` LCtrl, `0x38` LAlt, `0x3A` CapsLock, `0x5B/0x5C` L/RWin). If you see entries touching non-modifier scancodes, something else on the system wrote them — clear via the app's Settings page.

## Collecting a good bug report

If none of the above resolves the issue, open an issue with:

1. **Triage table** from the 3 tests above (which passed, which failed).
2. **Which layout** (`apple-fr-azerty`, `apple-us-qwerty`, …).
3. **DLL check** output (both `System32\kbdapl*.dll` and `target\kbd_dlls\kbdapl*.dll` sizes + mtimes).
4. **Scancode Map dump** (PowerShell snippet above).
5. **`cargo test -p klc-codegen`** result.
6. **Windows build** (`winver`) and **Apple keyboard model** (e.g. "Magic Keyboard A1314, UK ISO variant").

That's enough context to diagnose >90% of reports without a back-and-forth.
