use std::path::Path;

use anyhow::anyhow;
#[cfg(not(target_os = "windows"))]
use anyhow::Context;

#[cfg(target_os = "windows")]
pub fn open_with_default_app(path: &Path) -> anyhow::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;

    let op: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();
    let file: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();

    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(op.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOW,
        )
    };
    let code = result.0 as isize;
    if code <= 32 {
        return Err(anyhow!("ShellExecuteW failed with code {}", code));
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn open_with_default_app(path: &Path) -> anyhow::Result<()> {
    let status = std::process::Command::new("xdg-open")
        .arg(path)
        .status()
        .context("failed to launch xdg-open")?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("xdg-open exited with status {:?}", status.code()))
    }
}
