# Uvessel

Uvessel is a template for distributing **self-contained Python applications** without requiring a global Python interpreter or developer tools on the target machine.

It uses **uv** for Python environment management and a **thin Rust launcher** to execute the application. The final output is a **single install directory** containing:

* a bundled Python interpreter
* uv binaries
* your application source code and dependencies
* a minimal launcher executable

Once installed, the application runs independently of any system-level Python installation.

For a full walkthrough of the architecture and data flow, see `overview.md`.

---

## Important

Uvessel is a **template**, not a turn-key solution.

It is intended to be adapted to your project. The end result is either:

* an automated pipeline that produces distributable `.exe` installers, or
* a one-off distributable built locally

You are expected to:

* have the Rust toolchain installed
* understand your application’s runtime requirements
* make a small number of project-specific changes (outlined below)

If you are looking for a drag-and-drop binary packager, this is not that.

---

## How it works (high level)

1. Your Python project is bundled as part of the build.
2. A Rust launcher is compiled that:

   * sets the required environment variables
   * invokes the bundled `uv` binary
   * runs your application entry point
3. An installer places everything into a single directory.
4. The launcher is used to run the application after installation.

The launcher itself is intentionally small and dumb. All setup happens at install time.

Think of the pipeline as three phases:

- Build time: `builder-rust/` assembles a distributable payload based on `config.toml`.
- Install time: `installer-rust/` writes files to disk, creates persistent directories, and reports progress.
- Runtime: `launcher-rust/` boots the bundled Python app through `uv` without system dependencies.

---

## Repository layout

- `app/`: Your Python application and any runtime assets you want bundled.
- `assets/`: Shared assets such as icons and branding for the installer/launcher.
- `builder-rust/`: Build tool that assembles the installer payload.
- `installer-rust/`: Installer core responsible for disk layout and install/update logic.
- `launcher-rust/`: Minimal runtime launcher for the bundled app.
- `tauri-ui-rust/`: Tauri-based installer UI project.
- `dist/`: Output installers produced by the build process.
- `config.toml`: Central config for product naming, entry point, and install paths.

---

## Usage

1. Clone this repository.

2. Place your Python project inside the `app/` directory.
   This can include:

   * Python source code
   * assets
   * frontend files
   * compiled binaries or other runtime resources

3. Edit `config.toml` to match your project:

   * application name
   * entry point
   * Python / uv configuration
   * install paths

4. (Optional) Place an `.ico` file in `assets/` or another configured location if you want a custom executable icon.

5. Build the installer:

   * run the Rust builder in `builder-rust/`, or
   * use the provided GitHub Actions workflow in `.github/` to produce artifacts automatically

6. The resulting installer will be placed in `dist/`.
   This is the file you distribute to end users.

### Configuration (`config.toml`)

The most commonly edited fields are:

* `app_id`: Stable application identifier.
* `name` / `product_name`: User-facing names used by the installer.
* `company`: Publisher/brand name.
* `description`: Short product description.
* `version`: Semantic version string.
* `entry_point`: Python entry point to run at startup.
* `icon`: Path to your `.ico` file.
* `install_dir`: Optional custom install root (relative paths resolve under `LOCALAPPDATA\Uvessel`).

### Build commands (local)

```
cargo build --release --manifest-path builder-rust/Cargo.toml
.\builder-rust\target\release\uvessel-builder.exe
```

This produces `dist/<product_name>-installer.exe`.

### Install location

By default, installs go to:
`%LOCALAPPDATA%\Uvessel\<product_name>`

Optional override in `config.toml`:

```
# install_dir can be absolute or relative.
# If relative, it is placed under LOCALAPPDATA\Uvessel.
install_dir = "MyApps"
```

---

## Persistent storage

The installer creates the following directories inside the install location:

* `data/`
* `assets/`

These directories are **never modified or deleted** by reinstalling or updating the application, even if:

* the same installer is run again
* a newer version is installed

They are intended for application-managed or user-managed data, such as:

* SQLite databases
* configuration files
* cached data
* large assets (e.g. game data, models, datasets)

What your application stores there is entirely up to you.

---

## Installer UI

The installer experience is built with Tauri and a webview UI:

- UI: `tauri-ui-rust/webview-installer-rust/` (Svelte front-end)
- Tauri host: `tauri-ui-rust/webview-installer-rust/src-tauri/`

The UI reflects status and logs coming from the installer core.

---

## Design goals

* No global Python dependency on the target system
* No requirement for developer tools on the target system
* Minimal launcher surface area
* Clear separation between build, install, and runtime
* Predictable install directory layout
* Installer can be re-run safely
