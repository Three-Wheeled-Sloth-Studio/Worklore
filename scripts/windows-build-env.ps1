Set-StrictMode -Version Latest

function Test-WorkLoreMsvcRuntimeLibrary {
    if (-not $env:LIB) {
        return $false
    }

    foreach ($entry in ($env:LIB -split ';')) {
        if ([string]::IsNullOrWhiteSpace($entry)) {
            continue
        }

        $candidate = Join-Path $entry.Trim() "msvcrt.lib"
        if (Test-Path $candidate) {
            return $true
        }
    }

    return $false
}

function Import-WorkLoreVisualStudioEnvironment {
    if ($env:OS -ne "Windows_NT") {
        return
    }

    if ((Get-Command link.exe -ErrorAction SilentlyContinue) -and (Test-WorkLoreMsvcRuntimeLibrary)) {
        return
    }

    $vswhereCandidates = @()
    if (${env:ProgramFiles(x86)}) {
        $vswhereCandidates += Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    }
    if ($env:ProgramFiles) {
        $vswhereCandidates += Join-Path $env:ProgramFiles "Microsoft Visual Studio\Installer\vswhere.exe"
    }

    $vswhere = $vswhereCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
    if (-not $vswhere) {
        throw "Visual Studio Installer's vswhere.exe was not found. Install Visual Studio or Build Tools with the 'Desktop development with C++' workload and a Windows 10/11 SDK."
    }

    $installationPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath | Select-Object -First 1
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($installationPath)) {
        throw "A Visual Studio installation with the C++ x64/x86 build tools was not found. In Visual Studio Installer, add 'Desktop development with C++' and a Windows 10/11 SDK."
    }

    $devCmd = Join-Path $installationPath.Trim() "Common7\Tools\VsDevCmd.bat"
    if (-not (Test-Path $devCmd)) {
        throw "Visual Studio developer environment script was not found at: $devCmd"
    }

    Write-Host "Initializing Visual Studio x64 build environment..."
    $tempCmd = Join-Path ([System.IO.Path]::GetTempPath()) "worklore-vsenv-$PID.cmd"
    try {
        @(
            "@echo off",
            "call `"$devCmd`" -arch=x64 -host_arch=x64 >nul 2>&1",
            "if errorlevel 1 exit /b %errorlevel%",
            "set"
        ) | Set-Content -Encoding ASCII $tempCmd

        $environmentLines = & $env:ComSpec /d /c $tempCmd
        $devCmdExitCode = $LASTEXITCODE
        if ($devCmdExitCode -ne 0) {
            throw "Visual Studio developer environment initialization failed with exit code $devCmdExitCode."
        }

        foreach ($line in $environmentLines) {
            $separator = $line.IndexOf('=')
            if ($separator -le 0) {
                continue
            }

            $name = $line.Substring(0, $separator)
            $value = $line.Substring($separator + 1)
            [System.Environment]::SetEnvironmentVariable(
                $name,
                $value,
                [System.EnvironmentVariableTarget]::Process
            )
        }
    }
    finally {
        Remove-Item -Force -ErrorAction SilentlyContinue $tempCmd
    }

    if (-not (Get-Command link.exe -ErrorAction SilentlyContinue)) {
        throw "MSVC link.exe is still unavailable after initializing Visual Studio. Repair or modify the Visual Studio C++ workload."
    }

    if (-not (Test-WorkLoreMsvcRuntimeLibrary)) {
        throw "The Windows SDK runtime library msvcrt.lib is unavailable after initializing Visual Studio. In Visual Studio Installer, modify the installation and include 'Desktop development with C++' plus a Windows 10/11 SDK."
    }

    Write-Host "Visual Studio native build environment ready."
}
