# Build UEFI body image and launch QEMU with interactive serial.
# Type MOTOR commands directly in this console, e.g.:
#   MOTOR STATUS
#   MOTOR BREATHE
#   MOTOR EXPRESS
#   MOTOR ORGANS
#   MOTOR READ /body/genome
#
# Optional: $env:OVMF_PATH = "C:\path\to\OVMF_CODE.fd"
# Ctrl+A then X to quit QEMU (if mon:stdio); or close window.

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

Write-Host "=== FSOT Body — interactive QEMU serial ===" -ForegroundColor Cyan

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

Write-Host "Image: $Img"
Write-Host "Serial is interactive — type MOTOR lines, watch MIND frames" -ForegroundColor Green

$args = @(
    "-machine", "q35",
    "-m", "256M",
    "-serial", "mon:stdio",
    "-display", "none",
    "-no-reboot",
    "-drive", "format=raw,file=$Img"
)
if ($ovmf) {
    Write-Host "OVMF: $ovmf"
    $args += @("-drive", "if=pflash,format=raw,readonly=on,file=$ovmf")
} else {
    Write-Host "WARNING: no OVMF found — UEFI boot may fail. Set OVMF_PATH." -ForegroundColor Yellow
}

& qemu-system-x86_64 @args
exit $LASTEXITCODE
