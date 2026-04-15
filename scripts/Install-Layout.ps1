#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Installs an Apple Magic Keyboard layout on Windows.

.DESCRIPTION
    Copies the compiled keyboard layout DLL to the system directories and
    creates the required registry entries so Windows can discover the layout.

.PARAMETER DllPath
    Full path to the compiled keyboard layout DLL file.

.PARAMETER LayoutName
    Display name shown in Windows Settings (e.g. "French - Apple Magic Keyboard (AZERTY)").

.PARAMETER DllName
    DLL filename without the .dll extension (e.g. "kbdaplfr").

.PARAMETER LocaleId
    Windows locale identifier as an 8-character hex string (e.g. "0000040c").

.EXAMPLE
    .\Install-Layout.ps1 -DllPath "C:\build\kbdaplfr.dll" `
                         -LayoutName "French - Apple Magic Keyboard (AZERTY)" `
                         -DllName "kbdaplfr" `
                         -LocaleId "0000040c"
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$DllPath,

    [Parameter(Mandatory = $true)]
    [string]$LayoutName,

    [Parameter(Mandatory = $true)]
    [string]$DllName,

    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[0-9a-fA-F]{8}$')]
    [string]$LocaleId
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ---------------------------------------------------------------------------
# 1. Verify administrator privileges
# ---------------------------------------------------------------------------
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal(
    [Security.Principal.WindowsIdentity]::GetCurrent()
)
if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Error "This script must be run as Administrator. Right-click PowerShell and select 'Run as administrator'."
    exit 1
}

# ---------------------------------------------------------------------------
# 2. Validate the DLL exists
# ---------------------------------------------------------------------------
if (-not (Test-Path -LiteralPath $DllPath)) {
    Write-Error "DLL not found at path: $DllPath"
    exit 1
}

try {
    # ------------------------------------------------------------------
    # 3. Generate a unique layout ID
    #    Format: "a" + 3-digit prefix + last 4 hex chars of the locale ID
    #    e.g. LocaleId "0000040c" -> suffix "040c" -> "a001040c"
    # ------------------------------------------------------------------
    $suffix = $LocaleId.Substring(4, 4)
    $kbLayoutsRoot = 'HKLM:\SYSTEM\CurrentControlSet\Control\Keyboard Layouts'

    # Find the next available prefix starting from a001
    $prefix = 1
    do {
        $layoutId = 'a{0:x3}{1}' -f $prefix, $suffix
        $regPath  = Join-Path $kbLayoutsRoot $layoutId
        $prefix++
    } while (Test-Path -LiteralPath $regPath)

    # Derive a unique 4-digit hex Layout Id value for the registry
    # Scan existing Layout Id values to avoid collisions
    $existingIds = @()
    Get-ChildItem -Path $kbLayoutsRoot -ErrorAction SilentlyContinue | ForEach-Object {
        $val = (Get-ItemProperty -LiteralPath $_.PSPath -Name 'Layout Id' -ErrorAction SilentlyContinue).'Layout Id'
        if ($val) { $existingIds += $val }
    }

    $layoutNumber = 1
    do {
        $layoutIdHex = '{0:x4}' -f $layoutNumber
        $layoutNumber++
    } while ($existingIds -contains $layoutIdHex)

    Write-Host "Using registry key : $layoutId"
    Write-Host "Using Layout Id    : $layoutIdHex"

    # ------------------------------------------------------------------
    # 4. Copy DLL to system directories
    # ------------------------------------------------------------------
    $dllFileName = "$DllName.dll"
    $system32Path = Join-Path $env:SystemRoot 'System32'
    $destSystem32 = Join-Path $system32Path $dllFileName

    Write-Host "Copying DLL to $destSystem32 ..."
    Copy-Item -LiteralPath $DllPath -Destination $destSystem32 -Force

    # On 64-bit Windows, also copy to SysWOW64 for 32-bit applications
    $sysWow64Path = Join-Path $env:SystemRoot 'SysWOW64'
    if (Test-Path -LiteralPath $sysWow64Path) {
        $destSysWow64 = Join-Path $sysWow64Path $dllFileName
        Write-Host "Copying DLL to $destSysWow64 ..."
        Copy-Item -LiteralPath $DllPath -Destination $destSysWow64 -Force
    }

    # ------------------------------------------------------------------
    # 5. Create registry entries
    # ------------------------------------------------------------------
    Write-Host "Creating registry entries at $regPath ..."
    New-Item -Path $regPath -Force | Out-Null
    New-ItemProperty -LiteralPath $regPath -Name 'Layout File' -Value $dllFileName      -PropertyType String -Force | Out-Null
    New-ItemProperty -LiteralPath $regPath -Name 'Layout Text' -Value $LayoutName        -PropertyType String -Force | Out-Null
    New-ItemProperty -LiteralPath $regPath -Name 'Layout Id'   -Value $layoutIdHex       -PropertyType String -Force | Out-Null

    # ------------------------------------------------------------------
    # 6. Success
    # ------------------------------------------------------------------
    Write-Host ''
    Write-Host '========================================' -ForegroundColor Green
    Write-Host ' Keyboard layout installed successfully!' -ForegroundColor Green
    Write-Host '========================================' -ForegroundColor Green
    Write-Host ''
    Write-Host "Layout Name : $LayoutName"
    Write-Host "Layout ID   : $layoutId"
    Write-Host "DLL         : $dllFileName"
    Write-Host ''
    Write-Host 'To start using the layout:'
    Write-Host '  1. Open Windows Settings > Time & Language > Language & region'
    Write-Host '  2. Click the "..." next to your language and select "Language options"'
    Write-Host '  3. Click "Add a keyboard" and select the new layout from the list'
    Write-Host ''
}
catch {
    Write-Error "Installation failed: $_"
    exit 1
}
