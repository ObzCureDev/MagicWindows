#Requires -Version 5.1
<#
.SYNOPSIS
    Fixes the Claude Code x VSCode "Shift+Enter does nothing / Enter types m"
    issue by repairing a bad VSCode user keybinding.

.DESCRIPTION
    Older versions of Claude Code's `/terminal-setup` wrote a VSCode user
    keybinding that sends ESC+CR (`\u001b\r`) when Shift+Enter is pressed
    inside the integrated terminal. Claude's input parses that as Alt+Enter
    (no binding, so Shift+Enter appears dead), and the residual `\r` can
    surface as Ctrl+M / the literal letter "m" in the chat input.

    This script locates %APPDATA%\Code\User\keybindings.json, detects the
    bad `"text": "\u001b\r"` value, and replaces it with `"\n"` (LF = Ctrl+J,
    the default `chat:newline` binding).

    Not a MagicWindows bug and not specific to this project. It affects any
    developer running Claude Code CLI inside VSCode's integrated terminal
    with an old `/terminal-setup` config. Standalone, user-level, no admin
    required.

.PARAMETER DryRun
    Report what would change without writing any files.

.EXAMPLE
    .\Fix-ClaudeCodeShiftEnter.ps1
    .\Fix-ClaudeCodeShiftEnter.ps1 -DryRun
#>

[CmdletBinding()]
param(
    [switch]$DryRun
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$ConfigPath = Join-Path $env:APPDATA 'Code\User\keybindings.json'

if (-not (Test-Path -LiteralPath $ConfigPath)) {
    Write-Host "VSCode keybindings.json not found at: $ConfigPath"
    Write-Host "Nothing to fix."
    exit 0
}

$raw = Get-Content -LiteralPath $ConfigPath -Raw -Encoding UTF8
if ($null -eq $raw) { $raw = '' }

# Literal-text match on the canonical bad "text" value. We deliberately do
# not JSON-parse the file: keybindings.json allows `//` comments (JSONC),
# and a regex replace preserves every other binding, comment, and whitespace
# the user has.
$pattern     = '"text"\s*:\s*"\\u001[bB]\\r"'
$replacement = '"text": "\n"'

$hits = [regex]::Matches($raw, $pattern)
if ($hits.Count -eq 0) {
    Write-Host "No bad Shift+Enter binding found."
    Write-Host "Checked: $ConfigPath"
    exit 0
}

Write-Host "Found $($hits.Count) bad binding(s) in $ConfigPath"
foreach ($h in $hits) {
    Write-Host "  $($h.Value)  ->  $replacement"
}

if ($DryRun) {
    Write-Host ""
    Write-Host "Dry run - no changes written."
    exit 0
}

$backup = "$ConfigPath.bak-$(Get-Date -Format 'yyyyMMddHHmmss')"
Copy-Item -LiteralPath $ConfigPath -Destination $backup

$new = [regex]::Replace($raw, $pattern, $replacement)

# UTF-8 without BOM keeps VSCode's own serialisation style.
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText($ConfigPath, $new, $utf8NoBom)

Write-Host ""
Write-Host "Fixed. Backup saved to: $backup"
Write-Host "Restart your VSCode terminal (or VSCode itself) for the change to take effect."
