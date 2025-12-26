Uvessel TODO — question space (no answers)

Product + UX
- What core features are still missing for the installer/launcher experience?
- What are the must-have UX affordances for first-time install vs update?
- What “small polish” wins would make the installer feel more premium?
- What user-facing errors need clearer or more actionable messaging?

Reliability + Safety
- What are the highest-risk failure points in build/install/runtime?
- Which failures would be the most expensive to diagnose after release?
- Are there any “silent failures” that should be surfaced explicitly?
- Where do we need better rollback or retry behavior?

Tauri UI
- Which Tauri features/plugins are we not leveraging but should?
- What permissions can be tightened without breaking functionality?
- Are there any UI state transitions that need stronger guarantees?
- Should we add a “diagnostics” or “export logs” affordance?

Installer + Runtime
- Are there any install-time steps that should be deferred to first run?
- Which parts of the payload could be optional or lazy-loaded?
- Are there any mismatches between config and runtime behavior?
- Where do we need stronger validation of config values?

Build + Pipeline
- What automation is missing (CI, dry runs, smoke tests, nightly builds)?
- Which scripts should be added or consolidated to reduce manual steps?
- What artifacts should be archived from CI for debugging/repro?
- Are we versioning/building in a way that’s fragile or inconsistent?

Code Health
- What modules are duplicated across launcher/installer that should be shared?
- Which files are carrying historical or unused code paths?
- Where do we need clearer ownership boundaries (build/install/runtime/UI)?
- Which configs are unused or could be removed?

Observability
- What logs are missing during install/runtime?
- Should logs be structured (JSON) vs plain text?
- Where should log rotation or size caps exist?
- What telemetry (if any) should be considered?

Security + Trust
- Should we sign installers or binaries? Where in the pipeline?
- Are we downloading any runtime artifacts at install time that should be verified?
- Do we need checksums or integrity validation for payloads?
- What filesystem or registry writes should be minimized or audited?

Documentation
- What project docs are missing (setup, troubleshooting, architecture)?
- Which sections are too vague or out of date?
- Where should we add examples (config, CLI usage, common workflows)?

Other
- What “nice to have” ideas should be captured now for later?
- Any cross-platform considerations that should be planned for?

