param(
    [string]$OutDir = "dist"
)

$ErrorActionPreference = "Stop"

cargo build --release --manifest-path builder-rust/Cargo.toml
.\builder-rust\target\release\uvessel-builder.exe --out-dir $OutDir
