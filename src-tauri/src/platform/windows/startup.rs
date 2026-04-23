//! Launch-on-login via the HKCU Run key.

use anyhow::Result;

const REG_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "Utterbar";

pub fn set_launch_on_login(enabled: bool) -> Result<()> {
    let exe = std::env::current_exe()?;
    let exe_str = exe.to_string_lossy().to_string();
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(REG_PATH)?;
    if enabled {
        key.set_value(APP_NAME, &exe_str)?;
    } else {
        let _ = key.delete_value(APP_NAME);
    }
    Ok(())
}

/// Reader pair for `set_launch_on_login`. Unused today — settings stores the intent
/// directly and calls the setter — but kept as the canonical "what does the registry
/// actually say" probe for the diagnostics route we expect to add.
#[allow(dead_code)]
pub fn is_launch_on_login_enabled() -> bool {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    hkcu.open_subkey(REG_PATH)
        .and_then(|key| key.get_value::<String, _>(APP_NAME))
        .is_ok()
}
