@echo off
setlocal
cd /d "%~dp0"

echo [pi-clipper] Stopping existing process (if running)...
taskkill /IM pi-clipper.exe /F >nul 2>&1

echo [pi-clipper] Building fresh release binary...
cd src-tauri
cargo build --release
if errorlevel 1 (
  echo [pi-clipper] Build failed. Not launching.
  exit /b 1
)
cd ..

echo [pi-clipper] Launching src-tauri\target\release\pi-clipper.exe
start "" "src-tauri\target\release\pi-clipper.exe"
