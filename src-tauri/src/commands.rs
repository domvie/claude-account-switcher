use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::error::{AppError, Result};
use crate::launcher::{self, LaunchOutcome};
use crate::platform;
use crate::profiles::{Env, NewProfile, Profile, ProfileUpdate, Settings, Store};

pub struct AppState {
    pub store: Mutex<Store>,
    pub env: Env,
}

fn notify_profiles_changed(app: &AppHandle, state: &AppState) {
    let profiles = state.store.lock().unwrap().config.profiles.clone();
    let _ = app.emit("profiles-changed", &profiles);
    crate::tray::rebuild(app, &profiles);
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvInfo {
    pub platform: &'static str,
    pub desktop_app_path: Option<PathBuf>,
    pub claude_code_on_path: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteOptions {
    pub delete_desktop_dir: bool,
    pub delete_code_dir: bool,
    pub delete_keychain_entry: bool,
    pub remove_shim: bool,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteReport {
    pub desktop_dir_deleted: bool,
    pub code_dir_deleted: bool,
    pub keychain_entry_deleted: bool,
    pub shim_removed: bool,
    pub warnings: Vec<String>,
}

#[tauri::command]
pub fn list_profiles(state: State<AppState>) -> Vec<Profile> {
    state.store.lock().unwrap().config.profiles.clone()
}

#[tauri::command]
pub fn create_profile(
    app: AppHandle,
    state: State<AppState>,
    input: NewProfile,
) -> Result<Profile> {
    let profile = state.store.lock().unwrap().create(input, &state.env)?;
    notify_profiles_changed(&app, &state);
    Ok(profile)
}

#[tauri::command]
pub fn update_profile(
    app: AppHandle,
    state: State<AppState>,
    id: String,
    patch: ProfileUpdate,
) -> Result<Profile> {
    let profile = state.store.lock().unwrap().update(&id, patch)?;
    notify_profiles_changed(&app, &state);
    Ok(profile)
}

#[tauri::command]
pub fn delete_profile(
    app: AppHandle,
    state: State<AppState>,
    id: String,
    opts: DeleteOptions,
) -> Result<DeleteReport> {
    let profile = state.store.lock().unwrap().remove(&id)?;
    let mut report = DeleteReport::default();

    if opts.delete_desktop_dir && profile.desktop.user_data_dir.exists() {
        match trash::delete(&profile.desktop.user_data_dir) {
            Ok(()) => report.desktop_dir_deleted = true,
            Err(e) => report
                .warnings
                .push(format!("could not trash desktop data dir: {e}")),
        }
    }
    if opts.delete_code_dir && profile.code.config_dir.exists() {
        match trash::delete(&profile.code.config_dir) {
            Ok(()) => report.code_dir_deleted = true,
            Err(e) => report
                .warnings
                .push(format!("could not trash Claude Code config dir: {e}")),
        }
    }
    if opts.delete_keychain_entry {
        match platform::delete_keychain_credential(&profile.code.config_dir) {
            Ok(deleted) => report.keychain_entry_deleted = deleted,
            Err(e) => report
                .warnings
                .push(format!("could not delete keychain entry: {e}")),
        }
    }
    if opts.remove_shim {
        match crate::shims::remove(&profile, &state.env) {
            Ok(removed) => report.shim_removed = removed,
            Err(e) => report.warnings.push(format!("could not remove shim: {e}")),
        }
    }

    notify_profiles_changed(&app, &state);
    Ok(report)
}

#[tauri::command]
pub fn launch_desktop(app: AppHandle, state: State<AppState>, id: String) -> Result<LaunchOutcome> {
    let (profile, settings) = {
        let store = state.store.lock().unwrap();
        (store.get(&id)?.clone(), store.config.settings.clone())
    };
    let result = launcher::launch_desktop(&profile, &settings)?;
    let _ = state.store.lock().unwrap().touch_launched(&id);
    notify_profiles_changed(&app, &state);
    Ok(result)
}

#[tauri::command]
pub fn launch_terminal(state: State<AppState>, id: String, terminal: Option<String>) -> Result<()> {
    let (profile, settings) = {
        let store = state.store.lock().unwrap();
        (store.get(&id)?.clone(), store.config.settings.clone())
    };
    launcher::launch_terminal(&profile, &settings, terminal.as_deref())
}

#[tauri::command]
pub fn terminal_command(state: State<AppState>, id: String) -> Result<String> {
    let profile = state.store.lock().unwrap().get(&id)?.clone();
    let shim_on_path = crate::shims::status(&profile, &state.env)
        .map(|s| s.installed && s.on_path)
        .unwrap_or(false);
    Ok(launcher::terminal_command(&profile, shim_on_path))
}

#[tauri::command]
pub fn detect_environment(state: State<AppState>) -> EnvInfo {
    let settings = state.store.lock().unwrap().config.settings.clone();
    EnvInfo {
        platform: std::env::consts::OS,
        desktop_app_path: launcher::resolve_desktop_app(&settings).ok(),
        claude_code_on_path: which_claude(),
    }
}

fn which_claude() -> bool {
    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|p| {
                let candidate = p.join("claude");
                candidate.is_file()
            })
        })
        .unwrap_or(false)
}

#[tauri::command]
pub fn install_shim(
    app: AppHandle,
    state: State<AppState>,
    id: String,
) -> Result<crate::shims::ShimStatus> {
    let profile = state.store.lock().unwrap().get(&id)?.clone();
    std::fs::create_dir_all(&profile.code.config_dir)?;
    let status = crate::shims::install(&profile, &state.env)?;
    set_shim_installed(&state, &id, true)?;
    notify_profiles_changed(&app, &state);
    Ok(status)
}

#[tauri::command]
pub fn remove_shim(app: AppHandle, state: State<AppState>, id: String) -> Result<bool> {
    let profile = state.store.lock().unwrap().get(&id)?.clone();
    let removed = crate::shims::remove(&profile, &state.env)?;
    set_shim_installed(&state, &id, false)?;
    notify_profiles_changed(&app, &state);
    Ok(removed)
}

fn set_shim_installed(state: &AppState, id: &str, installed: bool) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    for p in store.config.profiles.iter_mut() {
        if p.id == id {
            p.code.shim_installed = installed;
        }
    }
    store.save()
}

#[tauri::command]
pub fn shim_status(state: State<AppState>, id: String) -> Result<crate::shims::ShimStatus> {
    let profile = state.store.lock().unwrap().get(&id)?.clone();
    crate::shims::status(&profile, &state.env)
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Settings {
    state.store.lock().unwrap().config.settings.clone()
}

#[tauri::command]
pub fn set_settings(state: State<AppState>, settings: Settings) -> Result<()> {
    let mut store = state.store.lock().unwrap();
    store.config.settings = settings;
    store.save()
}

#[tauri::command]
pub fn reveal_path(path: PathBuf) -> Result<()> {
    if !path.exists() {
        return Err(AppError::not_found(format!(
            "{} does not exist yet",
            path.display()
        )));
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&path).status()?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = &path;
    }
    Ok(())
}
