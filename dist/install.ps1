param(
    [switch]$Uninstall,
    [switch]$Silent,
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\YSharp"
)

$Version = "8.1.0"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

function Write-Status {
    param([string]$Msg)
    if (-not $Silent) { Write-Host "  $Msg" }
}

function Install-YSharp {
    Write-Host "Y# v$Version Installer"
    Write-Host "====================="
    Write-Host ""

    $BinDir = Join-Path $InstallDir "bin"
    $VscodeDir = Join-Path $InstallDir "vscode"
    $LauncherDir = Join-Path $InstallDir "launcher"

    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    New-Item -ItemType Directory -Force -Path $VscodeDir | Out-Null
    New-Item -ItemType Directory -Force -Path $LauncherDir | Out-Null

    # Copy binaries
    Copy-Item (Join-Path $ScriptDir "oys.exe") (Join-Path $BinDir "oys.exe") -Force
    Copy-Item (Join-Path $ScriptDir "yo.exe") (Join-Path $BinDir "yo.exe") -Force
    Write-Status "Installed oys.exe ($([math]::Round((Get-Item (Join-Path $BinDir "oys.exe")).Length/1MB,2)) MB)"
    Write-Status "Installed yo.exe ($([math]::Round((Get-Item (Join-Path $BinDir "yo.exe")).Length/1MB,2)) MB)"

    # Copy launcher
    $LauncherSrc = Join-Path $ScriptDir "..\launcher\Y# Launcher.bat"
    if (Test-Path $LauncherSrc) {
        Copy-Item $LauncherSrc (Join-Path $LauncherDir "Y# Launcher.bat") -Force
        Write-Status "Installed Y# Launcher"
    }

    # Copy VS Code extension
    $VsixSrc = Join-Path $ScriptDir "y-sharp-v8.0.5.vsix"
    if (Test-Path $VsixSrc) {
        Copy-Item $VsixSrc (Join-Path $VscodeDir "y-sharp-v8.0.5.vsix") -Force
        Write-Status "Copied VS Code extension"
    }

    # Add to PATH
    $CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($CurrentPath -notlike "*$BinDir*") {
        $NewPath = "$BinDir;$CurrentPath"
        [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
        Write-Status "Added $BinDir to user PATH"
        $env:Path = "$BinDir;$env:Path"
    } else {
        Write-Status "Already in PATH"
    }

    # Register file associations
    $LauncherPath = (New-Object -ComObject Scripting.FileSystemObject).GetFile((Join-Path $LauncherDir "Y# Launcher.bat")).ShortPath
    reg add "HKCU\Software\Classes\.ys" /ve /d "YSharp.File" /f *>&1 | Out-Null
    reg add "HKCU\Software\Classes\.yse" /ve /d "YSharp.File" /f *>&1 | Out-Null
    reg add "HKCU\Software\Classes\YSharp.File\shell\open\command" /ve /d "`"$LauncherPath`" `"%%1`"" /f *>&1 | Out-Null
    reg add "HKCU\Software\Classes\YSharp.File\DefaultIcon" /ve /d "`"$BinDir\oys.exe`",0" /f *>&1 | Out-Null
    Write-Status "Registered .ys and .yse file associations"

    Write-Host ""
    Write-Host "Y# v$Version installed successfully!"
    Write-Host "  Binaries: $BinDir"
    Write-Host "  Launcher: $LauncherDir"
    Write-Host ""
    Write-Host "Open a new terminal and try: oys build myprogram.ys"
}

function Uninstall-YSharp {
    Write-Host "Y# v$Version Uninstaller"
    Write-Host "========================="
    Write-Host ""

    $BinDir = Join-Path $InstallDir "bin"
    $LauncherDir = Join-Path $InstallDir "launcher"

    # Remove from PATH
    $CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($CurrentPath -like "*$BinDir*") {
        $NewPath = ($CurrentPath.Split(';') | Where-Object { $_ -ne $BinDir }) -join ';'
        [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
        Write-Status "Removed from user PATH"
    }

    # Remove file associations
    reg delete "HKCU\Software\Classes\.ys" /f | Out-Null
    reg delete "HKCU\Software\Classes\.yse" /f | Out-Null
    reg delete "HKCU\Software\Classes\YSharp.File" /f | Out-Null
    Write-Status "Removed file associations"

    # Remove install directory
    if (Test-Path $InstallDir) {
        Remove-Item -Recurse -Force $InstallDir
        Write-Status "Removed $InstallDir"
    }

    Write-Host ""
    Write-Host "Y# v$Version uninstalled successfully."
}

# Main
if ($Uninstall) {
    Uninstall-YSharp
} else {
    Install-YSharp
}
