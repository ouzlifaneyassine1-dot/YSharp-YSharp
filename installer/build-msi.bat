@echo off
REM Build Y# MSI Installer
REM Requires WiX Toolset: https://wixtoolset.org/
REM Install: choco install wixtoolset

setlocal enabledelayedexpansion

if not exist "%WIX%\bin\candle.exe" (
    echo WiX Toolset not found. Install it with:
    echo   choco install wixtoolset
    echo Or download from: https://wixtoolset.org/
    exit /b 1
)

if not exist "..\dist\oys.exe" (
    echo oys.exe not found! Build the project first:
    echo   cd .. ^&^& cargo build --release
    exit /b 1
)

echo ==^> Compiling MSI for Y# v8.0.1...

"%WIX%\bin\candle.exe" installer.wxs -out installer.wixobj
if %ERRORLEVEL% neq 0 exit /b %ERRORLEVEL%

"%WIX%\bin\light.exe" -ext WixUIExtension installer.wixobj -out ..\dist\YSharp-v8.0.1-windows-x64.msi
if %ERRORLEVEL% neq 0 exit /b %ERRORLEVEL%

echo ==^> MSI created: ..\dist\YSharp-v8.0.1-windows-x64.msi
