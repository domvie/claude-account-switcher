use std::path::PathBuf;

use serde::Serialize;

use crate::error::{AppError, Result};
use crate::platform;
use crate::profiles::{Profile, Settings};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LaunchResult {
    Launched,
    FocusedExisting,
}

pub fn resolve_desktop_app(settings: &Settings) -> Result<PathBuf> {
    if let Some(path) = &settings.claude_desktop_path {
        if path.exists() {
            return Ok(path.clone());
        }
    }
    platform::find_claude_desktop().ok_or_else(|| {
        AppError::not_found("Claude Desktop was not found — set its location in Settings")
    })
}

pub fn launch_desktop(profile: &Profile, settings: &Settings) -> Result<LaunchResult> {
    if !profile.desktop.enabled {
        return Err(AppError::validation(format!(
            "profile '{}' has Claude Desktop disabled",
            profile.name
        )));
    }
    let app_path = resolve_desktop_app(settings)?;
    std::fs::create_dir_all(&profile.desktop.user_data_dir)?;

    if !profile.is_default {
        if let Some(pid) = platform::find_running_desktop(&profile.desktop.user_data_dir) {
            platform::focus_process(pid)?;
            return Ok(LaunchResult::FocusedExisting);
        }
    }
    platform::launch_desktop(
        &app_path,
        &profile.desktop.user_data_dir,
        profile.is_default,
    )?;
    Ok(LaunchResult::Launched)
}

pub fn launch_terminal(profile: &Profile, settings: &Settings) -> Result<()> {
    if !profile.code.enabled {
        return Err(AppError::validation(format!(
            "profile '{}' has Claude Code disabled",
            profile.name
        )));
    }
    std::fs::create_dir_all(&profile.code.config_dir)?;
    platform::launch_terminal(&settings.terminal, &profile.code.config_dir)
}
