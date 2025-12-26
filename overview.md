# Uvessel Project Overview

This repository is a template for building and distributing self-contained Python apps. It combines a Rust-based build/installer pipeline with a small Rust launcher and a Tauri-based installer UI, so the final output is a single install directory that can run without a system Python.

## Major parts

- `app/`: Your Python application and any runtime assets you want bundled.
- `assets/`: Shared assets such as icons and branding used by the launcher/installer.
- `builder-rust/`: The build tool that assembles a distributable installer from `app/`, config, and bundled runtime bits.
- `installer-rust/`: The installer core that writes files to disk and manages install/update flow.
- `launcher-rust/`: A thin executable that sets up environment variables and launches the bundled Python app via `uv`.
- `tauri-ui-rust/`: The Tauri desktop UI for the installer, including the webview UI in `webview-installer-rust/`.
- `config.toml`: Central configuration for product naming, entry point, and install paths.
- `dist/`: Output installers produced by the build process.
- `data/` and `assets/` (inside the install location): Persistent directories for user/app data that survive reinstalls.

## How the pieces work together

1. The build tool in `builder-rust/` reads `config.toml` and packages:
   - your Python app from `app/`
   - the `uv` binary and a bundled Python interpreter
   - launcher/installer binaries and assets
2. The installer in `installer-rust/` unpacks everything into the target install directory, creating the persistent `data/` and `assets/` folders.
3. The Tauri UI in `tauri-ui-rust/` provides the installer experience and reflects status/progress from the installer core.
4. After install, the `launcher-rust/` executable sets environment variables and runs the app through the bundled `uv` runtime, so it does not depend on any system Python.

## How it is achieved

The template keeps responsibilities separated:

- Build time: assemble a self-contained install payload with a known layout and assets.
- Install time: write that payload to disk, create persistent directories, and track status.
- Runtime: use a minimal launcher that delegates to `uv` and the bundled interpreter, keeping the launcher small and predictable.

This separation keeps the runtime surface area minimal, makes installs repeatable, and ensures the app runs consistently across machines without system-wide dependencies.
