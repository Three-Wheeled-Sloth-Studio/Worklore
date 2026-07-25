@echo off
setlocal
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\install-qa.ps1" -Launch
if errorlevel 1 pause
