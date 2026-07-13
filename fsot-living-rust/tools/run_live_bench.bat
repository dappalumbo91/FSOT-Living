@echo off
title FSOT Live Bench
cd /d "%~dp0\.."
echo Waiting for live TUI (LIVE_STATUS.json must be fresh)...
echo State: ..\state\habitat-rust\
echo.
python tools\live_bench.py %*
echo.
pause
