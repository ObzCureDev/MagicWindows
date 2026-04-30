MagicWindows — Apple keyboard layout for Windows
=================================================

INSTALL
-------
1. Right-click "Install-Layout.cmd" and pick "Run as administrator".
   Or just double-click it and accept the UAC prompt that appears.

   Windows SmartScreen may show "Windows protected your PC". Click
   "More info" -> "Run anyway". The script and DLL are open source;
   the warning is shown for any unsigned installer.

2. Wait for the script to finish. It copies the layout DLL into
   C:\Windows\System32 and registers it with Windows.

3. Restart your PC.

4. Open Settings -> Time & Language -> Language & region.
   Click "..." next to your language -> "Language options" ->
   "Add a keyboard". Pick the new layout from the list.

5. Switch between layouts with Win+Space.


UNINSTALL
---------
Run "Uninstall-Layout.cmd" the same way (Run as administrator).
It removes the DLL and the registry entries. Reboot to fully clear it.


WANT MORE?
----------
The desktop app adds: auto-detect, modifier toggles (Cmd<->Ctrl),
F12/Eject remap, post-install health check, one-click uninstall UI.

  https://magicwindows.mindvisionstudio.com/#/desktop


REPORT A BUG
------------
  https://github.com/ObzCureDev/MagicWindows/issues
