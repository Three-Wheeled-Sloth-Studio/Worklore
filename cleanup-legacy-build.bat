@echo off
setlocal
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\cleanup-legacy-build.ps1"
if errorlevel 1 pause
