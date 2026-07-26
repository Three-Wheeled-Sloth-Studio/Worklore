$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot "build-layout.ps1")

$repoRoot = Get-WorkLoreRepoRoot
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
}

$qa = Get-WorkLoreBuildLayout -Channel "qa"
$release = Get-WorkLoreBuildLayout -Channel "release"
if ((Get-NormalizedPath $qa.InstallRoot) -eq (Get-NormalizedPath $release.InstallRoot)) {
    throw "QA and release installations must use different folders."
}

Write-Host "External build layout validation passed for: $($channels -join ', ')"
