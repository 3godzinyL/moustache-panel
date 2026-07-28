use crate::models::{AudioSession, SystemSnapshot};
use libloading::Library;
use serde::de::DeserializeOwned;
use std::{
    ffi::{c_char, CStr, CString},
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};

type GetJsonFn = unsafe extern "C" fn() -> *mut c_char;
type FreeStringFn = unsafe extern "C" fn(*mut c_char);
type SetVolumeFn = unsafe extern "C" fn(*const c_char, f32) -> i32;
type SetMuteFn = unsafe extern "C" fn(*const c_char, i32) -> i32;

pub struct NativeEngine {
    _library: Library,
    get_system_snapshot: GetJsonFn,
    get_audio_sessions: GetJsonFn,
    free_string: FreeStringFn,
    set_audio_volume: SetVolumeFn,
    set_audio_mute: SetMuteFn,
}

impl NativeEngine {
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let candidates = candidate_paths(app);
        let path = candidates
            .iter()
            .find(|candidate| candidate.exists())
            .ok_or_else(|| {
                format!(
                    "Nie znaleziono moustache_native.dll. Sprawdzono: {}",
                    candidates
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;

        // SAFETY: the DLL path is controlled by the application bundle and all symbols
        // use the documented stable C ABI from native/include/moustache_native.h.
        let library = unsafe { Library::new(path) }
            .map_err(|error| format!("Nie można załadować {}: {error}", path.display()))?;

        // Copy function pointers so the symbols do not retain a borrow. The Library stays
        // alive for the full lifetime of NativeEngine.
        let get_system_snapshot = unsafe {
            *library
                .get::<GetJsonFn>(b"mp_get_system_snapshot_json\0")
                .map_err(|error| format!("Brak mp_get_system_snapshot_json: {error}"))?
        };
        let get_audio_sessions = unsafe {
            *library
                .get::<GetJsonFn>(b"mp_get_audio_sessions_json\0")
                .map_err(|error| format!("Brak mp_get_audio_sessions_json: {error}"))?
        };
        let free_string = unsafe {
            *library
                .get::<FreeStringFn>(b"mp_free_string\0")
                .map_err(|error| format!("Brak mp_free_string: {error}"))?
        };
        let set_audio_volume = unsafe {
            *library
                .get::<SetVolumeFn>(b"mp_set_audio_session_volume\0")
                .map_err(|error| format!("Brak mp_set_audio_session_volume: {error}"))?
        };
        let set_audio_mute = unsafe {
            *library
                .get::<SetMuteFn>(b"mp_set_audio_session_mute\0")
                .map_err(|error| format!("Brak mp_set_audio_session_mute: {error}"))?
        };

        Ok(Self {
            _library: library,
            get_system_snapshot,
            get_audio_sessions,
            free_string,
            set_audio_volume,
            set_audio_mute,
        })
    }

    fn call_json<T: DeserializeOwned>(&self, function: GetJsonFn) -> Result<T, String> {
        // SAFETY: function comes from the verified DLL ABI and returns an allocated,
        // NUL-terminated UTF-8 string that must be released with mp_free_string.
        let pointer = unsafe { function() };
        if pointer.is_null() {
            return Err("Natywny silnik zwrócił pusty wskaźnik".to_string());
        }

        unsafe {
            let raw = CStr::from_ptr(pointer).to_string_lossy().into_owned();
            (self.free_string)(pointer);
            serde_json::from_str::<T>(&raw)
                .map_err(|error| format!("Niepoprawny JSON z native engine: {error}; payload={raw}"))
        }
    }

    pub fn snapshot(&self) -> Result<SystemSnapshot, String> {
        self.call_json(self.get_system_snapshot)
    }

    pub fn audio_sessions(&self) -> Result<Vec<AudioSession>, String> {
        self.call_json(self.get_audio_sessions)
    }

    pub fn set_volume(&self, id: &str, volume: f32) -> Result<(), String> {
        let id = CString::new(id).map_err(|_| "Niepoprawny identyfikator sesji".to_string())?;
        // SAFETY: pointer remains valid for the duration of the call.
        let result = unsafe { (self.set_audio_volume)(id.as_ptr(), volume.clamp(0.0, 1.0)) };
        (result != 0)
            .then_some(())
            .ok_or_else(|| "Nie udało się zmienić głośności sesji".to_string())
    }

    pub fn set_mute(&self, id: &str, muted: bool) -> Result<(), String> {
        let id = CString::new(id).map_err(|_| "Niepoprawny identyfikator sesji".to_string())?;
        // SAFETY: pointer remains valid for the duration of the call.
        let result = unsafe { (self.set_audio_mute)(id.as_ptr(), i32::from(muted)) };
        (result != 0)
            .then_some(())
            .ok_or_else(|| "Nie udało się zmienić wyciszenia sesji".to_string())
    }
}

fn candidate_paths(app: &AppHandle) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(resource_dir) = app.path().resource_dir() {
        paths.push(resource_dir.join("moustache_native.dll"));
        paths.push(resource_dir.join("resources").join("moustache_native.dll"));
    }

    if let Ok(executable) = std::env::current_exe() {
        if let Some(dir) = executable.parent() {
            paths.push(dir.join("moustache_native.dll"));
            paths.push(dir.join("resources").join("moustache_native.dll"));
        }
    }

    paths.push(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("moustache_native.dll"),
    );
    paths
}
