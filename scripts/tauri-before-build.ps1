$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot "build-layout.ps1")

$repoRoot = Get-WorkLoreRepoRoot
Set-Location $repoRoot

$frontendDist = if ($env:WORKLORE_FRONTEND_DIST) {
    Get-NormalizedPath $env:WORKLORE_FRONTEND_DIST
}
else {
    Get-NormalizedPath (Join-Path (Split-Path -Parent $repoRoot) "WorkLoreBuild\frontend")
}
Assert-OutsideWorkLoreRepo -Path $frontendDist -Purpose "Frontend build output"
New-Item -ItemType Directory -Force -Path $frontendDist | Out-Null

$env:WORKLORE_FRONTEND_DIST = $frontendDist
npm run build:frontend

& (Join-Path $PSScriptRoot "assert-repo-clean.ps1")
