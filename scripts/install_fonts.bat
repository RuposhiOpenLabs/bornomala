@echo off
setlocal enabledelayedexpansion
title Bornomala Preloaded Fonts Installer

echo ======================================================================
echo Bornomala (বর্ণমালা) — Preloaded Bangla Fonts Installer
echo Ruposhi Open Labs | Developer: Ryan Shelby (ryn@disr.it)
echo ======================================================================
echo.

set "SCRIPT_DIR=%~dp0"
set "FONTS_DIR=%SCRIPT_DIR%..\fonts"
if not exist "%FONTS_DIR%" (
    set "FONTS_DIR=%SCRIPT_DIR%fonts"
)

if not exist "%FONTS_DIR%" (
    echo [ERROR] Preloaded fonts folder not found at: "%FONTS_DIR%"
    pause
    exit /b 1
)

echo Preloaded Fonts Directory: %FONTS_DIR%
echo.
echo Launching automated font installation via PowerShell...
echo.

powershell -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%install_fonts.ps1" -SourceDir "%FONTS_DIR%" -ConflictMode prompt

echo.
echo ======================================================================
echo Process finished. Press any key to exit.
echo ======================================================================
pause
