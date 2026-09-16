@echo off
setlocal

cd /d "%~dp0"

where git >nul 2>&1
if errorlevel 1 (
    echo ERROR: git was not found on PATH.
    exit /b 1
)

where powershell >nul 2>&1
if errorlevel 1 (
    echo ERROR: Windows PowerShell was not found on PATH.
    exit /b 1
)

for /f "delims=" %%I in ('git status --porcelain 2^>nul') do set "WORKLORE_DIRTY=1"
if defined WORKLORE_DIRTY (
    echo ERROR: The WorkLore checkout has uncommitted changes.
    echo.
    git status --short --untracked-files=all
    echo.
    echo Commit, stash, or discard them before building the latest dev branch.
    exit /b 1
)

echo.
echo [1/4] Switching to dev...
git switch dev
if errorlevel 1 goto :failed

echo.
echo [2/4] Pulling latest origin/dev...
git pull --ff-only origin dev
if errorlevel 1 goto :failed

echo.
echo [3/4] Building the WorkLore desktop executable...
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build.ps1 -Channel qa -SkipTests
if errorlevel 1 goto :failed

echo.
echo [4/4] Staging the fresh QA executable...
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\install-qa.ps1 -SkipBuild
if errorlevel 1 goto :failed

echo.
echo WorkLore desktop build completed successfully.
echo Launch the staged QA app from the installed\qa folder printed above.
exit /b 0

:failed
echo.
echo ERROR: Update or build failed. Review the first error above for the actual cause.
exit /b 1
