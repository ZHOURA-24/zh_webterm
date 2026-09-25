$ErrorActionPreference = "Stop"

$repo = "ZHOURA-24/zh_webterm"
$binDir = "$env:LOCALAPPDATA\Programs\zh_webterm"
$exePath = "$binDir\zh_webterm.exe"
$url = "https://github.com/$repo/releases/latest/download/zh_webterm-windows-amd64.exe"

# Stop existing running instance if any
$procs = Get-Process -Name "zh_webterm" -ErrorAction SilentlyContinue
if ($procs) {
    $procs | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 500
}

Write-Host "Downloading zh_webterm binary..." -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
Invoke-WebRequest -Uri $url -OutFile $exePath

# Add to user PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$binDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$binDir", "User")
    $env:Path += ";$binDir"
}

# Create secure zh_webterm.env if not exists
$envFile = "$binDir\zh_webterm.env"
$isNewInstall = $false
if (-not (Test-Path $envFile)) {
    $bytes = New-Object byte[] 12
    [System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($bytes)
    $genPassword = [Convert]::ToBase64String($bytes).Replace('+', '').Replace('/', '').Replace('=', '')
    @"
PORT=2424
WEBTERM_PASSWORD=$genPassword
"@ | Set-Content -Path $envFile -Encoding utf8
    $isNewInstall = $true
}

# Auto-start on Windows login
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "zh_webterm" -Value "`"$exePath`""

# Start new instance in background
Start-Process -FilePath $exePath -WindowStyle Hidden

Write-Host "✅ zh_webterm installed and running in background on http://localhost:2424" -ForegroundColor Green
if ($isNewInstall) {
    Write-Host "🔑 Initial Generated Password: $genPassword" -ForegroundColor Yellow
}
Write-Host "💡 Config file: $envFile" -ForegroundColor Cyan
