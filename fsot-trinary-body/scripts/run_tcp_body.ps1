# Real-time body host (TCP Mind ABI on 127.0.0.1:7368).
# Run living TUI in another terminal — it auto-connects.
#
# Ctrl+C to stop.

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$State = Join-Path (Split-Path -Parent $Root) "state\habitat-rust"
$env:FSOT_BRIDGE_DIR = $State
if ($env:FSOT_BODY_PORT) { $port = $env:FSOT_BODY_PORT } else { $port = "7368" }

Write-Host "=== FSOT body host — TCP real-time ===" -ForegroundColor Cyan
Write-Host "listen  127.0.0.1:$port"
Write-Host "files   $State"
Write-Host "cortex: start fsot-living-rust (auto TCP); ask: body status"
Write-Host ""

cargo run --release -p mind_bridge -- --tcp $port
