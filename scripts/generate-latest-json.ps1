param(
    [string]$DistDir = "dist",
    [string]$Version = "",
    [string]$Repo = ""
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $DistDir)) {
    throw "dist dir not found: $DistDir"
}

$installer = Get-ChildItem (Join-Path $DistDir "*-installer.exe") | Select-Object -First 1
if (-not $installer) {
    throw "installer exe not found in $DistDir"
}

if (-not $Version) {
    $line = Get-Content config.toml | Select-String -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
    if (-not $line) { throw "version not found in config.toml" }
    $Version = $line.Matches[0].Groups[1].Value
}

$hash = (Get-FileHash $installer.FullName -Algorithm SHA256).Hash.ToLower()

$url = ""
if ($Repo) {
    $tag = "v$Version"
    $url = "https://github.com/$Repo/releases/download/$tag/$($installer.Name)"
}

$manifest = @{
    version = $Version
    installer_url = $url
    sha256 = $hash
} | ConvertTo-Json -Depth 5

$out = Join-Path $DistDir "latest.json"
$manifest | Out-File -FilePath $out -Encoding utf8
Write-Host "wrote $out"
