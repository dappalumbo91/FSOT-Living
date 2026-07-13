# FSOT OS nerve — QEMU body with serial exposed as TCP chardev.
#
# Architecture:
#   living cortex  --TCP:7370-->  QEMU -serial chardev  -->  bare-metal soma (kernel)
#   same Mind ABI lines as host mind_bridge (MOTOR / MIND)
#
# Usage:
#   Terminal A:  .\scripts\run_qemu_serial_tcp.ps1
#   Terminal B:  cd ..\fsot-living-rust
#                $env:FSOT_BODY_QEMU_TCP = "127.0.0.1:7370"
#                .\start_fsot_living.bat
#
# Optional:
#   $env:FSOT_QEMU_SERIAL_PORT = "7370"
#   $env:OVMF_PATH = "C:\path\to\OVMF_CODE.fd"
#   $env:FSOT_QEMU_MEM = "256M"

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

$Port = if ($env:FSOT_QEMU_SERIAL_PORT) { $env:FSOT_QEMU_SERIAL_PORT } else { "7370" }
$Mem = if ($env:FSOT_QEMU_MEM) { $env:FSOT_QEMU_MEM } else { "256M" }

Write-Host "=== FSOT Body — QEMU serial TCP nerve :$Port ===" -ForegroundColor Cyan

cargo build -p fsot_body_kernel --target x86_64-unknown-none
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$Kernel = Join-Path $Root "target\x86_64-unknown-none\debug\fsot_body_kernel"
if (-not (Test-Path $Kernel)) {
    Write-Error "kernel not found: $Kernel"
    exit 1
}

$env:KERNEL_ELF = $Kernel
$env:OUT_DIR = Join-Path $Root "target\body_image"
cargo run -p fsot_body_boot
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$Img = Join-Path $Root "target\body_image\fsot_body_uefi.img"
if (-not (Test-Path $Img)) {
    Write-Error "UEFI image missing: $Img"
    exit 1
}

$qemu = Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue
if (-not $qemu) {
    Write-Host "qemu-system-x86_64 not in PATH. Image ready at $Img" -ForegroundColor Yellow
    Write-Host "Install QEMU or add to PATH, then re-run." -ForegroundColor Yellow
    exit 0
}

$ovmf = $env:OVMF_PATH
if (-not $ovmf) {
    foreach ($c in @(
        "C:\Program Files\qemu\share\edk2-x86_64-code.fd",
        "C:\Program Files\qemu\share\OVMF.fd",
        "C:\Program Files\qemu\share\edk2-i386-code.fd"
    )) {
        if (Test-Path $c) { $ovmf = $c; break }
    }
}

Write-Host "Image:  $Img"
Write-Host "Nerve:  TCP 127.0.0.1:$Port  (living: FSOT_BODY_QEMU_TCP=127.0.0.1:$Port)"
Write-Host "Protocol: MOTOR / MIND lines (same ABI as mind_bridge)" -ForegroundColor Green

# chardev socket server — cortex connects as client
$args = @(
    "-machine", "q35",
    "-m", $Mem,
    "-display", "none",
    "-no-reboot",
    "-chardev", "socket,id=fsotser,host=127.0.0.1,port=$Port,server=on,wait=off",
    "-serial", "chardev:fsotser",
    "-drive", "format=raw,file=$Img"
)
if ($ovmf) {
    Write-Host "OVMF: $ovmf"
    $args += @("-drive", "if=pflash,format=raw,readonly=on,file=$ovmf")
} else {
    Write-Host "WARNING: no OVMF found — UEFI boot may fail. Set OVMF_PATH." -ForegroundColor Yellow
}

Write-Host "Launching QEMU (Ctrl+C to stop)..."
& qemu-system-x86_64 @args
exit $LASTEXITCODE
