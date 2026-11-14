$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot "build-layout.ps1")

$repoRoot = Get-WorkLoreRepoRoot
$tauriProjectRoot = Get-NormalizedPath (Join-Path $repoRoot "src-tauri")
$channels = @("dev", "validate", "qa", "release")

foreach ($channel in $channels) {
    $layout = Get-WorkLoreBuildLayout -Channel $channel
    Assert-OutsideWorkLoreRepo -Path $layout.BuildRoot -Purpose "$channel build root"
    Assert-OutsideWorkLoreRepo -Path $layout.FrontendDist -Purpose "$channel frontend output"
    Assert-OutsideWorkLoreRepo -Path $layout.CargoTarget -Purpose "$channel Cargo target"
    Assert-OutsideWorkLoreRepo -Path $layout.InstallRoot -Purpose "$channel installation"

    if (Test-PathInside -Candidate $layout.InstallRoot -Parent $layout.BuildRoot) {
        throw "$channel install root must not be inside its build root."
    }

    $tauriFrontendDist = Get-WorkLoreTauriFrontendDist -Layout $layout
    if ([System.IO.Path]::IsPathRooted($tauriFrontendDist) -or $tauriFrontendDist -match '^[A-Za-z]:') {
        throw "$channel Tauri frontendDist must be relative, got: $tauriFrontendDist"
    }

    $resolvedFrontendDist = Get-NormalizedPath (Join-Path $tauriProjectRoot $tauriFrontendDist)
    if ($resolvedFrontendDist -ne (Get-NormalizedPath $layout.FrontendDist)) {
        throw "$channel Tauri frontendDist resolves to the wrong directory. Expected $($layout.FrontendDist), got $resolvedFrontendDist"
    }
}

$qa = Get-WorkLoreBuildLayout -Channel "qa"
$release = Get-WorkLoreBuildLayout -Channel "release"
if ((Get-NormalizedPath $qa.InstallRoot) -eq (Get-NormalizedPath $release.InstallRoot)) {
    throw "QA and release installations must use different folders."
}

Write-Host "External build layout validation passed for: $($channels -join ', ')"
Write-Host "Tauri frontend paths are relative and resolve to the configured external frontend outputs."
