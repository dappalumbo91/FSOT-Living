@echo off
title FSOT Living - Rust TUI + Desktop Visual
cd /d "%~dp0"

set EXE=target\release\fsot-living.exe
set VIZ_PORT=3850
set VIZ_LOG=..\state\habitat-rust\monitor_viz.log
set VIZ_ERR=..\state\habitat-rust\monitor_viz_err.log

if not exist "%EXE%" (
  echo Building release binary first...
  cargo build --release --bin fsot-living
  if errorlevel 1 (
    echo BUILD FAILED
    pause
    exit /b 1
  )
)

if not exist "..\state\habitat-rust" mkdir "..\state\habitat-rust"

REM --- Auto-boot desktop visual with keep-alive (self-restart inside python) ---
if /I not "%FSOT_VIZ_AUTO%"=="off" (
  echo Starting desktop visual on http://127.0.0.1:%VIZ_PORT% ...
  REM Kill stale viz on same port if any (best-effort)
  for /f "tokens=5" %%P in ('netstat -ano 2^>nul ^| findstr ":%VIZ_PORT% " ^| findstr LISTENING') do (
    taskkill /PID %%P /F >nul 2>&1
  )
  start "FSOT Desktop Viz" /MIN cmd /c "python tools\desktop_viz.py >> "%VIZ_LOG%" 2>> "%VIZ_ERR%""
  timeout /t 1 /nobreak >nul
  start "" "http://127.0.0.1:%VIZ_PORT%/"
)

echo.
echo Starting FSOT Living TUI...
echo   Speak in TUI bottom + Enter  OR  chat box in the browser visual
echo   Esc=quit TUI
echo   Body: run mind_bridge --tcp in another terminal for chassis link
echo   Binary: %EXE%
echo   Viz health: http://127.0.0.1:%VIZ_PORT%/api/health
echo.

"%EXE%"
set ERR=%ERRORLEVEL%

echo.
if not "%ERR%"=="0" (
  echo Program exited with code %ERR%
) else (
  echo Program exited normally.
)
echo.
pause
