@echo off
title Y# (YSharp) Launcher v8.0.2
color 0B

:: Find oys.exe - check PATH first, then common install locations
set OYS_CMD=oys
where oys.exe >nul 2>&1
if %ERRORLEVEL% neq 0 (
    if exist "C:\Program Files\YSharp\bin\oys.exe" set OYS_CMD="C:\Program Files\YSharp\bin\oys.exe"
    if exist "%~dp0oys.exe" set OYS_CMD="%~dp0oys.exe"
    if exist "%~dp0bin\oys.exe" set OYS_CMD="%~dp0bin\oys.exe"
)

echo.
echo   Y# (YSharp) v8.0.2 Launcher
echo   ============================
echo.
echo   Commands:
echo     1 - Build and run a .ys/.yse file
echo     2 - Open Y# command prompt
echo     3 - Create new Y# project
echo     4 - Build all .ys files in current folder
echo     5 - Quit
echo.

set /p CHOICE="Choose an option (1-5): "

if "%CHOICE%"=="1" goto buildrun
if "%CHOICE%"=="2" goto prompt
if "%CHOICE%"=="3" goto newproj
if "%CHOICE%"=="4" goto buildall
if "%CHOICE%"=="5" exit /b
goto end

:buildrun
echo.
set /p FILE="Path to .ys or .yse file: "
if not exist "%FILE%" (
    echo Error: file not found: %FILE%
    pause
    goto end
)
echo.
echo ==^> Building: %FILE%
%OYS_CMD% build "%FILE%"
if %ERRORLEVEL% neq 0 (
    echo Build failed!
    pause
    goto end
)
echo.
echo ==^> Running...
set EXE=%~n1.exe
if exist "%~n1.exe" (
    "%~n1.exe"
) else if exist "%FILE%.exe" (
    "%FILE%.exe"
)
pause
goto end

:prompt
echo.
echo Opening Y# command prompt...
start "Y# Command Prompt" cmd /K "%OYS_CMD%"
goto end

:newproj
echo.
set /p NAME="Project name: "
%OYS_CMD% new "%NAME%"
goto end

:buildall
echo.
echo ==^> Building all .ys files...
for %%f in (*.ys *.yse) do (
    echo   Building: %%f
    %OYS_CMD% build "%%f"
)
pause
goto end

:end
echo.
echo Done.
timeout /t 3 >nul
