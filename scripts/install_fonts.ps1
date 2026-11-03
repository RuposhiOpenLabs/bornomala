# Bornomala Preloaded Bangla Fonts Automated Installer (PowerShell)
# Publisher: Ruposhi Open Labs (https://github.com/RuposhiOpenLabs)
# Lead Developer: Ryan Shelby (https://github.com/MdSagorMunshi/)
# Support: ryn@disr.it
# License: MPL-2.0

[CmdletBinding()]
param(
    [string]$SourceDir = "$PSScriptRoot\..\fonts",
    [ValidateSet("prompt", "keep", "replace")]
    [string]$ConflictMode = "prompt",
    [ValidateSet("user", "machine")]
    [string]$Scope = "user"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $SourceDir)) {
    # Check alternate local folder
    $alt = "$PSScriptRoot\fonts"
    if (Test-Path $alt) {
        $SourceDir = $alt
    } else {
        Write-Error "Preloaded fonts source directory not found: $SourceDir"
        exit 1
    }
}

$IsAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if ($Scope -eq "machine" -and -not $IsAdmin) {
    Write-Warning "Machine scope requested but shell is not elevated. Falling back to current user scope."
    $Scope = "user"
}

if ($Scope -eq "machine") {
    $DestDir = "$env:WINDIR\Fonts"
    $RegKey = "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts"
} else {
    $DestDir = "$env:LOCALAPPDATA\Microsoft\Windows\Fonts"
    $RegKey = "HKCU:\Software\Microsoft\Windows NT\CurrentVersion\Fonts"
}

if (-not (Test-Path $DestDir)) {
    New-Item -ItemType Directory -Path $DestDir -Force | Out-Null
}

$Fonts = @(
    @{ File = "Nikosh.ttf"; Name = "Nikosh (TrueType)"; Desc = "Govt Unicode Standard" },
    @{ File = "NikoshBAN.ttf"; Name = "NikoshBAN (TrueType)"; Desc = "Bengali Typography Variant" },
    @{ File = "NikoshGrameen.ttf"; Name = "NikoshGrameen (TrueType)"; Desc = "Grameen Display Variant" },
    @{ File = "NikoshLight.ttf"; Name = "NikoshLight (TrueType)"; Desc = "Ultra-Clean Light" },
    @{ File = "NikoshLightBan.ttf"; Name = "NikoshLightBan (TrueType)"; Desc = "Bengali Print Light" },
    @{ File = "SutonnyMJ Regular.ttf"; Name = "SutonnyMJ (TrueType)"; Desc = "Classic ANSI Font" },
    @{ File = "AdorshoLipi_20-07-2007.ttf"; Name = "AdorshoLipi (TrueType)"; Desc = "Educational Font" },
    @{ File = "Bangla.ttf"; Name = "Bangla (TrueType)"; Desc = "Traditional OpenType Font" }
)

Write-Host "======================================================================" -ForegroundColor Green
Write-Host "Bornomala (বর্ণমালা) — Preloaded Bangla Fonts Installer" -ForegroundColor Green
Write-Host "Ruposhi Open Labs | Developer: Ryan Shelby (ryn@disr.it)" -ForegroundColor Green
Write-Host "Target Directory: $DestDir" -ForegroundColor Cyan
Write-Host "Registry Key:     $RegKey" -ForegroundColor Cyan
Write-Host "Conflict Mode:    $ConflictMode" -ForegroundColor Cyan
Write-Host "======================================================================" -ForegroundColor Green

$installedCount = 0
$skippedCount = 0

foreach ($f in $Fonts) {
    $src = Join-Path $SourceDir $f.File
    if (-not (Test-Path $src)) {
        Write-Warning "Font source file missing: $src"
        continue
    }

    $dest = Join-Path $DestDir $f.File
    $alreadyExists = Test-Path $dest

    $shouldCopy = $true
    if ($alreadyExists) {
        if ($ConflictMode -eq "keep") {
            Write-Host "  [KEEP] Existing font preserved: $($f.Name)" -ForegroundColor Yellow
            $shouldCopy = $false
        } elseif ($ConflictMode -eq "replace") {
            Write-Host "  [REPLACE] Overwriting with preloaded: $($f.Name)" -ForegroundColor Green
            $shouldCopy = $true
        } else {
            Write-Host "  [CONFLICT] Font '$($f.Name)' already exists." -ForegroundColor Magenta
            $ans = Read-Host "    Keep existing [K] or Replace with Bornomala preloaded [R]? (default: K)"
            if ($ans -eq "R" -or $ans -eq "r") {
                $shouldCopy = $true
            } else {
                $shouldCopy = $false
            }
        }
    }

    if ($shouldCopy) {
        Copy-Item -Path $src -Destination $dest -Force
        Set-ItemProperty -Path $RegKey -Name $f.Name -Value $dest -Force
        Write-Host "  [INSTALLED] $($f.Name) ($($f.Desc))" -ForegroundColor Green
        $installedCount++
    } else {
        $skippedCount++
    }
}

# Broadcast WM_FONTCHANGE so all running programs (Office, Photoshop, Browsers) reload fonts immediately
try {
    $HWND_BROADCAST = [IntPtr]0xffff
    $WM_FONTCHANGE = 0x001D
    $sig = @"
    using System;
    using System.Runtime.InteropServices;
    public class Win32 {
        [DllImport("user32.dll", CharSet = CharSet.Auto)]
        public static extern IntPtr SendMessage(IntPtr hWnd, uint Msg, UIntPtr wParam, IntPtr lParam);
    }
"@
    Add-Type -TypeDefinition $sig -ErrorAction SilentlyContinue
    [Win32]::SendMessage($HWND_BROADCAST, $WM_FONTCHANGE, [UIntPtr]::Zero, [IntPtr]::Zero) | Out-Null
    Write-Host "  [BROADCAST] Notified Windows system of font updates (WM_FONTCHANGE)." -ForegroundColor DarkGray
} catch {
    # Non-critical if broadcast encounters restriction
}

Write-Host "======================================================================" -ForegroundColor Green
Write-Host "Installation Complete! Installed/Updated: $installedCount | Kept Existing: $skippedCount" -ForegroundColor Green
Write-Host "======================================================================" -ForegroundColor Green
