# Uvessel
Concept project to make distributing Python apps easier. End users do not need Python, uv, or any compilers installed.

## How it works
- A small Rust launcher controls the flow.
- When the launcher runs on a target machine it creates `%APPDATA%/Uvessel/project-name`.
- It downloads uv binaries (Windows only for now) into that folder.
- It copies your Python project into the same folder.
- It sets environment variables so uv uses `%APPDATA%/Uvessel/project-name/.runtime` for all storage (temp files, cache, Python interpreter, and venv).
- It runs `uv sync` for your project.
- If successful, subsequent runs launch the Python app from the entry point (currently hardcoded to `main.py`).

Everything needed to run the app stays in the target folder, with no leftover files elsewhere on the user's machine.

## Setup
1. Clone the repo.
2. Copy your Python project and all dependencies (SQL DB, HTML, JS, images, etc.) into `Uvessel/app`.
3. Optional: place an `.ico` file in `Uvessel/media`.
4. `cd Uvessel/launcher-rust`
5. `cargo build --release`
6. Rename `launcher.exe` to the name of your app.

The executable should now be distributable to any target Windows computer.

## Notes
- This is barebones and tested only on a few personal projects. It started as a concept to see if it worked (and it does).
- Size is large: uv cache is 500MB+ and the venv can be 0.5-1GB.
- Ease of use is great compared to pyinstaller, and launch times are ~10x faster than pyinstaller/nuitka in my tests.
- Feel free to fork or suggest improvements.

## To do
- Minimize to tray.
- Add a simple UI for installation (set ICO at runtime, choose install path, etc.).
- Consider an embedded binary to auto-update.

## Requirements
For building the executable: Rust toolchain is required. Python is not required.
