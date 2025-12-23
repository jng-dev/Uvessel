# Uvessel Pipeline (Current Scope)

This document captures the current build/install/run flow based on the code in this repo.

## Inputs and configuration

- `config.toml` is the single source of truth for versioning and product metadata.
- The Rust build steps embed config values into binaries at build time:
  - Installer: `installer-rust/build.rs` writes constants into `installer-rust/src/config.rs`.
- The launcher also embeds metadata for Windows resources via `launcher-rust/build.rs`.

Key config fields (current usage):
- `version` is compiled into installer binaries.
- `product_name`, `name`, `app_id`, `company`, `description` drive Windows resource metadata.
- `install_dir` overrides default install root when set.

## Build pipeline (local or CI)

1) Build the launcher (shim):
   - `builder-rust/src/main.rs` runs `cargo build --release` in `launcher-rust`.
   - Result: `launcher-rust/target/release/launcher.exe`.

2) Build the installer UI:
   - `builder-rust/src/main.rs` runs `npm run tauri build` in
     `tauri-ui-rust/webview-installer-rust`.
   - Result: `tauri-ui-rust/webview-installer-rust/src-tauri/target/release/webview-installer-rust.exe`.

3) Stage the launcher shim into the installer:
   - Builder copies the built launcher into `installer-rust/embedded/launcher.exe`.

4) Build the installer binary:
   - Builder runs `cargo build --release` in `installer-rust`.
   - `installer-rust/build.rs`:
     - Reads `config.toml`.
     - Creates `app_payload.zip` from `app/`, `assets/`, and `data/`.
     - Embeds `launcher.exe` as the shim payload.
     - Embeds `installer-ui.exe` as the installer UI payload.
     - Embeds Windows resources (icon, version info).
     - Emits `installer-rust/src/config.rs` constants (version, etc).

5) Produce a distributable installer:
   - Builder copies `installer-rust/target/release/launcher.exe` to `dist/<product>-installer.exe`.
   - In CI, `.github/workflows/build-release.yml` builds and uploads the installer as a GitHub release asset.

## Install flow (installer binary)

1) Determine install directory:
   - Default: `%LOCALAPPDATA%\Uvessel\<product_name>`.
   - If `install_dir` is set in `config.toml`, use that as absolute or under `LOCALAPPDATA\Uvessel`.

2) Check current install state:
   - `installer-rust/src/installer.rs` reads state if it exists.
   - If version matches, it launches the installed app and exits.
   - If the installer version is older than the installed version, it aborts.

3) Install/update:
   - Launch the installer UI and keep it open while install runs.
   - Write the launcher shim to `<install_root>\<product_name>.exe`.
   - If upgrading, remove `app/` and `.runtime/venv`, preserving `data/` and `assets/`.
   - Extract embedded `app_payload.zip` into the install root.
   - Ensure `uv.exe` is present; install Python and sync dependencies using uv.
   - Create Start Menu shortcut.
   - Write updated state metadata.
   - Launch the installed launcher exe.

## Runtime flow (launcher binary)

1) The launcher runs from the install root.
2) Launcher sets UV environment and runs the configured entry point via `uv`.

## Release artifacts

- GitHub Actions builds the installer and publishes it as a release asset.
- The version is read from `config.toml` and matched to the tag.

## Webview installer (current state)

`webview-installer-rust/` provides the installer UI for the Rust installer.
