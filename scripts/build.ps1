param(
    [ValidateSet("validate", "qa", "release")]
    [string]$Channel = "qa",
    [switch]$Clean,
    [switch]$SkipTests,
    [switch]$SkipBundle
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot "build-layout.ps1")

$repoRoot = Get-WorkLoreRepoRoot
Set-Location $repoRoot
& (Join-Path $PSScriptRoot "assert-repo-clean.ps1")

if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    throw "npm was not found. Install a current Node.js LTS release and reopen the terminal."
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo was not found. Install the Rust toolchain and reopen the terminal."
}

$layout = Get-WorkLoreBuildLayout -Channel $Channel
Initialize-WorkLoreBuildLayout -Layout $layout -Clean:$Clean

$env:WORKLORE_BUILD_CHANNEL = $Channel
$env:WORKLORE_BUILD_ROOT = $layout.BuildRoot
$env:WORKLORE_FRONTEND_DIST = $layout.FrontendDist
$env:CARGO_TARGET_DIR = $layout.CargoTarget

Write-Host "Repository: $repoRoot"
Write-Host "Build root: $($layout.BuildRoot)"
Write-Host "Cargo target: $($layout.CargoTarget)"
Write-Host "Frontend output: $($layout.FrontendDist)"

if (-not (Test-Path "node_modules")) {
    Write-Host "Installing frontend dependencies..."
    npm install
}

if (-not $SkipTests) {
    npm run test
}

npm run build:frontend

if (-not $SkipTests) {
    cargo fmt --manifest-path src-tauri/Cargo.toml --all --check
    cargo test --manifest-path src-tauri/Cargo.toml
    cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
}

$shouldBundle = -not $SkipBundle -and $Channel -ne "validate"
if ($shouldBundle) {
    $productName = if ($Channel -eq "qa") { "WorkLore QA" } else { "WorkLore" }
    $identifier = if ($Channel -eq "qa") {
        "org.threewheeledsloth.worklore.qa"
    }
    else {
        "org.threewheeledsloth.worklore"
    }

    $override = @{
        productName = $productName
        identifier = $identifier
        build = @{
            beforeBuildCommand = "cmd /c echo Using externally staged frontend."
            frontendDist = $layout.FrontendDist
        }
        bundle = @{
            active = $true
            targets = @("nsis")
        }
    }
    $override | ConvertTo-Json -Depth 10 | Set-Content -Encoding UTF8 $layout.ConfigPath

    npm run tauri -- build --config $layout.ConfigPath
}

$gitCommit = $null
try {
    $gitCommit = (git rev-parse HEAD).Trim()
}
catch {
    $gitCommit = $null
}

$manifest = [ordered]@{
    schemaVersion = 1
    channel = $Channel
    builtAt = (Get-Date).ToUniversalTime().ToString("o")
    repository = $repoRoot
    buildRoot = $layout.BuildRoot
    frontendDist = $layout.FrontendDist
    cargoTarget = $layout.CargoTarget
    bundled = $shouldBundle
    gitCommit = $gitCommit
}
$manifest | ConvertTo-Json -Depth 5 | Set-Content -Encoding UTF8 (Join-Path $layout.BuildRoot "build-manifest.json")

& (Join-Path $PSScriptRoot "assert-repo-clean.ps1")
Write-Host "WorkLore $Channel build completed outside the repository."
