// Linux support lands in M5 (beta; see plan). Stubs keep the build green.
use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};

pub fn find_claude_desktop() -> Option<PathBuf> {
    None
}

pub fn launch_desktop(_app_path: &Path, _user_data_dir: &Path, _is_default: bool) -> Result<()> {
    Err(AppError::launch("Linux support is not implemented yet"))
}

pub fn find_running_desktop(_user_data_dir: &Path) -> Option<u32> {
    None
}

pub fn focus_process(_pid: u32) -> Result<()> {
    Err(AppError::launch("Linux support is not implemented yet"))
}

pub fn launch_terminal(
    _terminal: &str,
    _env_var: &str,
    _config_dir: &Path,
    _binary: &str,
) -> Result<()> {
    Err(AppError::launch("Linux support is not implemented yet"))
}

pub fn default_shim_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".local/bin"))
}

pub fn delete_keychain_credential(_config_dir: &Path) -> Result<bool> {
    // Linux stores credentials in .credentials.json inside the config dir;
    // deleting the dir covers it.
    Ok(false)
}
