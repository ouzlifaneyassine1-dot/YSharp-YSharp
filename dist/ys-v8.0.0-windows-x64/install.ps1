# Y# v8.0.0 Installer for Windows (PowerShell)
param(
    [string]$InstallDir = "$env:LOCALAPPDATA\YS-Lang"
)

$BinDir = Join-Path $InstallDir "bin"
$StdDir = Join-Path $InstallDir "std"

Write-Host "Installing Y# v8.0.0 to $InstallDir" -ForegroundColor Cyan

# Create directories
New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
New-Item -ItemType Directory -Path $StdDir -Force | Out-Null

# Copy files
$ScriptPath = Split-Path -Parent $PSCommandPath
Copy-Item (Join-Path $ScriptPath "bin\oys.exe") $BinDir -Force
Copy-Item (Join-Path $ScriptPath "bin\yo.exe") $BinDir -Force
Copy-Item -Recurse (Join-Path $ScriptPath "std\*") $StdDir -Force

# Add to PATH for current user
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$BinDir*") {
    $NewPath = "$BinDir;$UserPath"
    [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    Write-Host "Added $BinDir to user PATH" -ForegroundColor Green
}

Write-Host "Y# v8.0.0 installed successfully!" -ForegroundColor Green
Write-Host "Usage: oys build myprogram.ys" -ForegroundColor Yellow
