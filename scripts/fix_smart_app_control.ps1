# Fix Smart App Control blocking local Rust/cargo builds (fsot-probe, etc.)
# Run elevated (Right-click -> Run with PowerShell as Administrator)
# Root cause: Windows Smart App Control (Code Integrity) blocks unsigned local EXEs
# after it flips to enforcement — cargo rebuilds are unsigned by design.

#Requires -RunAsAdministrator
$ErrorActionPreference = "Stop"
$path = "HKLM:\SYSTEM\CurrentControlSet\Control\CI\Policy"

$before = Get-ItemProperty $path -ErrorAction SilentlyContinue
Write-Host "BEFORE VerifiedAndReputablePolicyState=$($before.VerifiedAndReputablePolicyState) (0=Off 1=Eval 2=On)"

# 0 = Off — required for local development of unsigned cargo binaries
Set-ItemProperty -Path $path -Name "VerifiedAndReputablePolicyState" -Value 0 -Type DWord -Force
Remove-ItemProperty -Path $path -Name "SAC_EnforcementReason" -ErrorAction SilentlyContinue
Remove-ItemProperty -Path $path -Name "SAC_PreviousState" -ErrorAction SilentlyContinue

$after = Get-ItemProperty $path
Write-Host "AFTER  VerifiedAndReputablePolicyState=$($after.VerifiedAndReputablePolicyState)"

# Refresh Code Integrity policies (may still need reboot if blocks persist)
$ci = Join-Path $env:SystemRoot "System32\CiTool.exe"
if (Test-Path $ci) {
    & $ci -r 2>$null
    Write-Host "CiTool policy refresh requested"
}

Restart-Service AppIDSvc -Force -ErrorAction SilentlyContinue

# Optional: Defender exclusion for cargo target (not a substitute for SAC Off)
$build = "C:\Users\damia\Desktop\living fsot\files-e5887462\fsot-living-rust\target"
try {
    $ex = @( (Get-MpPreference).ExclusionPath )
    if ($ex -notcontains $build) {
        Add-MpPreference -ExclusionPath $build
        Write-Host "Defender exclusion: $build"
    }
} catch {
    Write-Host "Defender exclusion skipped: $_"
}

Write-Host ""
Write-Host "If binaries still block, REBOOT once so user-mode CI drops SAC enforcement."
Write-Host "UI path: Windows Security > App & browser control > Smart App Control > Off"
Write-Host "DONE"
