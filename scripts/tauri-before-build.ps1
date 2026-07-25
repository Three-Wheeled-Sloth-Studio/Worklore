$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot "build-layout.ps1")

$repoRoot = Get-WorkLoreRepoRoot
Set-Location $repoRoot

$layout = Get-WorkLoreBuildLayout -Channel $(if ($env:WORKLORE_BUILD_CHANNEL) { $env:WORKLORE_BUILD_CHANNEL } else { "qa" })
Initialize-WorkLoreBuildLayout -Layout $layout

$frontendDist = if ($env:WORKLORE_FRONTEND_DIST) {
    Get-NormalizedPath $env:WORKLORE_FRONTEND_DIST
}
else {
    $layout.FrontendDist
}
Assert-OutsideWorkLoreRepo -Path $frontendDist -Purpose "Frontend build output"

$env:WORKLORE_FRONTEND_DIST = $frontendDist
npm run build:frontend

& (Join-Path $PSScriptRoot "assert-repo-clean.ps1")
