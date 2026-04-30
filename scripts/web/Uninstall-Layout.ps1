<#
.SYNOPSIS
    Uninstalls an Apple Magic Keyboard layout previously installed from this ZIP.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$here = Split-Path -Parent $MyInvocation.MyCommand.Path

$metaPath = Join-Path $here 'layout.json'
if (-not (Test-Path -LiteralPath $metaPath)) {
    Write-Error "layout.json not found next to this script."
    exit 1
}
$meta = Get-Content -LiteralPath $metaPath -Raw | ConvertFrom-Json
$dllName  = [string]$meta.dllName
$localeId = [string]$meta.localeId
if ($localeId -notmatch '^[0-9a-fA-F]{8}$') {
    Write-Error "layout.json: localeId must be 8 hex chars."
    exit 1
}

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
    $suffix  = $localeId.Substring(4, 4)
    $kbRoot  = 'HKLM:\SYSTEM\CurrentControlSet\Control\Keyboard Layouts'
    $dllFile = "$dllName.dll"

    # Find every key whose Layout File matches our DLL
    $keysToRemove = @()
    Get-ChildItem -Path $kbRoot -ErrorAction SilentlyContinue | ForEach-Object {
        $val = (Get-ItemProperty -LiteralPath $_.PSPath -Name 'Layout File' -ErrorAction SilentlyContinue).'Layout File'
        if ($val -and $val.ToLower() -eq $dllFile.ToLower()) {
            $keysToRemove += $_.PSChildName
        }
    }

    if ($keysToRemove.Count -eq 0) {
        Write-Host "No registry entry references $dllFile. Nothing to remove."
    } else {
        foreach ($keyName in $keysToRemove) {
            $path = Join-Path $kbRoot $keyName
            Write-Host "Removing $path ..."
            Remove-Item -LiteralPath $path -Recurse -Force
        }
    }

    foreach ($subDir in @('System32', 'SysWOW64')) {
        $full = Join-Path (Join-Path $env:SystemRoot $subDir) $dllFile
        if (Test-Path -LiteralPath $full) {
            Write-Host "Removing $full ..."
            Remove-Item -LiteralPath $full -Force
        }
    }

    Write-Host ''
    Write-Host '========================================' -ForegroundColor Green
    Write-Host ' Layout uninstalled. Please restart Windows.' -ForegroundColor Green
    Write-Host '========================================' -ForegroundColor Green
}
catch {
    Write-Error "Uninstall failed: $_"
    exit 1
}
