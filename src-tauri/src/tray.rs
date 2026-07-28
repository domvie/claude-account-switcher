use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Runtime};

use crate::profiles::Profile;

const TRAY_ID: &str = "main-tray";

fn build_menu<R: Runtime>(app: &AppHandle<R>, profiles: &[Profile]) -> tauri::Result<Menu<R>> {
    let menu = Menu::new(app)?;
    for p in profiles.iter().filter(|p| p.desktop.enabled) {
        let item = MenuItem::with_id(app, format!("launch:{}", p.id), &p.name, true, None::<&str>)?;
        menu.append(&item)?;
    }
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(
        app,
        "open-manager",
        "Open Manager…",
        true,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?)?;
    Ok(menu)
}

pub fn setup(app: &AppHandle, profiles: &[Profile]) -> tauri::Result<()> {
    let menu = build_menu(app, profiles)?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            if id == "quit" {
                app.exit(0);
            } else if id == "open-manager" {
                show_main_window(app);
            } else if let Some(profile_id) = id.strip_prefix("launch:") {
                let state = app.state::<crate::commands::AppState>();
                let result = {
                    let store = state.store.lock().unwrap();
                    store
                        .get(profile_id)
                        .map(|p| crate::launcher::launch_desktop(p, &store.config.settings))
                };
                if let Err(e) | Ok(Err(e)) = result {
                    log::error!("tray launch failed: {e}");
                }
            }
        })
        .build(app)?;
    Ok(())
}

/// Rebuild the tray menu after any profile mutation.
pub fn rebuild(app: &AppHandle, profiles: &[Profile]) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        match build_menu(app, profiles) {
            Ok(menu) => {
                let _ = tray.set_menu(Some(menu));
            }
            Err(e) => log::error!("tray menu rebuild failed: {e}"),
        }
    }
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
