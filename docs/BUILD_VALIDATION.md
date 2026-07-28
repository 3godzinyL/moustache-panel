# Build validation

## Portable checks completed for 2.0.0

- Strict TypeScript checking for every `src/**/*.ts` and `src/**/*.tsx` file.
- TypeScript checking of `vite.config.ts`.
- Node.js syntax check of `scripts/build-native.mjs`.
- Parsing of every JSON configuration file.
- Parsing of the MSIX XML manifest and both GitHub Actions workflows.
- Verification of required MSIX image dimensions and Tauri PNG integrity.
- Verification that the C header and Rust loader expose the same fixed-width ABI.
- C++20 syntax/warning pass for the portable fallback translation unit.

## Windows release gate

The final native and installer gate must run on Windows because the C++ engine
uses Win32, PDH, DXGI and WASAPI, and Tauri Windows bundles require the MSVC
Rust target and Windows SDK. The included CI and release workflows perform that
gate on `windows-latest`.

Recommended local release commands:

```powershell
npm install
npm run check
node scripts/build-native.mjs
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npm run build
```
