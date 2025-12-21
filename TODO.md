TODO

# Core goals

* Provide a simple, deterministic way to run a Python application via a Rust launcher.
* Treat the launcher as disposable and rebuildable; treat user data as sacred.
* Avoid background magic, auto-fetching, or self-mutating behavior in v1.


# 3) Installer / setup UX

* Replace terminal-only prompts with a minimal UI (or simple guided flow).
* Minimum setup UX:
  * Show progress (runtime setup, venv creation)
  * Clear error messages



# Still missing
* Installer mini UI.
* Optional: tag/config version guard in GitHub Actions.
