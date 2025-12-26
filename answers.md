Uvessel TODO — answers (initial draft)

Product + UX
- Core missing features: clear success/failure states, retry and close flows, and a visible log toggle. You already have status + logs; consider adding a simple "copy log" action.
- Must-have UX: deterministic progress messaging for install/update, and clear exit actions (close/launch). Current UI largely covers this.
- Small polish wins: consistent spacing, sharper text rendering on transparent windows, and subtle focus/hover feedback for buttons.
- Error messaging: surface common failures (missing uv, missing payload, write permissions) with short, user-friendly copy.

Reliability + Safety
- Highest-risk points: payload extraction, uv download, and writing to install directories.
- Most expensive failures: partial installs with mismatched runtime vs app payload, or uv install failures without clear logs.
- Silent failures: any skipped file operations or retries that end in success should still log warnings.
- Rollback/retry: add retry with backoff for network and file writes; keep a "state.json" marker to resume or rollback.

Tauri UI
- Unused features: window-level controls (minimize, reopen) are not leveraged; you can avoid extra plugins and keep capability scope tight.
- Permissions: keep to only core window + required APIs; opener not needed (already removed).
- UI transitions: guard polling with shutdown logic and ensure timers are cleared (already in place).
- Diagnostics: add "export logs" or "copy logs" for support.

Installer + Runtime
- Defer to first run: python environment sync and cache warming could run on first launch if install speed is critical.
- Optional payload: large assets can be placed in persistent assets/ and downloaded on demand.
- Config vs runtime mismatch: ensure config entry_point exists and is a supported format (already validated).
- Validation: check install_dir conflicts and write permission before extraction.

Build + Pipeline
- Missing automation: CI build for builder + installer + UI; smoke test install; artifact upload.
- Scripts: one script to build UI + installer + launcher and verify outputs; optional "clean" script for dist/target caches.
- Artifacts: upload installer exe, build logs, and a manifest of file hashes.
- Versioning: ensure config version feeds all outputs (launcher metadata, installer UI title, and artifact naming).

Code Health
- Duplication: launcher and installer both implement uv run flows; consider a shared crate or copied minimal helper with notes.
- Historical paths: remove unused modules (started); keep dead code out of src to avoid confusion.
- Boundaries: keep builder (packaging), installer (disk layout), launcher (runtime) separated; avoid cross-calls.
- Unused configs: add checks for empty icon, unused install_dir, or missing app folder.

Observability
- Missing logs: log start/end of major phases (payload extraction, uv install, sync, launch).
- Structured logs: optional, but a simple prefix or JSON mode could help later.
- Rotation/caps: cap log file size or rotate on launch to avoid long-term growth.
- Telemetry: avoid by default; if added, make it opt-in.

Security + Trust
- Signing: sign installer and launcher as a build step if distributing publicly.
- Downloads: verify uv download checksum or pin a version.
- Integrity: store and verify hash manifest for payload content.
- Writes: minimize writes outside install_dir and avoid registry unless necessary.

Documentation
- Missing docs: troubleshooting guide and "known failure modes" doc.
- Vague sections: build steps and config fields could include examples (added in README).
- Examples: add sample config.toml, and a minimal "hello world" app under app/.

Other
- Nice to have: self-update flow for uv or app payload.
- Cross-platform: explicitly document Windows-only assumptions (uv exe naming, CreateMutex, WinRes).
