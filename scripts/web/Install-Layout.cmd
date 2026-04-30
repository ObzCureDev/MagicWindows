@echo off
title Install MagicWindows layout
powershell -ExecutionPolicy Bypass -NoProfile -File "%~dp0Install-Layout.ps1"
echo.
echo Press any key to close this window.
pause >nul
