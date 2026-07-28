use crate::models::{
    AppSettings, PanelPosition, PanelSettings, PanelVisual, PanelsSettings, PositionPreset,
    ThemePreset,
};
use std::{collections::HashMap, fs, path::PathBuf};
use tauri::{AppHandle, Manager};

fn content(entries: &[(&str, bool)]) -> HashMap<String, bool> {
    entries
        .iter()
        .map(|(key, value)| ((*key).to_string(), *value))
        .collect()
}

fn visual(accent: &str) -> PanelVisual {
    PanelVisual {
        theme: ThemePreset::Obsidian,
        accent: accent.to_string(),
        opacity: 0.86,
        blur: 26,
        radius: 24,
        scale: 1.0,
        compact: false,
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: 2,
            launch_at_startup: false,
            sample_interval_ms: 750,
            panels: PanelsSettings {
                performance: PanelSettings {
                    enabled: true,
                    show_on_launch: true,
                    hotkey: "Alt+1".to_string(),
                    click_through: false,
                    visual: visual("#8b5cf6"),
                    position: PanelPosition {
                        preset: PositionPreset::TopRight,
                        x: 0,
                        y: 0,
                        width: 430,
                        height: 520,
                    },
                    content: content(&[
                        ("cpu", true),
                        ("gpu", true),
                        ("ram", true),
                        ("disk", true),
                        ("network", true),
                        ("processes", true),
                    ]),
                },
                mixer: PanelSettings {
                    enabled: true,
                    show_on_launch: false,
                    hotkey: "Alt+2".to_string(),
                    click_through: false,
                    visual: visual("#22c55e"),
                    position: PanelPosition {
                        preset: PositionPreset::BottomRight,
                        x: 0,
                        y: 0,
                        width: 480,
                        height: 520,
                    },
                    content: content(&[("master", true), ("sessions", true), ("peaks", true)]),
                },
                notifications: PanelSettings {
                    enabled: true,
                    show_on_launch: false,
                    hotkey: "Alt+3".to_string(),
                    click_through: false,
                    visual: visual("#38bdf8"),
                    position: PanelPosition {
                        preset: PositionPreset::TopRight,
                        x: 0,
                        y: 0,
                        width: 410,
                        height: 540,
                    },
                    content: content(&[
                        ("app", true),
                        ("timestamp", true),
                        ("icon", true),
                        ("body", true),
                    ]),
                },
            },
        }
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Nie można ustalić katalogu ustawień: {error}"))?;
    fs::create_dir_all(&dir)
        .map_err(|error| format!("Nie można utworzyć katalogu ustawień: {error}"))?;
    Ok(dir.join("settings.json"))
}

pub fn load(app: &AppHandle) -> AppSettings {
    let Ok(path) = settings_path(app) else {
        return AppSettings::default();
    };
    let Ok(raw) = fs::read_to_string(path) else {
        return AppSettings::default();
    };
    normalize(serde_json::from_str(&raw).unwrap_or_default())
}

pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let raw = serde_json::to_string_pretty(settings)
        .map_err(|error| format!("Nie można serializować ustawień: {error}"))?;
    fs::write(path, raw).map_err(|error| format!("Nie można zapisać ustawień: {error}"))
}

pub fn normalize(mut settings: AppSettings) -> AppSettings {
    settings.version = 2;
    settings.sample_interval_ms = settings.sample_interval_ms.clamp(350, 2_500);

    for panel in [
        &mut settings.panels.performance,
        &mut settings.panels.mixer,
        &mut settings.panels.notifications,
    ] {
        panel.hotkey = panel.hotkey.trim().chars().take(64).collect();
        panel.visual.opacity = panel.visual.opacity.clamp(0.35, 1.0);
        panel.visual.blur = panel.visual.blur.min(60);
        panel.visual.radius = panel.visual.radius.min(40);
        panel.visual.scale = panel.visual.scale.clamp(0.75, 1.30);
        panel.position.width = panel.position.width.clamp(260, 1_200);
        panel.position.height = panel.position.height.clamp(180, 1_200);
    }

    settings
}
