# Architecture

## Goals

1. Keep the idle footprint small.
2. Never poll expensive data from multiple layers at once.
3. Separate UI, orchestration and Windows-specific code.
4. Keep the native ABI stable and replaceable.
5. Fail gracefully when a sensor or capability is unavailable.

## Runtime layers

### React / WebView2

The frontend renders four windows from one bundle:

- `main`
- `performance`
- `mixer`
- `notifications`

The window label selects the component. No local web server is used in
production.

### Rust / Tauri

Rust owns:

- window lifecycle,
- position and size,
- settings persistence,
- global command surface,
- autostart integration,
- fallback metrics,
- native library lifetime,
- notification feed state.

Commands return typed JSON values. No raw pointers cross into JavaScript.

### C++ native engine

`moustache_native.dll` exposes five C ABI functions. Rust dynamically loads
the DLL and copies the function pointers while retaining the library handle.

The native collector uses:

- `GetSystemTimes` for CPU load,
- `GlobalMemoryStatusEx` for RAM,
- `GetDiskFreeSpaceExW` for disk capacity,
- `GetIfTable2` for network throughput,
- PDH GPU Engine counters,
- DXGI 1.4 for local video memory,
- Toolhelp + process APIs for process CPU/RAM,
- WASAPI/Core Audio for master and per-session volume.

## Why there is no DLL injection

The DLL and Rust process are both parts of the same trusted application.
Ordinary dynamic loading already provides direct in-process calls with
negligible overhead. Injection would add undefined startup ordering, security
software alerts, signing problems and crash risk without reducing latency.

## Sampling model

The performance overlay requests samples only while its window is visible.
CPU, memory, GPU load and network deltas remain responsive. Process enumeration
and VRAM are cached for two seconds, while disk capacity is cached for ten
seconds. Native state stores previous counters needed for deltas. The UI
updates existing React elements and animates only bar widths.

The fallback collector remains behind the same command and is used only when
the native DLL is unavailable or returns invalid data.

## Error handling

- Missing GPU counters produce `null`.
- The base build does not pretend to expose universal CPU/GPU package
  temperatures; missing vendor sensor adapters produce `null`.
- Missing DLL activates Rust fallback.
- Missing audio engine displays a clear empty state.
- Settings parse errors revert to versioned defaults.
