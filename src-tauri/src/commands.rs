use crate::{
    models::{
        AppSettings, AudioSession, NotificationItem, PanelId, PositionPreset,
        RuntimeCapabilities, SystemSnapshot,
    },
    settings,
    state::AppState,
};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, State, WebviewWindow,
};
use uuid::Uuid;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    state
        .settings
        .lock()
        .map(|settings| settings.clone())
        .map_err(|_| "Blokada ustawień została uszkodzona".to_string())
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings_value: AppSettings,
) -> Result<(), String> {
    let settings_value = settings::normalize(settings_value);
    settings::save(&app, &settings_value)?;
    {
        let mut current = state
            .settings
            .lock()
            .map_err(|_| "Blokada ustawień została uszkodzona".to_string())?;
        *current = settings_value.clone();
    }
    app.emit("settings-updated", settings_value)
        .map_err(|error| format!("Nie można rozesłać ustawień do paneli: {error}"))
}

#[tauri::command]
pub fn get_system_snapshot(state: State<'_, AppState>) -> Result<SystemSnapshot, String> {
    let native_snapshot = state.native.lock().ok().and_then(|native| {
        native
            .as_ref()
            .and_then(|engine| engine.snapshot().ok())
    });
    if let Some(snapshot) = native_snapshot {
        return Ok(snapshot);
    }

    state
        .fallback
        .lock()
        .map_err(|_| "Blokada kolektora została uszkodzona".to_string())
        .map(|mut collector| collector.sample())
}

#[tauri::command]
pub fn get_audio_sessions(state: State<'_, AppState>) -> Result<Vec<AudioSession>, String> {
    let native = state
        .native
        .lock()
        .map_err(|_| "Blokada native engine została uszkodzona".to_string())?;
    let engine = native
        .as_ref()
        .ok_or_else(|| "Natywna biblioteka audio jest niedostępna".to_string())?;
    engine.audio_sessions()
}

#[tauri::command]
pub fn set_audio_volume(
    state: State<'_, AppState>,
    id: String,
    volume: f32,
) -> Result<(), String> {
    let native = state
        .native
        .lock()
        .map_err(|_| "Blokada native engine została uszkodzona".to_string())?;
    native
        .as_ref()
        .ok_or_else(|| "Natywna biblioteka audio jest niedostępna".to_string())?
        .set_volume(&id, volume)
}

#[tauri::command]
pub fn set_audio_mute(
    state: State<'_, AppState>,
    id: String,
    muted: bool,
) -> Result<(), String> {
    let native = state
        .native
        .lock()
        .map_err(|_| "Blokada native engine została uszkodzona".to_string())?;
    native
        .as_ref()
        .ok_or_else(|| "Natywna biblioteka audio jest niedostępna".to_string())?
        .set_mute(&id, muted)
}

#[tauri::command]
pub fn toggle_panel(app: AppHandle, panel_id: PanelId) -> Result<(), String> {
    let window = panel_window(&app, panel_id)?;
    if window
        .is_visible()
        .map_err(|error| format!("Nie można odczytać widoczności: {error}"))?
    {
        window
            .hide()
            .map_err(|error| format!("Nie można ukryć panelu: {error}"))
    } else {
        window
            .show()
            .and_then(|_| window.set_focus())
            .map_err(|error| format!("Nie można pokazać panelu: {error}"))
    }
}

#[tauri::command]
pub fn show_panel(app: AppHandle, panel_id: PanelId) -> Result<(), String> {
    let window = panel_window(&app, panel_id)?;
    window
        .show()
        .and_then(|_| window.set_focus())
        .map_err(|error| format!("Nie można pokazać panelu: {error}"))
}

#[tauri::command]
pub fn hide_panel(app: AppHandle, panel_id: PanelId) -> Result<(), String> {
    panel_window(&app, panel_id)?
        .hide()
        .map_err(|error| format!("Nie można ukryć panelu: {error}"))
}

#[tauri::command]
pub fn set_panel_click_through(
    app: AppHandle,
    panel_id: PanelId,
    enabled: bool,
) -> Result<(), String> {
    panel_window(&app, panel_id)?
        .set_ignore_cursor_events(enabled)
        .map_err(|error| format!("Nie można zmienić trybu kliknięć: {error}"))
}

#[tauri::command]
pub fn apply_panel_window(
    app: AppHandle,
    state: State<'_, AppState>,
    panel_id: PanelId,
) -> Result<(), String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "Blokada ustawień została uszkodzona".to_string())?
        .clone();
    apply_panel_window_core(&app, panel_id, &settings)
}

