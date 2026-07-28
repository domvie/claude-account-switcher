use std::path::PathBuf;

use serde::Serialize;

use crate::error::{AppError, Result};
use crate::platform;
use crate::profiles::{Profile, Settings};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LaunchKind {
    Launched,
    FocusedExisting,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchOutcome {
    pub kind: LaunchKind,
    /// True when this profile has no login session yet — the frontend shows
    /// a heads-up about the email-verification deep link landing in the
    /// default Claude instance.
    pub first_login: bool,
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

pub fn launch_desktop(profile: &Profile, settings: &Settings) -> Result<LaunchOutcome> {
    if !profile.desktop.enabled {
        return Err(AppError::validation(format!(
            "profile '{}' has Claude Desktop disabled",
            profile.name
        )));
    }
    let app_path = resolve_desktop_app(settings)?;
    // A session cookie store only exists after a successful login.
    let first_login =
        !profile.is_default && !profile.desktop.user_data_dir.join("Cookies").exists();
    std::fs::create_dir_all(&profile.desktop.user_data_dir)?;

    if !profile.is_default {
        if let Some(pid) = platform::find_running_desktop(&profile.desktop.user_data_dir) {
            platform::focus_process(pid)?;
            return Ok(LaunchOutcome {
                kind: LaunchKind::FocusedExisting,
                first_login,
            });
        }
    }
    platform::launch_desktop(
        &app_path,
        &profile.desktop.user_data_dir,
        profile.is_default,
    )?;
    Ok(LaunchOutcome {
        kind: LaunchKind::Launched,
        first_login,
    })
}

pub fn launch_terminal(
    profile: &Profile,
    settings: &Settings,
    terminal: Option<&str>,
) -> Result<()> {
    if !profile.code.enabled {
        return Err(AppError::validation(format!(
            "profile '{}' has Claude Code disabled",
            profile.name
        )));
    }
    std::fs::create_dir_all(&profile.code.config_dir)?;
    let terminal = terminal.unwrap_or(&settings.terminal);
    platform::launch_terminal(terminal, &profile.code.config_dir)
}

/// The command a user pastes into their own terminal: the shim when it's
/// installed and on PATH, otherwise the explicit env-var form.
pub fn terminal_command(profile: &Profile, shim_on_path: bool) -> String {
    if profile.is_default {
        return "claude".into();
    }
    if profile.code.shim_installed && shim_on_path {
        return profile.code.shim_name.clone();
    }
    format!(
        "CLAUDE_CONFIG_DIR=\"{}\" claude",
        profile.code.config_dir.display()
    )
}
