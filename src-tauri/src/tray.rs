use anyhow::Result;
use tauri::image::Image;
use tauri::menu::{IconMenuItemBuilder, Menu};
use tauri::tray::TrayIconBuilder;
use tauri::{App, Manager};

use crate::app_state::AppState;
use crate::pipeline::logger::EventLogger;

// Embed menu icons (16x16 PNGs)
const ICON_PLAY: &[u8] = include_bytes!("../icons/menu/play.png");
const ICON_PAUSE: &[u8] = include_bytes!("../icons/menu/pause.png");
const ICON_RELOAD: &[u8] = include_bytes!("../icons/menu/reload.png");
const ICON_FOLDER: &[u8] = include_bytes!("../icons/menu/folder.png");
const ICON_QUIT: &[u8] = include_bytes!("../icons/menu/quit.png");

pub fn setup_tray(app: &mut App) -> Result<()> {
    // Load icons
    let icon_play = Image::from_bytes(ICON_PLAY)?;
    let icon_pause = Image::from_bytes(ICON_PAUSE)?;
    let icon_reload = Image::from_bytes(ICON_RELOAD)?;
    let icon_folder = Image::from_bytes(ICON_FOLDER)?;
    let icon_quit = Image::from_bytes(ICON_QUIT)?;

    let enable = IconMenuItemBuilder::with_id("enable_monitoring", "Enable monitoring")
        .enabled(true)
        .icon(icon_play)
        .build(app)?;
    let disable = IconMenuItemBuilder::with_id("disable_monitoring", "Disable monitoring")
        .enabled(true)
        .icon(icon_pause)
        .build(app)?;
    let reload_rules = IconMenuItemBuilder::with_id("reload_rules", "Reload rules")
        .enabled(true)
        .icon(icon_reload)
        .build(app)?;
    let open_logs = IconMenuItemBuilder::with_id("open_logs", "Open logs folder")
        .enabled(true)
        .icon(icon_folder)
        .build(app)?;
    let quit = IconMenuItemBuilder::with_id("quit", "Quit")
        .enabled(true)
        .icon(icon_quit)
        .build(app)?;

    let menu = Menu::with_items(app, &[&enable, &disable, &reload_rules, &open_logs, &quit])?;

    let mut tray_builder = TrayIconBuilder::new().menu(&menu);

    if let Some(icon) = app.default_window_icon().cloned() {
        tray_builder = tray_builder.icon(icon);
    }

    tray_builder
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "enable_monitoring" => {
                    if let Some(state) = app.try_state::<AppState>() {
                        state.set_monitoring_enabled(true);
                    }
                }
                "disable_monitoring" => {
                    if let Some(state) = app.try_state::<AppState>() {
                        state.set_monitoring_enabled(false);
                    }
                }
                "reload_rules" => {
                    tracing::info!("reload rules requested");
                }
                "open_logs" => {
                    if let Some(state) = app.try_state::<AppState>() {
                        if let Some(log_dir) = state.log_dir() {
                            if let Ok(logger) = EventLogger::new(log_dir, 7) {
                                let _ = logger.open_logs_folder();
                            }
                        }
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
