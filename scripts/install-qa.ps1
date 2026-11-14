param(
    [switch]$Launch,
    [switch]$SkipBuild,
    [switch]$CleanBuild
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot "build-layout.ps1")

$repoRoot = Get-WorkLoreRepoRoot
Set-Location $repoRoot
& (Join-Path $PSScriptRoot "assert-repo-clean.ps1")

$layout = Get-WorkLoreBuildLayout -Channel "qa"
Assert-OutsideWorkLoreRepo -Path $layout.InstallRoot -Purpose "QA installation"

if (-not $SkipBuild) {
    & (Join-Path $PSScriptRoot "build.ps1") -Channel "qa" -Clean:$CleanBuild
}

$builtExecutable = Join-Path $layout.CargoTarget "release\worklore.exe"
if (-not (Test-Path $builtExecutable)) {
    throw "The QA executable was not found at $builtExecutable. Run the QA build first."
}

# The QA install folder is also the portable app location. WorkLore vaults may live beside
# the executable by explicit product design, so never delete the install root during refresh.
# Replace only files owned by the QA installer and preserve user-created folders and vaults.
New-Item -ItemType Directory -Force -Path $layout.InstallRoot | Out-Null

$installedExecutable = Join-Path $layout.InstallRoot "WorkLore-QA.exe"
Copy-Item -Force $builtExecutable $installedExecutable

$licensePath = Join-Path $repoRoot "LICENSE"
if (Test-Path $licensePath) {
    Copy-Item -Force $licensePath (Join-Path $layout.InstallRoot "LICENSE")
}

$buildManifest = Join-Path $layout.BuildRoot "build-manifest.json"
if (Test-Path $buildManifest) {
    Copy-Item -Force $buildManifest (Join-Path $layout.InstallRoot "build-manifest.json")
}

$gitCommit = $null
try {
    $gitCommit = (git rev-parse HEAD).Trim()
}
catch {
    $gitCommit = $null
}

$installManifest = [ordered]@{
    schemaVersion = 1
    channel = "qa"
    installedAt = (Get-Date).ToUniversalTime().ToString("o")
    installRoot = $layout.InstallRoot
    executable = $installedExecutable
    sourceBuildRoot = $layout.BuildRoot
    repository = $repoRoot
    gitCommit = $gitCommit
}
$installManifest | ConvertTo-Json -Depth 5 | Set-Content -Encoding UTF8 (Join-Path $layout.InstallRoot "install-manifest.json")

& (Join-Path $PSScriptRoot "assert-repo-clean.ps1")
Write-Host "WorkLore QA installed outside the repository: $installedExecutable"
Write-Host "Existing vaults and user-created folders under the QA install location were preserved."

if ($Launch) {
    $env:WORKLORE_RUN_CHANNEL = "qa"
    Start-Process -FilePath $installedExecutable -WorkingDirectory $layout.InstallRoot
}
