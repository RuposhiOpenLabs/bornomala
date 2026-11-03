@echo off
setlocal
echo =====================================================================
echo  Bornomala (বর্ণমালা) - Enable Windows Auto-Startup
echo  Publisher: Ruposhi Open Labs (https://github.com/RuposhiOpenLabs)
echo  Developer: Ryan Shelby (https://github.com/MdSagorMunshi/)
echo  Support:   ryn@disr.it
echo =====================================================================
echo.

set "EXE_PATH=%~dp0bornomala.exe"

if not exist "%EXE_PATH%" (
    if exist "%~dp0..\target\x86_64-pc-windows-gnu\release\bornomala-cli.exe" (
        set "EXE_PATH=%~dp0..\target\x86_64-pc-windows-gnu\release\bornomala-cli.exe"
    ) else if exist "%~dp0..\target\x86_64-pc-windows-gnu\debug\bornomala-cli.exe" (
        set "EXE_PATH=%~dp0..\target\x86_64-pc-windows-gnu\debug\bornomala-cli.exe"
    ) else if exist "%LOCALAPPDATA%\Bornomala\bornomala.exe" (
        set "EXE_PATH=%LOCALAPPDATA%\Bornomala\bornomala.exe"
    ) else if exist "%ProgramFiles%\Bornomala\bornomala.exe" (
        set "EXE_PATH=%ProgramFiles%\Bornomala\bornomala.exe"
    )
)

echo Configuring Windows Run registry for Bornomala:
echo Executable: "%EXE_PATH%"
echo Key:        HKCU\Software\Microsoft\Windows\CurrentVersion\Run
echo.

reg add "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "Bornomala" /t REG_SZ /d "\"%EXE_PATH%\" --startup" /f
if %ERRORLEVEL% equ 0 (
    echo [SUCCESS] Bornomala is now registered to start automatically on Windows logon.
    echo Upon startup, Bornomala will briefly display the startup splash screen (Bijoy/Ekushe style)
    echo and remain active in the system tray and TSF input method.
) else (
    echo [ERROR] Failed to add registry key. Exit code: %ERRORLEVEL%
)

echo.
pause
