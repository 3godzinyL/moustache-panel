use crate::{
    fallback::FallbackCollector,
    models::{AppSettings, NotificationItem},
    native_bridge::NativeEngine,
};
use std::sync::Mutex;

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub native: Mutex<Option<NativeEngine>>,
    pub fallback: Mutex<FallbackCollector>,
    pub notifications: Mutex<Vec<NotificationItem>>,
}
