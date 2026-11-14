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

Show-WorkLoreLegacyBuildWarning
$layout = Get-WorkLoreBuildLayout -Channel "dev"
Initialize-WorkLoreBuildLayout -Layout $layout
$env:WORKLORE_BUILD_CHANNEL = "dev"
$env:WORKLORE_BUILD_ROOT = $layout.BuildRoot
$env:WORKLORE_FRONTEND_DIST = $layout.FrontendDist
$env:CARGO_TARGET_DIR = $layout.CargoTarget

if (-not (Test-Path "node_modules")) {
    Write-Host "Installing frontend dependencies..."
    npm install
}

Write-Host "Repository: $repoRoot"
Write-Host "External build root: $($layout.BuildRoot)"
Write-Host "Cargo compilation cache: $($layout.CargoTarget)"
npm run desktop:dev:raw
