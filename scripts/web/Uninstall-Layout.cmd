@echo off
title Uninstall MagicWindows layout
powershell -ExecutionPolicy Bypass -NoProfile -File "%~dp0Uninstall-Layout.ps1"
echo.
echo Press any key to close this window.
pause >nul
