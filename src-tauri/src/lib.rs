mod commands;
mod fallback;
mod models;
mod native_bridge;
mod settings;
mod state;

use commands::{
    apply_panel_window, apply_panel_window_core, clear_notifications, dismiss_notification,
    get_audio_sessions, get_notifications, get_runtime_capabilities, get_settings,
    get_system_snapshot, hide_panel, persist_panel_position, push_demo_notification, quit_app,
    save_settings, set_audio_mute, set_audio_volume, set_panel_click_through, show_panel,
    toggle_panel,
};
use fallback::FallbackCollector;
use models::{NotificationItem, PanelId};
use native_bridge::NativeEngine;
use state::AppState;
use std::{
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use uuid::Uuid;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            let app_handle = app.handle().clone();
            let settings = settings::load(&app_handle);
            let native = NativeEngine::load(&app_handle).ok();
            let notifications = initial_notifications();

            app.manage(AppState {
                settings: Mutex::new(settings.clone()),
                native: Mutex::new(native),
                fallback: Mutex::new(FallbackCollector::new()),
                notifications: Mutex::new(notifications),
            });

            for panel_id in [
                PanelId::Performance,
                PanelId::Mixer,
                PanelId::Notifications,
            ] {
                let panel = settings.panels.get(panel_id);
                let _ = apply_panel_window_core(&app_handle, panel_id, &settings);
                if panel.enabled && panel.show_on_launch {
                    if let Some(window) = app.get_webview_window(panel_id.label()) {
                        let _ = window.show();
                    }
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { .. } = event {
                    window.app_handle().exit(0);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            get_system_snapshot,
            get_audio_sessions,
            set_audio_volume,
            set_audio_mute,
            toggle_panel,
            show_panel,
            hide_panel,
            set_panel_click_through,
            apply_panel_window,
            persist_panel_position,
            get_notifications,
            push_demo_notification,
            dismiss_notification,
            clear_notifications,
            get_runtime_capabilities,
            quit_app
        ])
        .run(tauri::generate_context!())
        .expect("Błąd podczas uruchamiania Moustache Control Center");
}

fn initial_notifications() -> Vec<NotificationItem> {
    vec![
        NotificationItem {
            id: Uuid::new_v4().to_string(),
            app: "Moustache".to_string(),
            title: "Centrum sterowania jest gotowe".to_string(),
            body: "Alt+1 otwiera wydajność, Alt+2 mikser, a Alt+3 powiadomienia.".to_string(),
            timestamp: now_millis(),
            accent: Some("#8b5cf6".to_string()),
            icon_text: Some("M".to_string()),
            read: false,
        },
    ]
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}
