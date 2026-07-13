# FSOT Autonomous Habitat launcher (survives crashes, reopens browser)
$ErrorActionPreference = "Continue"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $Root
$Port = 3847
# Prefer 127.0.0.1 — avoids IPv6/localhost quirks on Windows
$Url = "http://127.0.0.1:$Port"

function Stop-Port {
  Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue |
    ForEach-Object {
      Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue
    }
}

Write-Host "FSOT Habitat launcher"
Write-Host "  Root: $Root"
Write-Host "  URL:  $Url"
Stop-Port
Start-Sleep -Seconds 1

# Open browser once habitat answers
$job = Start-Job -ScriptBlock {
  param($u)
  for ($i = 0; $i -lt 40; $i++) {
    Start-Sleep -Seconds 1
    try {
      $null = Invoke-WebRequest -Uri "$u/api/state" -UseBasicParsing -TimeoutSec 2
      Start-Process $u
      return
    } catch { }
  }
} -ArgumentList $Url

# Restart loop
while ($true) {
  Write-Host "`n[$(Get-Date -Format o)] Starting habitat..."
  $p = Start-Process -FilePath "node" -ArgumentList "fsot_viz_server.js" `
    -WorkingDirectory $Root -PassThru -NoNewWindow -Wait
  $code = $p.ExitCode
  Write-Host "[$(Get-Date -Format o)] Habitat exited code=$code — restarting in 2s"
  Add-Content -Path (Join-Path $Root "state\habitat\crash.log") `
    -Value "[$(Get-Date -Format o)] process exit code=$code — auto-restart"
  Start-Sleep -Seconds 2
  Stop-Port
  Start-Sleep -Seconds 1
}
