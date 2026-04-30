<#
.SYNOPSIS
    Installs an Apple Magic Keyboard layout DLL bundled in this ZIP.

.DESCRIPTION
    Reads layout.json from the same folder, copies the named DLL to
    %SystemRoot%\System32 (and SysWOW64 when present), and writes the
    HKLM keyboard-layouts registry entries. Self-elevates if not admin.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$here = Split-Path -Parent $MyInvocation.MyCommand.Path

# ---------------------------------------------------------------------------
# 1. Read sidecar layout.json
# ---------------------------------------------------------------------------
$metaPath = Join-Path $here 'layout.json'
if (-not (Test-Path -LiteralPath $metaPath)) {
    Write-Error "layout.json not found next to this script. Re-extract the ZIP and try again."
    exit 1
}
$meta = Get-Content -LiteralPath $metaPath -Raw | ConvertFrom-Json
$dllName     = [string]$meta.dllName
$displayName = [string]$meta.displayName
$localeId    = [string]$meta.localeId

if (-not $dllName)     { Write-Error 'layout.json: dllName missing';     exit 1 }
if (-not $displayName) { Write-Error 'layout.json: displayName missing'; exit 1 }
if ($localeId -notmatch '^[0-9a-fA-F]{8}$') {
    Write-Error "layout.json: localeId must be 8 hex chars, got '$localeId'"
    exit 1
}

$dllPath = Join-Path $here ("$dllName.dll")
if (-not (Test-Path -LiteralPath $dllPath)) {
    Write-Error "DLL not found: $dllPath"
    exit 1
}

# ---------------------------------------------------------------------------
# 2. Self-elevate if not admin
# ---------------------------------------------------------------------------
$principal = New-Object Security.Principal.WindowsPrincipal(
    [Security.Principal.WindowsIdentity]::GetCurrent()
)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Host 'Requesting administrator privileges...' -ForegroundColor Yellow
    Start-Process -FilePath 'powershell.exe' -ArgumentList @(
        '-ExecutionPolicy', 'Bypass',
        '-NoProfile',
        '-File', $MyInvocation.MyCommand.Path
    ) -Verb RunAs
    exit 0
}

try {
    # ----------------------------------------------------------------------
    # 3. Pick a free registry key (a001<suffix>, a002<suffix>, ...)
    # ----------------------------------------------------------------------
    $suffix = $localeId.Substring(4, 4)
    $kbRoot = 'HKLM:\SYSTEM\CurrentControlSet\Control\Keyboard Layouts'

    $prefix = 1
    do {
        $regKey  = 'a{0:x3}{1}' -f $prefix, $suffix
        $regPath = Join-Path $kbRoot $regKey
        $prefix++
    } while (Test-Path -LiteralPath $regPath)

    $existingIds = @()
    Get-ChildItem -Path $kbRoot -ErrorAction SilentlyContinue | ForEach-Object {
        $val = (Get-ItemProperty -LiteralPath $_.PSPath -Name 'Layout Id' -ErrorAction SilentlyContinue).'Layout Id'
        if ($val) { $existingIds += $val }
    }

    $layoutNumber = 1
    do {
        $layoutIdHex = '{0:x4}' -f $layoutNumber
        $layoutNumber++
    } while ($existingIds -contains $layoutIdHex)

    Write-Host "Registry key : $regKey"
    Write-Host "Layout Id    : $layoutIdHex"

    # ----------------------------------------------------------------------
    # 4. Copy DLL to System32 (and SysWOW64 if present)
    # ----------------------------------------------------------------------
    $dllFile  = "$dllName.dll"
    $sys32    = Join-Path $env:SystemRoot 'System32'
    $destSys  = Join-Path $sys32 $dllFile

    Write-Host "Copying $dllFile to $sys32 ..."
    Copy-Item -LiteralPath $dllPath -Destination $destSys -Force

    $sysWow = Join-Path $env:SystemRoot 'SysWOW64'
    if (Test-Path -LiteralPath $sysWow) {
        $destWow = Join-Path $sysWow $dllFile
        Write-Host "Copying $dllFile to $sysWow ..."
        Copy-Item -LiteralPath $dllPath -Destination $destWow -Force
    }

    # ----------------------------------------------------------------------
    # 5. Registry entries
    # ----------------------------------------------------------------------
    Write-Host "Writing registry entries at $regPath ..."
    New-Item -Path $regPath -Force | Out-Null
    New-ItemProperty -LiteralPath $regPath -Name 'Layout File' -Value $dllFile     -PropertyType String -Force | Out-Null
    New-ItemProperty -LiteralPath $regPath -Name 'Layout Text' -Value $displayName -PropertyType String -Force | Out-Null
    New-ItemProperty -LiteralPath $regPath -Name 'Layout Id'   -Value $layoutIdHex -PropertyType String -Force | Out-Null

    Write-Host ''
    Write-Host '========================================' -ForegroundColor Green
    Write-Host ' Keyboard layout installed successfully!' -ForegroundColor Green
    Write-Host '========================================' -ForegroundColor Green
    Write-Host ''
    Write-Host "Layout       : $displayName"
    Write-Host "Registry key : $regKey"
    Write-Host ''
    Write-Host 'Restart your PC, then add the layout in:'
    Write-Host '  Settings > Time & Language > Language & region'
    Write-Host '  -> click "..." next to your language'
    Write-Host '  -> Language options -> Add a keyboard'
    Write-Host ''
}
catch {
    Write-Error "Installation failed: $_"
    exit 1
}
