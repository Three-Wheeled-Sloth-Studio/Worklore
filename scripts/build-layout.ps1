Set-StrictMode -Version Latest

function Get-WorkLoreRepoRoot {
    return [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
}

function Get-NormalizedPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $expanded = [Environment]::ExpandEnvironmentVariables($Path)
    return [System.IO.Path]::GetFullPath($expanded).TrimEnd('\', '/')
}

function Test-PathInside {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Candidate,
        [Parameter(Mandatory = $true)]
        [string]$Parent
    )

    $candidatePath = Get-NormalizedPath $Candidate
    $parentPath = Get-NormalizedPath $Parent
    $comparison = [System.StringComparison]::OrdinalIgnoreCase

    if ($candidatePath.Equals($parentPath, $comparison)) {
        return $true
    }

    return $candidatePath.StartsWith($parentPath + [System.IO.Path]::DirectorySeparatorChar, $comparison)
}

function Assert-OutsideWorkLoreRepo {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,
        [string]$Purpose = "Build output"
    )

    $repoRoot = Get-WorkLoreRepoRoot
    if (Test-PathInside -Candidate $Path -Parent $repoRoot) {
        throw "$Purpose must be outside the repository. Refusing path: $Path"
    }
}

function Get-WorkLoreExternalRoot {
    if ($env:WORKLORE_EXTERNAL_ROOT) {
        $root = Get-NormalizedPath $env:WORKLORE_EXTERNAL_ROOT
    }
    elseif ($env:LOCALAPPDATA) {
        $root = Get-NormalizedPath (Join-Path $env:LOCALAPPDATA "WorkLore")
    }
    else {
        $repoParent = Split-Path -Parent (Get-WorkLoreRepoRoot)
        $root = Get-NormalizedPath (Join-Path $repoParent "WorkLoreExternal")
    }

    Assert-OutsideWorkLoreRepo -Path $root -Purpose "WorkLore external root"
    return $root
}

function Get-WorkLoreBuildLayout {
    param(
        [ValidateSet("dev", "validate", "qa", "release")]
        [string]$Channel = "qa"
    )

    $externalRoot = Get-WorkLoreExternalRoot
    $buildRoot = if ($env:WORKLORE_BUILD_ROOT) {
        Get-NormalizedPath $env:WORKLORE_BUILD_ROOT
    }
    else {
        Get-NormalizedPath (Join-Path $externalRoot "build\$Channel")
    }

    $installRoot = if ($Channel -eq "qa") {
        Get-NormalizedPath (Join-Path $externalRoot "installed\qa")
    }
    elseif ($Channel -eq "release") {
        Get-NormalizedPath (Join-Path $externalRoot "installed\release")
    }
    else {
        Get-NormalizedPath (Join-Path $externalRoot "installed\$Channel")
    }

    Assert-OutsideWorkLoreRepo -Path $buildRoot -Purpose "$Channel build root"
    Assert-OutsideWorkLoreRepo -Path $installRoot -Purpose "$Channel install root"

    return [pscustomobject]@{
        Channel = $Channel
        ExternalRoot = $externalRoot
        BuildRoot = $buildRoot
        FrontendDist = Get-NormalizedPath (Join-Path $buildRoot "frontend")
        CargoTarget = Get-NormalizedPath (Join-Path $buildRoot "cargo-target")
        ConfigPath = Get-NormalizedPath (Join-Path $buildRoot "tauri.$Channel.conf.json")
        LogsRoot = Get-NormalizedPath (Join-Path $buildRoot "logs")
        InstallRoot = $installRoot
    }
}

function Initialize-WorkLoreBuildLayout {
    param(
        [Parameter(Mandatory = $true)]
        $Layout,
        [switch]$Clean
    )

    if ($Clean -and (Test-Path $Layout.BuildRoot)) {
        Remove-Item -Recurse -Force $Layout.BuildRoot
    }

    New-Item -ItemType Directory -Force -Path $Layout.BuildRoot | Out-Null
    New-Item -ItemType Directory -Force -Path $Layout.FrontendDist | Out-Null
    New-Item -ItemType Directory -Force -Path $Layout.CargoTarget | Out-Null
    New-Item -ItemType Directory -Force -Path $Layout.LogsRoot | Out-Null
}
