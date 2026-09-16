param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

if (-not $env:LOCALAPPDATA) {
    throw "LOCALAPPDATA is unavailable, so WorkLore cannot locate the legacy build folder."
}

$legacyBuildRoot = Join-Path $env:LOCALAPPDATA "WorkLore\build"
if (-not (Test-Path $legacyBuildRoot)) {
    Write-Host "No legacy WorkLore build folder was found at: $legacyBuildRoot"
    exit 0
}

Write-Host "Legacy WorkLore build folder: $legacyBuildRoot"
Write-Host "This removes generated frontend, Cargo, installer, and log output only."
Write-Host "Vaults, preferences, and imported career files are not stored under this folder."

if (-not $Force) {
    $answer = Read-Host "Delete the legacy build folder? Type YES to continue"
    if ($answer -cne "YES") {
        Write-Host "Cleanup cancelled."
        exit 0
    }
}

Remove-Item -LiteralPath $legacyBuildRoot -Recurse -Force
Write-Host "Legacy WorkLore build folder removed."
