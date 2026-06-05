# Y# (YSharp) Windows Installer
# Run: powershell -ExecutionPolicy Bypass -File install.ps1
#
# This installs oys.exe and yo.exe, adds to PATH for all users.

$ErrorActionPreference = "Stop"
$Version = "8.0.1"
$InstallDir = "$env:ProgramFiles\YSharp\bin"

Write-Host "Y# (YSharp) v$Version Installer" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# Check admin rights
$IsAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $IsAdmin) {
    Write-Host "This installer requires Administrator privileges." -ForegroundColor Yellow
    Write-Host "Restarting as Administrator..." -ForegroundColor Yellow
    Start-Process powershell -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
    exit
}

# Locate oys.exe
$ScriptPath = Split-Path -Parent $MyInvocation.MyCommand.Definition
$ProjectRoot = Split-Path -Parent $ScriptPath
$OysExe = Join-Path $ProjectRoot "dist\oys.exe"
$YoExe = Join-Path $ProjectRoot "dist\yo.exe"

if (-not (Test-Path $OysExe)) {
    Write-Host "oys.exe not found at: $OysExe" -ForegroundColor Red
    Write-Host "Build it first: cargo build --release" -ForegroundColor Yellow
    exit 1
}

# Create installation directory
Write-Host "Installing to: $InstallDir" -ForegroundColor White
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

# Copy binaries
Copy-Item -Force $OysExe (Join-Path $InstallDir "oys.exe")
Copy-Item -Force $YoExe (Join-Path $InstallDir "yo.exe")
Write-Host "  oys.exe installed" -ForegroundColor Green
Write-Host "  yo.exe installed" -ForegroundColor Green

# Add to machine PATH
$OldPath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
if ($OldPath -notlike "*$InstallDir*") {
    $NewPath = "$InstallDir;$OldPath"
    [Environment]::SetEnvironmentVariable("PATH", $NewPath, "Machine")
    Write-Host "  Added to system PATH" -ForegroundColor Green
} else {
    Write-Host "  Already in PATH" -ForegroundColor Gray
}

# Create Start Menu shortcut
$StartMenuDir = "$env:ProgramData\Microsoft\Windows\Start Menu\Programs\Y# (YSharp)"
New-Item -ItemType Directory -Force -Path $StartMenuDir | Out-Null

$WScriptShell = New-Object -ComObject WScript.Shell
$Shortcut = $WScriptShell.CreateShortcut("$StartMenuDir\Y# Command Prompt.lnk")
$Shortcut.TargetPath = "cmd.exe"
$Shortcut.Arguments = "/K oys"
$Shortcut.Description = "Y# v$Version Command Prompt"
$Shortcut.WorkingDirectory = "%USERPROFILE%"
$Shortcut.Save()

$Shortcut2 = $WScriptShell.CreateShortcut("$StartMenuDir\Y# Uninstall.lnk")
$Shortcut2.TargetPath = "$env:SystemRoot\System32\cmd.exe"
$Shortcut2.Arguments = "/C echo Run: rm 'C:\Program Files\YSharp' -Recurse -Force"
$Shortcut2.Save()

Write-Host ""
Write-Host "Installation complete!" -ForegroundColor Green
Write-Host "  Run: oys build myprogram.ys" -ForegroundColor White
Write-Host ""
Write-Host "You may need to restart your terminal for PATH changes." -ForegroundColor Yellow
