@echo off
setlocal
cd /d "%~dp0"

call verify-clip.bat
if errorlevel 1 (
  echo [pi-clipper] Not launching due to test failures.
  exit /b 1
)

call run.bat
