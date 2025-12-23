param(
    [string]$OutDir = "dist"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$builderManifest = Join-Path $repoRoot "builder-rust\Cargo.toml"

cargo build --release --manifest-path $builderManifest

$builderExe = Join-Path $repoRoot "builder-rust\target\release\uvessel-builder.exe"
& $builderExe --out-dir $OutDir

$distPath = Join-Path $repoRoot $OutDir
$installer = Get-ChildItem -Path $distPath -Filter "*-installer.exe" | Select-Object -First 1
if (-not $installer) {
    throw "installer exe not found in $distPath"
}

& $installer.FullName
