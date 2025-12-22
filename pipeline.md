# Uvessel Pipeline (Current Scope)

This document captures the current build/install/run/update flow based on the code in this repo.
It reflects what exists today, plus the explicit assumptions wired into the updater.

## Inputs and configuration

- `config.toml` is the single source of truth for versioning and product metadata.
- The Rust build steps embed config values into binaries at build time:
  - Installer: `installer-rust/build.rs` writes constants into `installer-rust/src/config.rs`.
  - Updater: `updater-rust/build.rs` writes constants into `updater-rust/src/config.rs`.
- The launcher also embeds metadata for Windows resources via `launcher-rust/build.rs`.

Key config fields (current usage):
- `version` is compiled into installer and updater binaries.
- `product_name`, `name`, `app_id`, `company`, `description` drive Windows resource metadata.
- `uvessel_instance_link` is used as a base URL for update discovery.
- `auto_update_enabled` and `update_manifest_url` control auto-updater behavior.
- `install_dir` overrides default install root when set.

## Build pipeline (local or CI)

1) Build the launcher (shim):
   - `builder-rust/src/main.rs` runs `cargo build --release` in `launcher-rust`.
   - Result: `launcher-rust/target/release/launcher.exe`.

2) Stage the launcher shim into the installer:
   - Builder copies the built launcher into `installer-rust/embedded/launcher.exe`.

3) Build the installer binary:
   - Builder runs `cargo build --release` in `installer-rust`.
   - `installer-rust/build.rs`:
     - Reads `config.toml`.
    - Creates `app_payload.zip` from `app/`, `assets/`, and `data/`.
    - Embeds `launcher.exe` as the shim payload.
    - Embeds `updater.exe` as the updater payload.
    - Embeds Windows resources (icon, version info).
    - Emits `installer-rust/src/config.rs` constants (version, links, etc).

4) Produce a distributable installer:
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
   - Write the launcher shim to `<install_root>\<product_name>.exe`.
   - If upgrading, remove `app/` and `.runtime/venv`, preserving `data/` and `assets/`.
   - Extract embedded `app_payload.zip` into the install root.
   - Ensure `uv.exe` is present; install Python and sync dependencies using uv.
   - Create Start Menu shortcut.
   - Write updated state metadata.
   - Launch the installed launcher exe.

## Runtime flow (launcher binary)

1) The launcher runs from the install root.
2) It optionally invokes `updater.exe` first (if present).
   - If updater returns exit code `10`, launcher exits early.
3) Launcher sets UV environment and runs the configured entry point via `uv`.

## Auto-update flow (updater binary)

1) Updater reads config constants embedded at build time:
   - `AUTO_UPDATE_ENABLED`
   - `UPDATE_MANIFEST_URL` (explicit)
   - `UVESSEL_INSTANCE_LINK` (fallback base)
   - `VERSION`

2) If updates are enabled:
   - Resolve manifest URL:
     - Prefer `update_manifest_url`.
     - Else use `<uvessel_instance_link>/latest/download/latest.json`.
   - Fetch manifest JSON:
     - Expected fields: `version`, `installer_url`, optional `sha256`.
   - Compare manifest `version` to compiled `VERSION` (semver).
   - If newer:
     - Download installer to a temp file.
     - Verify SHA-256 if provided.
     - Launch the installer and exit with code `10`.

Notes:
- The updater is a separate binary (`updater.exe`) so it can exit before the installer replaces files.
- If updater fails, it logs a warning and the launcher continues.

## Release/update artifacts

What exists today:
- GitHub Actions builds the installer and publishes it as a release asset.
- The version is read from `config.toml` and matched to the tag.

What is assumed by the updater:
- A `latest.json` manifest is published alongside the installer.

What exists today:
- The GitHub Actions workflow generates `dist/latest.json` and uploads it with the release.

Recommended manifest shape:
```
{
  "version": "1.2.3",
  "installer_url": "https://github.com/<owner>/<repo>/releases/download/v1.2.3/<product>-installer.exe",
  "sha256": "<hex>"
}
```

Local testing helper:
- `scripts/generate-latest-json.ps1` builds a `dist/latest.json` from the local installer.

## Webview installer (current state)

`webview-installer-rust/` is a Tauri + SvelteKit starter.
It currently does not integrate with the installer/build pipeline above.
No wiring exists yet between the webview UI and the Rust installer or updater.
