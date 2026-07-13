@echo off
cd /d "%~dp0"
echo FSOT Desktop Visual — http://127.0.0.1:3850
echo Start living TUI and mind_bridge --tcp in other terminals for full feed.
python tools\desktop_viz.py
if errorlevel 1 pause
