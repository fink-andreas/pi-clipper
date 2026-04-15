#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_state;
mod config;
mod context;
mod pipeline;
mod rules;
mod tray;

use anyhow::Result;
use tauri::Manager;

fn main() {
    if let Err(err) = run() {
        eprintln!("fatal startup error: {err:#}");
    }
}

fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .ok();

    tauri::Builder::default()
        .setup(|app| {
            let state = app_state::AppState::new()?;
            app.manage(state.clone());

            let handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                let _ = pipeline::start_background_workers(handle, state).await;
            });

            tray::setup_tray(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())?;

    Ok(())
}