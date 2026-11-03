@echo off
setlocal
echo =====================================================================
echo  Bornomala (বর্ণমালা) - Unregister and Remove TSF Server
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
        echo [ERROR] bornomala_tsf.dll not found.
        pause
        exit /b 1
    )
)

echo Unregistering Bornomala TSF:
regsvr32.exe /u /s "%DLL_PATH%"

if %ERRORLEVEL% equ 0 (
    echo [SUCCESS] Bornomala unregistered successfully.
) else (
    echo [ERROR] Failed to unregister. Exit code: %ERRORLEVEL%
)

echo.
pause
