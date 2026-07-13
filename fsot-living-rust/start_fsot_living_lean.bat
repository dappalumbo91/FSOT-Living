@echo off
REM 2012 Mac Mini / 4GB-class lean launch
set FSOT_LEAN=1
set FSOT_MIN_SPEC=1
REM Body TCP (default). Use FSOT_BODY_TCP=off for files only.
if not defined FSOT_BODY_TCP set FSOT_BODY_TCP=127.0.0.1:7368
cd /d "%~dp0"
echo FSOT Living (LEAN min-spec) — body TCP=%FSOT_BODY_TCP%
echo Start body first:  fsot-trinary-body\scripts\run_tcp_body.ps1
cargo run --release
if errorlevel 1 pause
