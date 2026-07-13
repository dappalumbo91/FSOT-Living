# Build trinary genetic body kernel + boot image + launch QEMU.
# Prereqs: rustup nightly, target x86_64-unknown-none, qemu-system-x86_64 in PATH.
# Optional: $env:OVMF_PATH = "C:\path\to\OVMF_CODE.fd"

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

Write-Host "=== FSOT Trinary Body — QEMU build ===" -ForegroundColor Cyan
Write-Host "Root: $Root"

# Host proof first (fast fail if fabric is wrong)
Write-Host "`n[1/3] host_probe + unit tests" -ForegroundColor Yellow
cargo test -p trinary -p genetic
cargo run -p host_probe
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Kernel for bare metal
Write-Host "`n[2/3] kernel x86_64-unknown-none" -ForegroundColor Yellow
rustup target add x86_64-unknown-none 2>$null
cargo build -p fsot_body_kernel --target x86_64-unknown-none
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$Kernel = Join-Path $Root "target\x86_64-unknown-none\debug\fsot_body_kernel"
if (-not (Test-Path $Kernel)) {
    # MSVC host may still produce ELF with no extension
    $alt = Get-ChildItem (Join-Path $Root "target\x86_64-unknown-none\debug") -Filter "fsot_body_kernel*" -ErrorAction SilentlyContinue |
        Where-Object { -not $_.PSIsContainer -and $_.Extension -ne ".d" -and $_.Name -notlike "*.pdb" } |
        Select-Object -First 1
    if ($alt) { $Kernel = $alt.FullName }
}
if (-not (Test-Path $Kernel)) {
    Write-Error "Kernel ELF not found under target\x86_64-unknown-none\debug"
    exit 1
}
Write-Host "KERNEL_ELF=$Kernel"
$env:KERNEL_ELF = $Kernel
$env:OUT_DIR = Join-Path $Root "target\body_image"

# Image + QEMU
Write-Host "`n[3/3] boot image + QEMU" -ForegroundColor Yellow
$qemu = Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue
if (-not $qemu) {
    Write-Host "qemu-system-x86_64 not in PATH — building image only" -ForegroundColor DarkYellow
    cargo run -p fsot_body_boot
    exit $LASTEXITCODE
}

cargo run -p fsot_body_boot -- --run
exit $LASTEXITCODE
