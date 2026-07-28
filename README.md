# Moustache Control Center 2.0

A lightweight, modern Windows overlay center rebuilt from the original Electron
prototype. It provides configurable performance, audio mixer and notification
panels without bundling the heavy Electron/Chromium runtime.

> Polish documentation is first. English version is below.

## Najważniejsze funkcje

- **Panel wydajności**: CPU, GPU, VRAM, RAM, dysk, transfer sieciowy i procesy.
- **Natywny mikser Windows**: master volume oraz sesje aplikacji przez WASAPI.
- **Własny panel powiadomień**: konfigurowalny feed, kolory, ikony i układ.
- **Trzy niezależne overlaye** z osobnymi skrótami, pozycjami i rozmiarami.
- **Cztery presety wyglądu**: Obsidian, Frost, Neon i Minimal.
- Przezroczystość, blur, accent color, radius, skala i tryb kompaktowy.
- Click-through, always-on-top, autostart i zapisywanie ustawień.
- MSI/NSIS, GitHub Actions i gotowa struktura repozytorium.

## Architektura

```text
React 19 + TypeScript
        │ Tauri invoke
        ▼
Rust / Tauri 2 core
        │ dynamic C ABI
        ▼
moustache_native.dll (C++20)
        ├─ Win32 system metrics
        ├─ PDH GPU counters
        ├─ DXGI video memory
        └─ WASAPI audio sessions
```

Biblioteka natywna jest **normalnie ładowana przez FFI**. Projekt nie wykonuje
iniekcji DLL do żadnego procesu. Iniekcja do własnej aplikacji niczego tu nie
przyspiesza, utrudnia podpisywanie, aktualizacje i debugowanie, a zwiększa
powierzchnię błędów.

Gdy DLL nie może się załadować, panel wydajności automatycznie używa lekkiego
fallbacku `sysinfo` w Rust. Mikser audio wymaga natywnej biblioteki Windows.

## Wymagania

- Windows 10 lub Windows 11 x64
- Node.js 22+
- Rust stable (MSVC toolchain)
- Visual Studio 2022 Build Tools:
  - Desktop development with C++
  - Windows 10/11 SDK
  - CMake tools for Windows
- WebView2 Runtime

## Uruchomienie deweloperskie

```powershell
npm install
npm run dev
```

Skrypt przed startem buduje `native/moustache_native.dll`, kopiuje ją do
`src-tauri/resources`, uruchamia Vite i aplikację Tauri.

## Kontrola projektu

```powershell
npm run check
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

## Build produkcyjny

```powershell
npm install
npm run build
```

Gotowe instalatory pojawią się w:

```text
src-tauri/target/release/bundle/msi/
src-tauri/target/release/bundle/nsis/
```

Tauri buduje instalatory Windows na komputerze Windows. Workflow
`.github/workflows/release.yml` wykonuje ten proces automatycznie i publikuje
artefakty dla tagów `v*`.

## Skróty domyślne

| Panel | Skrót |
|---|---|
| Wydajność | `Alt+1` |
| Mikser | `Alt+2` |
| Powiadomienia | `Alt+3` |

Każdy skrót można zmienić w centrum sterowania.

## Powiadomienia Windows — ważne ograniczenie systemowe

Standardowy instalator EXE/MSI nie może po prostu przejąć i niezawodnie
wyłączyć wszystkich powiadomień innych aplikacji. `UserNotificationListener`
wymaga:

1. tożsamości pakietu MSIX,
2. deklaracji capability `userNotificationListener`,
3. jawnej zgody użytkownika,
4. obsługi sytuacji, w której użytkownik cofnie zgodę.

Repo zawiera poprawny manifest MSIX, assety i dokumentację przygotowującą tę ścieżkę, ale
domyślny MSI/NSIS używa własnego feedu Moustache. Projekt nie zmienia globalnych
ustawień Windows bez świadomej decyzji użytkownika i nie udaje, że potrafi
stłumić każdy toast niezależnie od źródła.

Szczegóły: [`docs/NOTIFICATIONS.md`](docs/NOTIFICATIONS.md).

## Temperatura CPU/GPU

Windows nie oferuje jednego szybkiego, stabilnego API dla temperatur każdego
CPU i GPU. Bazowa kompilacja nie instaluje sterownika ani adaptera konkretnego
producenta, dlatego pola temperatury zwracają `null`. UI poprawnie ukrywa brak
danych zamiast pokazywać `NaN` albo fałszywe `0°C`. Integracje vendor-specific
(np. NVAPI, ADLX lub osobny serwis sensorów) mają przygotowane pole w modelu i
mogą zostać dodane jako odseparowane adaptery bez zmiany interfejsu panelu.

## Bezpieczeństwo i prywatność

- Brak serwera HTTP.
- Brak telemetrii i połączeń zewnętrznych.
- CSP blokuje zewnętrzne skrypty.
- Brak zdalnych fontów, CDN i Font Awesome.
- Natywna biblioteka ma mały, stabilny interfejs C ABI.
- Ustawienia są lokalne w katalogu danych aplikacji.

## Struktura

```text
src/                    React UI i overlaye
src-tauri/              Rust core i konfiguracja Tauri
native/                 C++20 Win32/WASAPI/DXGI collector
scripts/                build native + MSIX helpers
packaging/msix/         manifest dla package identity
docs/                   architektura i ograniczenia API
.github/workflows/      CI i build release
```

---

# English

Moustache Control Center is a Windows-first overlay utility rebuilt from the
original Electron prototype with Tauri 2, Rust, React and a small C++20 native
engine.

### Features

- Configurable performance overlay.
- Native WASAPI master and per-application volume mixer.
- Custom notification feed and MSIX notification-listener preparation.
- Independent hotkeys, positions, sizes and click-through behavior.
- Obsidian, Frost, Neon and Minimal themes.
- MSI/NSIS builds and GitHub Actions release workflow.
- Rust fallback when the native metrics DLL is unavailable.

### Development

```powershell
npm install
npm run dev
```

### Production build

```powershell
npm run build
```

See the Polish sections above and the files in `docs/` for detailed
architecture, notification constraints and release instructions.
