@echo off
title Y# (YSharp) Launcher v8.0.3
color 0B

:: Find oys.exe
set OYS_CMD=oys
where oys.exe >nul 2>&1
if %ERRORLEVEL% neq 0 (
    if exist "C:\Program Files\YSharp\bin\oys.exe" set OYS_CMD="C:\Program Files\YSharp\bin\oys.exe"
    if exist "%~dp0oys.exe" set OYS_CMD="%~dp0oys.exe"
    if exist "%~dp0bin\oys.exe" set OYS_CMD="%~dp0bin\oys.exe"
)

:: If a .ys/.yse file was dropped/associated, build and run it directly
if not "%~1"=="" goto buildrun

:: No argument — show menu
:menu
cls
echo.
echo   Y# (YSharp) v8.0.3 Launcher
echo   ============================
echo.
echo   Commands:
echo     1 - Build and run a .ys/.yse file
echo     2 - Open Y# command prompt
echo     3 - Create new Y# project
echo     4 - Build all .ys files in current folder
echo     5 - Register .ys/.yse file association (double-click to run)
echo     6 - Quit
echo.
set /p CHOICE="Choose an option (1-6): "
if "%CHOICE%"=="1" goto choosefile
if "%CHOICE%"=="2" goto prompt
if "%CHOICE%"=="3" goto newproj
if "%CHOICE%"=="4" goto buildall
if "%CHOICE%"=="5" goto register
if "%CHOICE%"=="6" exit /b
goto menu

:choosefile
echo.
set /p FILE="Path to .ys or .yse file: "
if not exist "%FILE%" (
    echo Error: file not found: %FILE%
    pause
    goto menu
)
goto dobuild

:buildrun
set FILE=%~1
if not exist "%FILE%" (
    echo Error: file not found: %FILE%
    pause
    exit /b 1
)

:dobuild
echo.
echo ==^> Building: %FILE%
%OYS_CMD% build "%FILE%"
if %ERRORLEVEL% neq 0 (
    echo.
    echo Build failed!
    pause
    exit /b 1
)
echo.
echo ==^> Running output.exe...
if exist "output.exe" (
    "output.exe"
) else if exist "%~dp0output.exe" (
    "%~dp0output.exe"
) else (
    echo Error: output.exe not found
    pause
    exit /b 1
)
echo.
echo Program exited with code %ERRORLEVEL%
pause
exit /b %ERRORLEVEL%

:prompt
echo.
echo Opening Y# command prompt...
start "Y# Command Prompt" cmd /K "echo Y# (YSharp) v8.0.3 — type oys help to begin"
goto menu

:newproj
echo.
set /p NAME="Project name: "
%OYS_CMD% new "%NAME%"
goto menu

:buildall
echo.
echo ==^> Building all .ys files...
for %%f in (*.ys *.yse) do (
    echo   Building: %%f
    %OYS_CMD% build "%%f"
)
pause
goto menu

:register
echo.
echo Registering .ys and .yse file associations...
set LAUNCHER=%~f0
reg add "HKCU\Software\Classes\.ys" /ve /d "YSharp.File" /f >nul
reg add "HKCU\Software\Classes\.yse" /ve /d "YSharp.File" /f >nul
reg add "HKCU\Software\Classes\YSharp.File\shell\open\command" /ve /d "\"%~s0\" \"%%1\"" /f >nul
reg add "HKCU\Software\Classes\YSharp.File\DefaultIcon" /ve /d "%OYS_CMD%,0" /f >nul
echo.
echo Done! .ys and .yse files will now open with Y# Launcher.
echo You may need to refresh Explorer (F5) for icons to update.
pause
goto menu
