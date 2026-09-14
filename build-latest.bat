@echo off
setlocal

cd /d "%~dp0"

where git >nul 2>&1
if errorlevel 1 (
    echo ERROR: git was not found on PATH.
    exit /b 1
)

where npm >nul 2>&1
if errorlevel 1 (
    echo ERROR: npm was not found on PATH.
    exit /b 1
)

for /f "delims=" %%I in ('git status --porcelain 2^>nul') do set "WORKLORE_DIRTY=1"
if defined WORKLORE_DIRTY (
    echo ERROR: The WorkLore checkout has uncommitted changes.
    echo Commit, stash, or discard them before building the latest dev branch.
    exit /b 1
)

echo.
echo [1/3] Switching to dev...
git switch dev
if errorlevel 1 goto :failed

echo.
echo [2/3] Pulling latest origin/dev...
git pull --ff-only origin dev
if errorlevel 1 goto :failed

echo.
echo [3/3] Building the WorkLore desktop application...
call npm run desktop:build
if errorlevel 1 goto :failed

echo.
echo WorkLore desktop build completed successfully.
exit /b 0

:failed
echo.
echo ERROR: Update or build failed. Review the output above for details.
exit /b 1
