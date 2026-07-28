mod commands;
mod error;
mod launcher;
mod platform;
mod profiles;
mod shims;
mod tray;

use std::sync::Mutex;

use tauri::Manager;

use commands::AppState;
use profiles::{Env, Store};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            let env = Env::detect()?;
            let config_path = app
                .path()
                .app_config_dir()
                .expect("app config dir unavailable")
                .join("config.json");
            let store = Store::load(config_path, &env)?;
            let profiles = store.config.profiles.clone();
            app.manage(AppState {
                store: Mutex::new(store),
                env,
            });
            tray::setup(app.handle(), &profiles)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // Close-to-tray: hide instead of exiting so the tray stays alive.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let hide = {
                    let state = window.state::<AppState>();
                    let store = state.store.lock().unwrap();
                    store.config.settings.hide_window_on_close
                };
                if hide {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_profiles,
            commands::create_profile,
            commands::update_profile,
            commands::delete_profile,
            commands::launch_desktop,
            commands::launch_terminal,
            commands::terminal_command,
            commands::install_shim,
            commands::remove_shim,
            commands::shim_status,
            commands::detect_environment,
            commands::get_settings,
            commands::set_settings,
            commands::reveal_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