pub fn apply_panel_window_core(
    app: &AppHandle,
    panel_id: PanelId,
    settings: &AppSettings,
) -> Result<(), String> {
    let window = panel_window(app, panel_id)?;
    let panel = settings.panels.get(panel_id);
    let width = panel.position.width.clamp(260, 1200);
    let height = panel.position.height.clamp(180, 1200);

    window
        .set_size(LogicalSize::new(width as f64, height as f64))
        .map_err(|error| format!("Nie można ustawić rozmiaru: {error}"))?;

    let monitor = window
        .current_monitor()
        .map_err(|error| format!("Nie można pobrać monitora: {error}"))?
        .or_else(|| window.primary_monitor().ok().flatten())
        .ok_or_else(|| "Nie znaleziono monitora".to_string())?;

    let monitor_position = monitor.position();
    let monitor_size = monitor.size();
    let scale_factor = monitor.scale_factor();
    let physical_width = (width as f64 * scale_factor).round() as i32;
    let physical_height = (height as f64 * scale_factor).round() as i32;
    let margin = (18.0 * scale_factor).round() as i32;
    let max_x = monitor_position.x + monitor_size.width as i32 - physical_width;
    let max_y = monitor_position.y + monitor_size.height as i32 - physical_height;

    let (x, y) = match panel.position.preset {
        PositionPreset::TopLeft => (monitor_position.x + margin, monitor_position.y + margin),
        PositionPreset::TopCenter => (
            monitor_position.x + (monitor_size.width as i32 - physical_width) / 2,
            monitor_position.y + margin,
        ),
        PositionPreset::TopRight => (max_x - margin, monitor_position.y + margin),
        PositionPreset::Center => (
            monitor_position.x + (monitor_size.width as i32 - physical_width) / 2,
            monitor_position.y + (monitor_size.height as i32 - physical_height) / 2,
        ),
        PositionPreset::BottomLeft => (monitor_position.x + margin, max_y - margin),
        PositionPreset::BottomRight => (max_x - margin, max_y - margin),
        PositionPreset::Custom => (panel.position.x, panel.position.y),
    };

    window
        .set_position(PhysicalPosition::new(x, y))
        .and_then(|_| window.set_ignore_cursor_events(panel.click_through))
        .map_err(|error| format!("Nie można zastosować położenia panelu: {error}"))
}

#[tauri::command]
pub fn persist_panel_position(
    app: AppHandle,
    state: State<'_, AppState>,
    panel_id: PanelId,
    x: i32,
    y: i32,
) -> Result<(), String> {
    let mut next = state
        .settings
        .lock()
        .map_err(|_| "Blokada ustawień została uszkodzona".to_string())?
        .clone();
    let panel = next.panels.get_mut(panel_id);
    panel.position.preset = PositionPreset::Custom;
    panel.position.x = x;
    panel.position.y = y;

    settings::save(&app, &next)?;
    {
        let mut current = state
            .settings
            .lock()
            .map_err(|_| "Blokada ustawień została uszkodzona".to_string())?;
        *current = next.clone();
    }
    app.emit("settings-updated", next)
        .map_err(|error| format!("Nie można zapisać pozycji panelu: {error}"))
}

#[tauri::command]
pub fn get_notifications(state: State<'_, AppState>) -> Result<Vec<NotificationItem>, String> {
    state
        .notifications
        .lock()
        .map(|items| items.clone())
        .map_err(|_| "Blokada powiadomień została uszkodzona".to_string())
}

#[tauri::command]
pub fn push_demo_notification(state: State<'_, AppState>) -> Result<(), String> {
    let samples = [
        (
            "Discord",
            "3godziny",
            "Mikser audio działa natywnie przez WASAPI.",
            "#5865f2",
            "D",
        ),
        (
            "Moustache",
            "Panel wydajności",
            "CPU i RAM są aktualizowane bez ciężkiego procesu Electron.",
            "#8b5cf6",
            "M",
        ),
        (
            "System",
            "Tryb skupienia",
            "Ten feed można w pełni dostosować kolorystycznie.",
            "#38bdf8",
            "S",
        ),
    ];

    let mut items = state
        .notifications
        .lock()
        .map_err(|_| "Blokada powiadomień została uszkodzona".to_string())?;
    let sample = samples[items.len() % samples.len()];
    items.insert(
        0,
        NotificationItem {
            id: Uuid::new_v4().to_string(),
            app: sample.0.to_string(),
            title: sample.1.to_string(),
            body: sample.2.to_string(),
            timestamp: now_millis(),
            accent: Some(sample.3.to_string()),
            icon_text: Some(sample.4.to_string()),
            read: false,
        },
    );
    items.truncate(24);
    Ok(())
}

#[tauri::command]
pub fn dismiss_notification(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut items = state
        .notifications
        .lock()
        .map_err(|_| "Blokada powiadomień została uszkodzona".to_string())?;
    items.retain(|item| item.id != id);
    Ok(())
}

#[tauri::command]
pub fn clear_notifications(state: State<'_, AppState>) -> Result<(), String> {
    state
        .notifications
        .lock()
        .map_err(|_| "Blokada powiadomień została uszkodzona".to_string())?
        .clear();
    Ok(())
}

#[tauri::command]
pub fn get_runtime_capabilities(state: State<'_, AppState>) -> RuntimeCapabilities {
    let native_loaded = state
        .native
        .lock()
        .map(|native| native.is_some())
        .unwrap_or(false);
    let packaged = is_packaged();

    RuntimeCapabilities {
        native_engine_loaded: native_loaded,
        audio_mixer_available: native_loaded,
        system_notification_listener_available: false,
        system_notification_listener_reason: if packaged {
            "Pakiet ma tożsamość MSIX, ale adapter UserNotificationListener nie jest jeszcze aktywowany w tej kompilacji.".to_string()
        } else {
            "Listener powiadomień innych aplikacji wymaga adaptera MSIX oraz jawnej zgody użytkownika.".to_string()
        },
        packaged,
    }
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

fn panel_window(app: &AppHandle, panel_id: PanelId) -> Result<WebviewWindow, String> {
    app.get_webview_window(panel_id.label())
        .ok_or_else(|| format!("Nie znaleziono okna {}", panel_id.label()))
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn is_packaged() -> bool {
    std::env::var_os("APPX_PACKAGE_FAMILY_NAME").is_some()
        || std::env::var_os("PACKAGE_FAMILY_NAME").is_some()
}
