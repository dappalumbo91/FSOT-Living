# Closed loop body host — prefers real-time TCP; files still written.
# Terminal A: this script
# Terminal B: living TUI (start_fsot_living.bat or start_fsot_living_lean.bat)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Living = Join-Path (Split-Path -Parent $Root) "fsot-living-rust"
$State = Join-Path (Split-Path -Parent $Root) "state\habitat-rust"

Set-Location $Root
Write-Host "=== FSOT closed loop (TCP body host) ===" -ForegroundColor Cyan
Write-Host "State dir: $State"
Write-Host "Living:    $Living"
Write-Host ""
Write-Host "In another terminal, run living cortex:"
Write-Host "  cd `"$Living`""
Write-Host "  .\start_fsot_living_lean.bat   # 2012 Mini / 4GB"
Write-Host "  # or .\start_fsot_living.bat"
Write-Host "  Ask: body status"
Write-Host ""
Write-Host "TCP vs files: docs\TRANSPORT_TCP_VS_FILES.md" -ForegroundColor DarkCyan
Write-Host ""

$env:FSOT_BRIDGE_DIR = $State
# Default: infinite TCP. Pass a number for finite run: .\run_closed_loop.ps1 80
if ($args.Count -ge 1 -and $args[0] -match '^\d+$') {
    cargo run --release -p mind_bridge -- --tcp --ticks $args[0]
} else {
    cargo run --release -p mind_bridge -- --tcp
}
