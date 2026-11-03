@echo off
setlocal
echo =====================================================================
echo  Bornomala (বর্ণমালা) - Disable Windows Auto-Startup
echo  Publisher: Ruposhi Open Labs (https://github.com/RuposhiOpenLabs)
echo  Developer: Ryan Shelby (https://github.com/MdSagorMunshi/)
echo  Support:   ryn@disr.it
echo =====================================================================
echo.

echo Removing Bornomala from Windows Run registry...
reg delete "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v "Bornomala" /f >nul 2>&1
if %ERRORLEVEL% equ 0 (
    echo [SUCCESS] Bornomala auto-startup has been removed.
) else (
    echo [INFO] Bornomala was not registered in the startup run list.
)

echo.
pause
