use std::process::Command;
#[cfg(feature = "gui")]
pub fn attach_parent_console_for_cli() {
    if std::env::args_os().nth(1).is_none() {
        return;
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn AttachConsole(process_id: u32) -> i32;
    }
    // ATTACH_PARENT_PROCESS: reuse an existing terminal; never create a new console.
    // This call has no pointer arguments. Failure just means no parent console exists.
    unsafe {
        AttachConsole(u32::MAX);
    }
}

/// Prevent tool and game subprocesses from opening consoles during a GUI session.
/// stdout/stderr pipes remain available for the launcher's journal.
pub fn hide_console_for_gui(command: &mut Command) {
    if cfg!(feature = "gui") && std::env::args_os().nth(1).is_none() {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
}
