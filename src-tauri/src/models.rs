use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PanelId {
    Performance,
    Mixer,
    Notifications,
}

impl PanelId {
    pub fn label(self) -> &'static str {
        match self {
            Self::Performance => "performance",
            Self::Mixer => "mixer",
            Self::Notifications => "notifications",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub version: u32,
    pub launch_at_startup: bool,
    pub sample_interval_ms: u64,
    pub panels: PanelsSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelsSettings {
    pub performance: PanelSettings,
    pub mixer: PanelSettings,
    pub notifications: PanelSettings,
}

impl PanelsSettings {
    pub fn get(&self, id: PanelId) -> &PanelSettings {
        match id {
            PanelId::Performance => &self.performance,
            PanelId::Mixer => &self.mixer,
            PanelId::Notifications => &self.notifications,
        }
    }

    pub fn get_mut(&mut self, id: PanelId) -> &mut PanelSettings {
        match id {
            PanelId::Performance => &mut self.performance,
            PanelId::Mixer => &mut self.mixer,
            PanelId::Notifications => &mut self.notifications,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelSettings {
    pub enabled: bool,
    pub show_on_launch: bool,
    pub hotkey: String,
    pub click_through: bool,
    pub visual: PanelVisual,
    pub position: PanelPosition,
    pub content: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelVisual {
    pub theme: ThemePreset,
    pub accent: String,
    pub opacity: f64,
    pub blur: u32,
    pub radius: u32,
    pub scale: f64,
    pub compact: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreset {
    Obsidian,
    Frost,
    Neon,
    Minimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelPosition {
    pub preset: PositionPreset,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PositionPreset {
    TopLeft,
    TopCenter,
    TopRight,
    Center,
    BottomLeft,
    BottomRight,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMetric {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub memory_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSnapshot {
    pub timestamp: u64,
    pub source: SnapshotSource,
    pub cpu: CpuMetric,
    pub gpu: GpuMetric,
    pub ram: MemoryMetric,
    pub disk: StorageMetric,
    pub network: NetworkMetric,
    pub processes: Vec<ProcessMetric>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SnapshotSource {
    Native,
    RustFallback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuMetric {
    pub usage: f32,
    pub temp_c: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuMetric {
    pub usage: Option<f32>,
    pub vram_used_bytes: Option<u64>,
    pub vram_total_bytes: Option<u64>,
    pub temp_c: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryMetric {
    pub usage: f32,
    pub used_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageMetric {
    pub usage: f32,
    pub used_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkMetric {
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioSession {
    pub id: String,
    pub pid: u32,
    pub name: String,
    pub volume: f32,
    pub muted: bool,
    pub peak: f32,
    pub system: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationItem {
    pub id: String,
    pub app: String,
    pub title: String,
    pub body: String,
    pub timestamp: u64,
    pub accent: Option<String>,
    pub icon_text: Option<String>,
    pub read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCapabilities {
    pub native_engine_loaded: bool,
    pub audio_mixer_available: bool,
    pub system_notification_listener_available: bool,
    pub system_notification_listener_reason: String,
    pub packaged: bool,
}
