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
. (Join-Path $PSScriptRoot "windows-build-env.ps1")

$repoRoot = Get-WorkLoreRepoRoot
Set-Location $repoRoot
& (Join-Path $PSScriptRoot "assert-repo-clean.ps1")

if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    throw "npm was not found. Install a current Node.js LTS release and reopen the terminal."
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo was not found. Install the Rust toolchain and reopen the terminal."
}

if (-not (Get-Command python -ErrorAction SilentlyContinue)) {
    throw "python was not found. Python 3 with PyYAML is required for repository validation."
}

Write-Host "Validating repository path safety and project memory..."
python scripts/check-case-collisions.py
if ($LASTEXITCODE -ne 0) { throw "Tracked-path validation failed with exit code $LASTEXITCODE." }
python refs/tools/validate_refs.py --mode initialized
if ($LASTEXITCODE -ne 0) { throw "Project-reference validation failed with exit code $LASTEXITCODE." }
python refs/tools/generate_agent_context.py --check
if ($LASTEXITCODE -ne 0) { throw "Bounded agent-context validation failed with exit code $LASTEXITCODE." }

Show-WorkLoreLegacyBuildWarning
$layout = Get-WorkLoreBuildLayout -Channel $Channel
Initialize-WorkLoreBuildLayout -Layout $layout -Clean:$Clean

$env:WORKLORE_BUILD_CHANNEL = $Channel
$env:WORKLORE_BUILD_ROOT = $layout.BuildRoot
$env:WORKLORE_FRONTEND_DIST = $layout.FrontendDist
$env:CARGO_TARGET_DIR = $layout.CargoTarget

Write-Host "Repository: $repoRoot"
Write-Host "External root: $($layout.ExternalRoot)"
Write-Host "Build root: $($layout.BuildRoot)"
Write-Host "Cargo target: $($layout.CargoTarget)"
Write-Host "Frontend output: $($layout.FrontendDist)"

if (-not (Test-Path "node_modules")) {
    Write-Host "Installing frontend dependencies..."
    npm install
    if ($LASTEXITCODE -ne 0) { throw "npm install failed with exit code $LASTEXITCODE." }
}

Import-WorkLoreVisualStudioEnvironment

if (-not $SkipTests) {
    npm run test
    if ($LASTEXITCODE -ne 0) { throw "Frontend tests failed with exit code $LASTEXITCODE." }
}

npm run build:frontend
if ($LASTEXITCODE -ne 0) { throw "Frontend build failed with exit code $LASTEXITCODE." }

if (-not $SkipTests) {
    cargo fmt --manifest-path src-tauri/Cargo.toml --all --check
    if ($LASTEXITCODE -ne 0) { throw "Rust formatting check failed with exit code $LASTEXITCODE." }

    cargo test --manifest-path src-tauri/Cargo.toml
    if ($LASTEXITCODE -ne 0) { throw "Rust tests failed with exit code $LASTEXITCODE." }

    cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings -A clippy::manual-pattern-char-comparison
    if ($LASTEXITCODE -ne 0) { throw "Rust lint failed with exit code $LASTEXITCODE." }
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

    $tauriCli = Join-Path $repoRoot "node_modules\.bin\tauri.cmd"
    if (-not (Test-Path $tauriCli)) {
        throw "The local Tauri CLI was not found at $tauriCli. Delete node_modules and run npm install, then retry."
    }

    Write-Host "Building Tauri desktop bundle..."
    & $tauriCli build --config $layout.ConfigPath
    if ($LASTEXITCODE -ne 0) { throw "Tauri desktop build failed with exit code $LASTEXITCODE." }
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
    externalRoot = $layout.ExternalRoot
    buildRoot = $layout.BuildRoot
    frontendDist = $layout.FrontendDist
    cargoTarget = $layout.CargoTarget
    bundled = $shouldBundle
    gitCommit = $gitCommit
}
$manifest | ConvertTo-Json -Depth 5 | Set-Content -Encoding UTF8 (Join-Path $layout.BuildRoot "build-manifest.json")

& (Join-Path $PSScriptRoot "assert-repo-clean.ps1")
Write-Host "WorkLore $Channel build completed outside the repository."

if ($shouldBundle) {
    $appExecutable = Join-Path $layout.CargoTarget "release\worklore.exe"
    $installerRoot = Join-Path $layout.CargoTarget "release\bundle\nsis"
    if (Test-Path $appExecutable) {
        Write-Host "Application executable: $appExecutable"
    }
    if (Test-Path $installerRoot) {
        Get-ChildItem -Path $installerRoot -Filter "*.exe" -File | ForEach-Object {
            Write-Host "Windows installer: $($_.FullName)"
        }
    }
}
