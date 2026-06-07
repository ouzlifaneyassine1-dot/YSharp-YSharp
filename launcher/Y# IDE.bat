@echo off
title Y# IDE
setlocal

:: Find the IDE directory (same as launcher location)
set IDE_DIR=%~dp0
if "%IDE_DIR:~-1%"=="\" set IDE_DIR=%IDE_DIR:~0,-1%
set IDE_DIR=%IDE_DIR%\..\ide

:: Find Node.js
where node.exe >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Error: Node.js is required to run the Y# IDE.
    echo Download from: https://nodejs.org/
    pause
    exit /b 1
)

:: Run the IDE via npx electron
echo Starting Y# IDE...
cd /d "%IDE_DIR%"
if not exist "node_modules" (
    echo Installing dependencies (first launch)...
    call npm install --no-audit --no-fund 2>&1
)
npx electron main.js
exit /b %ERRORLEVEL%
