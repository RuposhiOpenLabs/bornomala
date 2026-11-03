@echo off
setlocal
echo =====================================================================
echo  Bornomala (বর্ণমালা) - System-Wide TSF Registration (Administrator)
echo  Publisher: Ruposhi Open Labs
echo =====================================================================
echo.

net session >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [ERROR] This script requires Administrator privileges.
    echo Please right-click this script and select 'Run as administrator'.
    pause
    exit /b 1
)

set "DLL_PATH=%~dp0bornomala_tsf.dll"

if not exist "%DLL_PATH%" (
    if exist "%~dp0..\target\x86_64-pc-windows-gnu\debug\bornomala_tsf.dll" (
        set "DLL_PATH=%~dp0..\target\x86_64-pc-windows-gnu\debug\bornomala_tsf.dll"
    ) else if exist "%~dp0..\target\x86_64-pc-windows-gnu\release\bornomala_tsf.dll" (
        set "DLL_PATH=%~dp0..\target\x86_64-pc-windows-gnu\release\bornomala_tsf.dll"
    ) else (
        echo [ERROR] bornomala_tsf.dll not found in current directory or target!
        pause
        exit /b 1
    )
)

echo Registering Bornomala TSF in HKLM for all users:
echo Path: "%DLL_PATH%"
echo.

regsvr32.exe /s "%DLL_PATH%"
if %ERRORLEVEL% equ 0 (
    echo [SUCCESS] Bornomala registered system-wide for all users.
) else (
    echo [ERROR] System registration failed with exit code %ERRORLEVEL%.
)

echo.
pause
