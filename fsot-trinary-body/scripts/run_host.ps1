# Host-only path: full body stack without QEMU.
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root
cargo test -p trinary -p genetic -p body -- --test-threads=1
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo run -p host_probe
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo run -p mind_bridge -- 20
exit $LASTEXITCODE
