use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontInstallScope {
    CurrentUser,
    AllUsers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontConflictAction {
    KeepExisting,
    ReplaceWithPreloaded,
    KeepAll,
    ReplaceAll,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreloadedFont {
    pub filename: &'static str,
    pub font_name: &'static str,
    pub registry_name: &'static str,
    pub is_ansi: bool,
    pub is_unicode: bool,
    pub description: &'static str,
    pub description_bn: &'static str,
}

pub const PRELOADED_FONTS: [PreloadedFont; 8] = [
    PreloadedFont {
        filename: "Nikosh.ttf",
        font_name: "Nikosh",
        registry_name: "Nikosh (TrueType)",
        is_ansi: false,
        is_unicode: true,
        description: "Standard Bangladesh Government (BCC) Unicode Font",
        description_bn: "বাংলাদেশ সরকার (বিসিসি) নির্ধারিত প্রমিত ইউনিকোড ফন্ট",
    },
    PreloadedFont {
        filename: "NikoshBAN.ttf",
        font_name: "NikoshBAN",
        registry_name: "NikoshBAN (TrueType)",
        is_ansi: false,
        is_unicode: true,
        description: "Nikosh Official Bengali Typography Variant",
        description_bn: "নিকশ অফিশিয়াল বাংলা রূপভেদ",
    },
    PreloadedFont {
        filename: "NikoshGrameen.ttf",
        font_name: "NikoshGrameen",
        registry_name: "NikoshGrameen (TrueType)",
        is_ansi: false,
        is_unicode: true,
        description: "Nikosh Grameen Clear Display Variant",
        description_bn: "নিকশ গ্রামীণ স্পষ্ট ডিসপ্লে ফন্ট",
    },
    PreloadedFont {
        filename: "NikoshLight.ttf",
        font_name: "NikoshLight",
        registry_name: "NikoshLight (TrueType)",
        is_ansi: false,
        is_unicode: true,
        description: "Nikosh Ultra-Clean Light Variant",
        description_bn: "নিকশ লাইট ওজন রূপভেদ",
    },
    PreloadedFont {
        filename: "NikoshLightBan.ttf",
        font_name: "NikoshLightBan",
        registry_name: "NikoshLightBan (TrueType)",
        is_ansi: false,
        is_unicode: true,
        description: "Nikosh Light Bengali Print Variant",
        description_bn: "নিকশ লাইট বাংলা মুদ্রণ রূপভেদ",
    },
    PreloadedFont {
        filename: "SutonnyMJ Regular.ttf",
        font_name: "SutonnyMJ",
        registry_name: "SutonnyMJ (TrueType)",
        is_ansi: true,
        is_unicode: false,
        description: "Essential Classic ANSI Font for Government Documents & Land Registry",
        description_bn: "সরকারি নথি ও ভূমিসেবার ক্লাসিক এএনএসআই ফন্ট",
    },
    PreloadedFont {
        filename: "AdorshoLipi_20-07-2007.ttf",
        font_name: "AdorshoLipi",
        registry_name: "AdorshoLipi (TrueType)",
        is_ansi: false,
        is_unicode: true,
        description: "Standard Educational & Literary Bangla Font",
        description_bn: "শিক্ষা ও সাহিত্যচর্চার উপযোগী আদর্শলিপি ফন্ট",
    },
    PreloadedFont {
        filename: "Bangla.ttf",
        font_name: "Bangla",
        registry_name: "Bangla (TrueType)",
        is_ansi: true,
        is_unicode: true,
        description: "Universal Bangla Traditional OpenType Font",
        description_bn: "সার্বজনীন বাংলা ঐতিহ্যবাহী ফন্ট",
    },
];

pub fn get_preloaded_fonts() -> &'static [PreloadedFont] {
    &PRELOADED_FONTS
}

pub fn generate_powershell_font_installer(font_source_dir: &str, scope: FontInstallScope) -> String {
    let (reg_key, dest_dir) = match scope {
        FontInstallScope::CurrentUser => (
            r"HKCU:\Software\Microsoft\Windows NT\CurrentVersion\Fonts",
            r"$env:LOCALAPPDATA\Microsoft\Windows\Fonts",
        ),
        FontInstallScope::AllUsers => (
            r"HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts",
            r"$env:WINDIR\Fonts",
        ),
    };

    format!(
        r###"# Bornomala Preloaded Fonts Automated Installer
# Publisher: Ruposhi Open Labs (https://github.com/RuposhiOpenLabs)
# Lead Developer: Ryan Shelby (ryn@disr.it)

param(
    [string]$SourceDir = "{font_source_dir}",
    [string]$ConflictMode = "prompt" # prompt | keep | replace
)

$ErrorActionPreference = "Stop"

$DestDir = "{dest_dir}"
if (-not (Test-Path $DestDir)) {{
    New-Item -ItemType Directory -Path $DestDir -Force | Out-Null
}}

$Fonts = @(
    @{{ File = "Nikosh.ttf"; Name = "Nikosh (TrueType)" }},
    @{{ File = "NikoshBAN.ttf"; Name = "NikoshBAN (TrueType)" }},
    @{{ File = "NikoshGrameen.ttf"; Name = "NikoshGrameen (TrueType)" }},
    @{{ File = "NikoshLight.ttf"; Name = "NikoshLight (TrueType)" }},
    @{{ File = "NikoshLightBan.ttf"; Name = "NikoshLightBan (TrueType)" }},
    @{{ File = "SutonnyMJ Regular.ttf"; Name = "SutonnyMJ (TrueType)" }},
    @{{ File = "AdorshoLipi_20-07-2007.ttf"; Name = "AdorshoLipi (TrueType)" }},
    @{{ File = "Bangla.ttf"; Name = "Bangla (TrueType)" }}
)

Write-Host "Installing Bornomala Preloaded Bangla Fonts into $DestDir..." -ForegroundColor Cyan

foreach ($f in $Fonts) {{
    $src = Join-Path $SourceDir $f.File
    if (-not (Test-Path $src)) {{
        Write-Warning "Font source file not found: $src"
        continue
    }}

    $dest = Join-Path $DestDir $f.File
    $alreadyExists = Test-Path $dest

    $shouldCopy = $true
    if ($alreadyExists) {{
        if ($ConflictMode -eq "keep") {{
            Write-Host "  [KEEP] Existing font preserved: $($f.Name)" -ForegroundColor Yellow
            $shouldCopy = $false
        }} elseif ($ConflictMode -eq "replace") {{
            Write-Host "  [REPLACE] Overwriting with preloaded: $($f.Name)" -ForegroundColor Green
            $shouldCopy = $true
        }} else {{
            # Interactive prompt if running from console
            Write-Host "  [CONFLICT] Font '$($f.Name)' already exists on this machine." -ForegroundColor Magenta
            $ans = Read-Host "  Keep existing [K] or Replace with Bornomala preloaded [R]? (default: K)"
            if ($ans -eq "R" -or $ans -eq "r") {{
                $shouldCopy = $true
            }} else {{
                $shouldCopy = $false
            }}
        }}
    }}

    if ($shouldCopy) {{
        Copy-Item -Path $src -Destination $dest -Force
        Set-ItemProperty -Path "{reg_key}" -Name $f.Name -Value $dest -Force
        Write-Host "  [INSTALLED] $($f.Name) -> $dest" -ForegroundColor Green
    }}
}}

# Broadcast WM_FONTCHANGE so running apps reload fonts without reboot
try {{
    $HWND_BROADCAST = [IntPtr]0xffff
    $WM_FONTCHANGE = 0x001D
    Add-Type -TypeDefinition @"
    using System;
    using System.Runtime.InteropServices;
    public class Win32 {{
        [DllImport("user32.dll", CharSet = CharSet.Auto)]
        public static extern IntPtr SendMessage(IntPtr hWnd, uint Msg, UIntPtr wParam, IntPtr lParam);
    }}
"@
    [Win32]::SendMessage($HWND_BROADCAST, $WM_FONTCHANGE, [UIntPtr]::Zero, [IntPtr]::Zero) | Out-Null
}} catch {{
    # Non-critical if broadcast fails in restricted shell
}}

Write-Host "Bornomala Preloaded Fonts setup completed successfully!" -ForegroundColor Cyan
"###,
        font_source_dir = font_source_dir,
        dest_dir = dest_dir,
        reg_key = reg_key
    )
}
