//! Process-spawning helpers that keep console windows hidden.
//!
//! Every PowerShell/cmd invocation in this app is a GUI process spawning a
//! console child. Without `CREATE_NO_WINDOW`, Windows allocates a *visible*
//! console window for each spawn, which flashes on screen — especially painful
//! on the Bluetooth pairing screen, which polls on a timer (a window every
//! couple of seconds). These helpers set the flag so spawned consoles stay
//! hidden.
//!
//! Note: the *elevated* install windows are created by `Start-Process` inside
//! the PowerShell scripts, not by these Rust spawns. Hiding the Rust launchers
//! here therefore never affects the UAC prompt or the elevated child window.

use std::process::Command;

/// `CREATE_NO_WINDOW` — suppress the console window of a spawned process.
/// See <https://learn.microsoft.com/windows/win32/procthread/process-creation-flags>.
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Build a [`Command`] for `program` that won't pop a console window on Windows.
/// On other platforms this is a plain `Command::new`.
pub fn hidden_command(program: &str) -> Command {
    #[allow(unused_mut)]
    let mut cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

/// Shorthand for a hidden `powershell` invocation — the app's most common spawn.
pub fn powershell() -> Command {
    hidden_command("powershell")
}
