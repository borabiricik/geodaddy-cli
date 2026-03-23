@echo off
REM geodaddy installer for Windows CMD
REM Usage: Download and run this file, or:
REM   curl -fsSL https://raw.githubusercontent.com/borabiricik/geodaddy-cli/main/install.cmd -o install.cmd && install.cmd

setlocal enabledelayedexpansion

set "REPO=borabiricik/geodaddy-cli"
set "BINARY=geodaddy"
set "INSTALL_DIR=%USERPROFILE%\.geodaddy\bin"

echo Fetching latest release...

REM Check for curl (available on Windows 10+)
where curl >nul 2>&1
if %errorlevel% neq 0 (
    echo error: curl not found. Please use install.ps1 with PowerShell instead.
    exit /b 1
)

REM Get latest version
for /f "tokens=2 delims=:, " %%a in ('curl -fsSL "https://api.github.com/repos/%REPO%/releases/latest" 2^>nul ^| findstr "tag_name"') do (
    set "VERSION=%%~a"
)

if "%VERSION%"=="" (
    echo error: Could not determine latest version.
    echo Check https://github.com/%REPO%/releases
    exit /b 1
)

set "TARGET=x86_64-pc-windows-msvc"
set "ARCHIVE=%BINARY%-%VERSION%-%TARGET%.zip"
set "URL=https://github.com/%REPO%/releases/download/%VERSION%/%ARCHIVE%"

echo Installing %BINARY% %VERSION% (%TARGET%)...

REM Create install directory
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

REM Create temp directory
set "TMP_DIR=%TEMP%\geodaddy-install-%RANDOM%"
mkdir "%TMP_DIR%"

REM Download
echo Downloading %URL%...
curl -fsSL "%URL%" -o "%TMP_DIR%\%ARCHIVE%"
if %errorlevel% neq 0 (
    echo error: Download failed. Is the release published?
    rmdir /s /q "%TMP_DIR%"
    exit /b 1
)

REM Extract using PowerShell (available on all modern Windows)
powershell -NoProfile -Command "Expand-Archive -Path '%TMP_DIR%\%ARCHIVE%' -DestinationPath '%TMP_DIR%' -Force"
if %errorlevel% neq 0 (
    echo error: Failed to extract archive.
    rmdir /s /q "%TMP_DIR%"
    exit /b 1
)

REM Install
copy /y "%TMP_DIR%\%BINARY%.exe" "%INSTALL_DIR%\%BINARY%.exe" >nul

REM Cleanup
rmdir /s /q "%TMP_DIR%"

REM Check if install dir is in PATH
echo %PATH% | findstr /i "%INSTALL_DIR%" >nul
if %errorlevel% neq 0 (
    echo.
    echo NOTE: %INSTALL_DIR% is not in your PATH.
    echo Add it manually or run:
    echo   setx PATH "%%PATH%%;%INSTALL_DIR%"
    echo Then restart your terminal.
)

echo.
echo Successfully installed %BINARY% %VERSION%
echo Location: %INSTALL_DIR%\%BINARY%.exe
echo.
echo Run 'geodaddy --help' to get started.

endlocal
