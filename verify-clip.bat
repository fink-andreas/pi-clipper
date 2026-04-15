@echo off
setlocal
cd /d "%~dp0"

echo [pi-clipper] Running clipboard regression tests...
cd src-tauri
cargo test --test test_clipboard_integration --test test_rules
if errorlevel 1 (
  echo [pi-clipper] Clipboard regression tests failed.
  exit /b 1
)

echo [pi-clipper] Clipboard regression tests passed.
