#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Uninstalls an Apple Magic Keyboard layout from Windows.

.DESCRIPTION
    Removes the keyboard layout registry entries and deletes the DLL files
    from the system directories.

.PARAMETER DllName
    DLL filename without the .dll extension (e.g. "kbdaplfr").

.PARAMETER LayoutId
    The registry layout ID to remove (e.g. "a001040c").

.EXAMPLE
    .\Uninstall-Layout.ps1 -DllName "kbdaplfr" -LayoutId "a001040c"
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$DllName,

    [Parameter(Mandatory = $true)]
    [ValidatePattern('^[0-9a-fA-F]{8}$')]
    [string]$LayoutId
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

try {
    # ------------------------------------------------------------------
    # 2. Remove registry entries
    # ------------------------------------------------------------------
    $regPath = "HKLM:\SYSTEM\CurrentControlSet\Control\Keyboard Layouts\$LayoutId"

    if (Test-Path -LiteralPath $regPath) {
        Write-Host "Removing registry key: $regPath ..."
        Remove-Item -LiteralPath $regPath -Recurse -Force
        Write-Host 'Registry entries removed.'
    }
    else {
        Write-Warning "Registry key not found: $regPath (skipping)"
    }

    # ------------------------------------------------------------------
    # 3. Remove DLL files from system directories
    # ------------------------------------------------------------------
    $dllFileName = "$DllName.dll"

    $system32Dll = Join-Path $env:SystemRoot "System32\$dllFileName"
    if (Test-Path -LiteralPath $system32Dll) {
        Write-Host "Removing $system32Dll ..."
        Remove-Item -LiteralPath $system32Dll -Force
        Write-Host 'Removed from System32.'
    }
    else {
        Write-Warning "DLL not found in System32 (skipping): $system32Dll"
    }

    $sysWow64Dll = Join-Path $env:SystemRoot "SysWOW64\$dllFileName"
    if (Test-Path -LiteralPath $sysWow64Dll) {
        Write-Host "Removing $sysWow64Dll ..."
        Remove-Item -LiteralPath $sysWow64Dll -Force
        Write-Host 'Removed from SysWOW64.'
    }
    else {
        Write-Warning "DLL not found in SysWOW64 (skipping): $sysWow64Dll"
    }

    # ------------------------------------------------------------------
    # 4. Success
    # ------------------------------------------------------------------
    Write-Host ''
    Write-Host '==========================================' -ForegroundColor Green
    Write-Host ' Keyboard layout uninstalled successfully!' -ForegroundColor Green
    Write-Host '==========================================' -ForegroundColor Green
    Write-Host ''
    Write-Host "Layout ID : $LayoutId"
    Write-Host "DLL       : $dllFileName"
    Write-Host ''
    Write-Host 'Note: If you had the layout active, you may need to sign out and'
    Write-Host 'sign back in (or restart) for the change to take full effect.'
    Write-Host ''
}
catch {
    Write-Error "Uninstallation failed: $_"
    exit 1
}
