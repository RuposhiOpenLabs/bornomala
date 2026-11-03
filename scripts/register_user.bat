@echo off
setlocal
echo =====================================================================
echo  Bornomala (বর্ণমালা) - Per-User TSF Registration (No Admin Required)
echo  Publisher: Ruposhi Open Labs
echo =====================================================================
echo.

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

echo Registering Bornomala TSF in HKCU:
echo Path: "%DLL_PATH%"
echo.

regsvr32.exe /n /i:user "%DLL_PATH%"
if %ERRORLEVEL% equ 0 (
    echo [SUCCESS] Bornomala registered successfully for current user.
    echo Press F12 or Ctrl+Alt+V in any application to type in Bangla.
) else (
    echo [ERROR] Registration failed with exit code %ERRORLEVEL%.
)

echo.
pause
