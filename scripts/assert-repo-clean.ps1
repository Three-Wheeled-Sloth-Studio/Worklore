$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot "build-layout.ps1")

$repoRoot = Get-WorkLoreRepoRoot
$forbiddenPaths = @(
    "dist",
    "build",
    "target",
    "src-tauri\target",
    "qa-install",
    "release",
    "artifacts"
)

$violations = @()
foreach ($relativePath in $forbiddenPaths) {
    $candidate = Join-Path $repoRoot $relativePath
    if (Test-Path $candidate) {
        $violations += $candidate
    }
}

if ($violations.Count -gt 0) {
    $formatted = $violations | ForEach-Object { " - $_" }
    throw "Build or install output exists inside the repository:`n$($formatted -join [Environment]::NewLine)`nMove or delete it before continuing."
}

Write-Host "Repository/build separation check passed."
