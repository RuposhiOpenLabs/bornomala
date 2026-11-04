; ============================================================================
; Bornomala (বর্ণমালা) — Professional Windows Setup Script (NSIS)
; "Your mother tongue doesn't need a license key."
; Publisher: Ruposhi Open Labs (https://github.com/RuposhiOpenLabs)
; Lead Developer: Ryan Shelby (https://github.com/MdSagorMunshi/)
; Support: ryn@disr.it
; ============================================================================

Unicode true

!define PRODUCT_NAME "Bornomala (বর্ণমালা)"
!define PRODUCT_VERSION "0.1.0"
!define PRODUCT_PUBLISHER "Ruposhi Open Labs"
!define PRODUCT_DEVELOPER "Ryan Shelby"
!define PRODUCT_WEB_SITE "https://github.com/RuposhiOpenLabs/bornomala"
!define PRODUCT_DIR_REGKEY "Software\Bornomala"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\Bornomala"
!define PRODUCT_UNINST_ROOT_KEY "HKLM"
!define PRODUCT_AUTORUN_KEY "Software\Microsoft\Windows\CurrentVersion\Run"

; ----------------------------------------------------------------------------
; Setup Metadata, Execution Level & Compression
; ----------------------------------------------------------------------------
Name "${PRODUCT_NAME} v${PRODUCT_VERSION}"
OutFile "../target/installer/Bornomala-Setup-v${PRODUCT_VERSION}.exe"
InstallDir "$PROGRAMFILES64\Bornomala"
InstallDirRegKey HKLM "${PRODUCT_DIR_REGKEY}" "InstallDir"
RequestExecutionLevel admin
SetCompressor /SOLID lzma
ShowInstDetails show
ShowUnInstDetails show

; Force Windows Indic standard font (Nirmala UI) across all wizard controls
SetFont "Nirmala UI" 9

; ----------------------------------------------------------------------------
; Modern UI 2 Configuration & Branding
; ----------------------------------------------------------------------------
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "WinMessages.nsh"
!include "FileFunc.nsh"
!include "x64.nsh"

!define MUI_ICON "bornomala.ico"
!define MUI_UNICON "bornomala.ico"
!define MUI_HEADERIMAGE
!define MUI_HEADERIMAGE_BITMAP "header.bmp"
!define MUI_WELCOMEFINISHPAGE_BITMAP "welcome.bmp"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "welcome.bmp"
!define MUI_ABORTWARNING

; Welcome Page
!define MUI_WELCOMEPAGE_TITLE "স্বাগতম — বর্ণমালা (Bornomala) সেটআপ উইজার্ড"
!define MUI_WELCOMEPAGE_TEXT "বর্ণমালা হলো ১০০% মুক্ত, ওপেন সোর্স, অফলাইন ও আধুনিক বাংলা টাইপিং স্যুট।$\r$\n$\r$\nসুবিধাসমূহ:$\r$\n• ক্লাসিক ইউনিকোড ও ফোনেটিক রোমানাইজড টাইপিং (Avro ও Bijoy স্টাইল)$\r$\n• সুতোন্নী (SutonnyMJ) এএনএসআই লেআউট ও হাই-স্পিড রূপান্তরকারী$\r$\n• সরকারি প্রমিত ৮টি প্রিলোডেড ফন্ট (নিকশ, সুতোন্নীএমজে, আদর্শলিপি ইত্যাদি)$\r$\n• কোনো ক্র্যাক নেই, কোনো ভাইরাস নেই, কোনো লাইসেন্স কি-র প্রয়োজন নেই।$\r$\n$\r$\nইনস্টলেশন শুরু করতে [পরবর্তী] বাটনে ক্লিক করুন।"
!insertmacro MUI_PAGE_WELCOME

; License Page (Terms & Conditions — Clean Native UTF-16LE with BOM)
!define MUI_LICENSEPAGE_CHECKBOX
!define MUI_LICENSEPAGE_TEXT_TOP "বর্ণমালা ব্যবহারের পূর্বে অনুগ্রহ করে শর্তাবলী ও অফলাইন নীতিমালা পাঠ করুন:"
!define MUI_LICENSEPAGE_CHECKBOX_TEXT "আমি শর্তাবলী ও গোপনীয়তার নীতিমালা পড়েছি এবং সম্মত আছি (I Agree)"
!insertmacro MUI_PAGE_LICENSE "terms.txt"

; Components Selection Page
!define MUI_COMPONENTSPAGE_TEXT_TOP "যেসব উপাদান ইনস্টল করতে চান তা নির্বাচন করুন:"
!insertmacro MUI_PAGE_COMPONENTS

; Directory Selection Page
!insertmacro MUI_PAGE_DIRECTORY

; Installation Progress Page
!insertmacro MUI_PAGE_INSTFILES

; Finish Page
!define MUI_FINISHPAGE_TITLE "অভিনন্দন! বর্ণমালা ইনস্টলেশন সম্পন্ন হয়েছে"
!define MUI_FINISHPAGE_TEXT "বর্ণমালা আপনার কম্পিউটারে সফলভাবে ইনস্টল ও কনফিগার হয়েছে।$\r$\n$\r$\nশর্টকাটসমূহ:$\r$\n• F12 / Ctrl + Space: ফোনেটিক বাংলা চালু/বন্ধ$\r$\n• Ctrl + Alt + V: ক্লাসিক ইউনিকোড$\r$\n• Ctrl + Alt + B: ক্লাসিক এএনএসআই (SutonnyMJ)$\r$\n• Ctrl + Alt + E: ইংরেজি (Pass-through)"
!define MUI_FINISHPAGE_RUN "$INSTDIR\bornomala.exe"
!define MUI_FINISHPAGE_RUN_TEXT "এখনই বর্ণমালা চালু করুন (Launch Bornomala)"
!define MUI_FINISHPAGE_LINK "বর্ণমালা গিটহাব সংগ্রহশালা (GitHub Repository)"
!define MUI_FINISHPAGE_LINK_LOCATION "https://github.com/RuposhiOpenLabs/bornomala"
!insertmacro MUI_PAGE_FINISH

; Uninstaller Pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

; Override language font to Nirmala UI for titles, headers, and labels
LangString ^Font ${LANG_ENGLISH} "Nirmala UI"
LangString ^FontSize ${LANG_ENGLISH} "9"

; ----------------------------------------------------------------------------
; Helper Function: Safely Close Running Bornomala Instances Without taskkill.exe
; (Direct Win32 API calls prevent external taskkill.exe and framedynos.dll errors)
; ----------------------------------------------------------------------------
Function CloseRunningInstances
  Push $0
  Push $1
  Push $2

  ; 1. Check Main App Window (BornomalaAppClass)
  FindWindow $0 "BornomalaAppClass" ""
  StrCmp $0 0 check_tb
    SendMessage $0 16 0 0 ; WM_CLOSE
    Sleep 250
    ; If still alive, terminate process cleanly via PID
    System::Call 'user32::GetWindowThreadProcessId(p r0, *i .r1)'
    StrCmp $1 0 check_tb
    System::Call 'kernel32::OpenProcess(i 0x0001, i 0, i r1) p .r2'
    StrCmp $2 0 check_tb
    System::Call 'kernel32::TerminateProcess(p r2, i 0)'
    System::Call 'kernel32::CloseHandle(p r2)'

check_tb:
  ; 2. Check Floating Toolbar Window (BornomalaToolbarClass)
  FindWindow $0 "BornomalaToolbarClass" ""
  StrCmp $0 0 check_conv
    SendMessage $0 16 0 0 ; WM_CLOSE
    Sleep 200
    System::Call 'user32::GetWindowThreadProcessId(p r0, *i .r1)'
    StrCmp $1 0 check_conv
    System::Call 'kernel32::OpenProcess(i 0x0001, i 0, i r1) p .r2'
    StrCmp $2 0 check_conv
    System::Call 'kernel32::TerminateProcess(p r2, i 0)'
    System::Call 'kernel32::CloseHandle(p r2)'

check_conv:
  FindWindow $0 "BornomalaConverterClass" ""
  StrCmp $0 0 check_sett
    SendMessage $0 16 0 0

check_sett:
  FindWindow $0 "BornomalaSettingsClass" ""
  StrCmp $0 0 check_abt
    SendMessage $0 16 0 0

check_abt:
  FindWindow $0 "BornomalaAboutClass" ""
  StrCmp $0 0 done_close
    SendMessage $0 16 0 0

done_close:
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

; ----------------------------------------------------------------------------
; Uninstaller Callback: Verify That Bornomala Is NOT Running
; If running, show an error message and require the user to completely close it.
; ----------------------------------------------------------------------------
Function un.EnsureNotRunning
  Push $0

  un_check_loop:
    ; 1. Check Single-Instance Named Mutex
    System::Call 'kernel32::OpenMutex(i 0x00100000, i 0, t "Bornomala_SingleInstance_Mutex") p .r0'
    IntCmp $0 0 un_check_windows
      ; Mutex exists - application is active!
      System::Call 'kernel32::CloseHandle(p r0)'
      Goto un_is_running

  un_check_windows:
    ; 2. Check Application Window Classes
    FindWindow $0 "BornomalaAppClass" ""
    IntCmp $0 0 +2
      Goto un_is_running

    FindWindow $0 "BornomalaToolbarClass" ""
    IntCmp $0 0 +2
      Goto un_is_running

    FindWindow $0 "BornomalaConverterClass" ""
    IntCmp $0 0 +2
      Goto un_is_running

    FindWindow $0 "BornomalaSettingsClass" ""
    IntCmp $0 0 +2
      Goto un_is_running

    FindWindow $0 "BornomalaAboutClass" ""
    IntCmp $0 0 un_not_running
      Goto un_is_running

  un_is_running:
    MessageBox MB_RETRYCANCEL|MB_ICONSTOP \
      "বর্ণমালা (Bornomala) বর্তমানে চালু রয়েছে!$\r$\n$\r$\nঅ্যাপ্লিকেশনটি পুরোপুরি বন্ধ না করলে ফাইল আনইনস্টল করা সম্ভব নয়।$\r$\nঅনুগ্রহ করে টাস্কবার/টুলবার থেকে বর্ণমালা বন্ধ (Exit) করুন এবং 'Retry' চাপুন।$\r$\n$\r$\nBornomala is currently running!$\r$\nPlease completely close/exit Bornomala from taskbar/tray and click Retry.$\r$\nClick Cancel to abort uninstallation." \
      IDRETRY un_check_loop IDCANCEL un_do_abort

  un_do_abort:
    Pop $0
    Abort

  un_not_running:
    Pop $0
FunctionEnd

; ----------------------------------------------------------------------------
; Initialization Callback: Enable 64-bit Registry & File System Redirection
; ----------------------------------------------------------------------------
Function .onInit
  SetRegView 64
  ; Close any running instances gracefully without taskkill.exe
  Call CloseRunningInstances
FunctionEnd

Function un.onInit
  SetRegView 64
  ; Enforce that Bornomala must be closed before uninstallation begins
  Call un.EnsureNotRunning
FunctionEnd

; ----------------------------------------------------------------------------
; Font Conflict Resolution Function
; ----------------------------------------------------------------------------
; Installs preloaded Bangla fonts directly into system C:\Windows\Fonts
; If a font already exists, asks the user whether to replace or keep it.
Function InstallFontWithConflictCheck
  Exch $R0 ; FontRegName
  Exch
  Exch $R1 ; FontFileName
  Push $R2
  Push $R3

  ; System font directory (C:\Windows\Fonts)
  StrCpy $R2 "$FONTS"
  StrCpy $R3 "$R2\$R1"

  IfFileExists "$R3" font_conflict_found do_copy_font

font_conflict_found:
  MessageBox MB_YESNO|MB_ICONQUESTION \
    "Preloaded Font Conflict / ফন্ট সংঘাত:$\r$\n$\r$\nফন্ট ফাইল '$R1' ইতিমধ্যে উইন্ডোজ সিস্টেম ফন্ট ফোল্ডারে ইনস্টল করা আছে।$\r$\n$\r$\nআপনি কি এটি বর্ণমালার প্রিলোডেড সংস্করণ দ্বারা প্রতিস্থাপন (Replace) করতে চান?$\r$\n$\r$\n• [Yes / হ্যাঁ]: বর্ণমালার প্রিলোডেড ফন্ট প্রতিস্থাপন করুন$\r$\n• [No / না]: পূর্বের বিদ্যমান ফন্টটি বজায় রাখুন" \
    IDYES do_copy_font IDNO keep_existing_font

do_copy_font:
  DetailPrint "ইনস্টল হচ্ছে প্রিলোডেড ফন্ট: $R1"
  CopyFiles /SILENT "$INSTDIR\fonts\$R1" "$R3"
  WriteRegStr HKLM "Software\Microsoft\Windows NT\CurrentVersion\Fonts" "$R0" "$R1"
  ; Register font resource in active Windows session
  System::Call "gdi32::AddFontResource(t '$R3') i .r0"
  Goto font_finished

keep_existing_font:
  DetailPrint "পূর্বের বিদ্যমান ফন্টটি অপরিবর্তিত রাখা হয়েছে: $R1"

font_finished:
  Pop $R3
  Pop $R2
  Pop $R1
  Pop $R0
FunctionEnd

; ----------------------------------------------------------------------------
; Installation Sections
; ----------------------------------------------------------------------------

Section "Bornomala Core Engine & TSF (প্রয়োজনীয়)" SecCore
  SectionIn RO
  SetShellVarContext all
  SetOutPath "$INSTDIR"
  
  ; Ensure any previous running instances are terminated so files are not locked
  DetailPrint "পূর্বের চালু থাকা ইনস্ট্যান্স বন্ধ করা হচ্ছে..."
  Call CloseRunningInstances
  Sleep 200

  ; Write Native Windows GUI Application & Libraries
  File "/oname=bornomala.exe" "../target/x86_64-pc-windows-gnu/release/bornomala.exe"
  File "/oname=bornomala_tsf.dll" "../target/x86_64-pc-windows-gnu/release/bornomala_tsf.dll"
  File "/oname=bornomala-cli.exe" "../target/x86_64-pc-windows-gnu/release/bornomala-cli.exe"
  File "/oname=TERMS.txt" "terms.txt"
  File "bornomala.ico"

  ; Out branding assets
  CreateDirectory "$INSTDIR\assets\branding"
  SetOutPath "$INSTDIR\assets\branding"
  File "/oname=startup_screen_bn.bmp" "../assets/branding/startup_screen_bn.bmp"
  File "/oname=startup_screen_bn.png" "../assets/branding/startup_screen_bn.png"
  File "/oname=startup_screen_en.png" "../assets/branding/startup_screen_en.png"
  File "/oname=logo.png" "../assets/branding/logo.png"
  File "/oname=bornomala_logo_mark.png" "../assets/branding/bornomala_logo_mark.png"
  File "/oname=bornomala_icon_512.png" "../assets/branding/bornomala_icon_512.png"
  SetOutPath "$INSTDIR"

  ; Register TSF InprocServer32 COM server system-wide (64-bit native)
  DetailPrint "রেজিস্ট্রি করা হচ্ছে TSF COM টেক্সট সার্ভিস..."
  ${If} ${RunningX64}
    ${DisableX64FSRedirection}
    ExecWait 'regsvr32.exe /s "$INSTDIR\bornomala_tsf.dll"'
    ${EnableX64FSRedirection}
  ${Else}
    ExecWait 'regsvr32.exe /s "$INSTDIR\bornomala_tsf.dll"'
  ${EndIf}

  ; Write Application Registry Keys in HKLM
  WriteRegStr HKLM "${PRODUCT_DIR_REGKEY}" "InstallDir" "$INSTDIR"
  WriteRegStr HKLM "${PRODUCT_DIR_REGKEY}" "Version" "${PRODUCT_VERSION}"
  WriteRegStr HKLM "${PRODUCT_DIR_REGKEY}" "Publisher" "${PRODUCT_PUBLISHER}"

  ; Add/Remove Programs Registry Entries in HKLM
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayName" "${PRODUCT_NAME}"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\bornomala.ico"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "HelpLink" "mailto:ryn@disr.it"

  ; Write Uninstaller
  WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Preloaded Bangla Fonts (প্রিলোডেড বাংলা ফন্ট)" SecFonts
  DetailPrint "প্রিলোডেড বাংলা ফন্ট ইনস্টলেশন শুরু হচ্ছে..."
  
  ; Stage font files into application fonts directory
  CreateDirectory "$INSTDIR\fonts"
  SetOutPath "$INSTDIR\fonts"
  File "../fonts/Nikosh.ttf"
  File "../fonts/NikoshBAN.ttf"
  File "../fonts/NikoshGrameen.ttf"
  File "../fonts/NikoshLight.ttf"
  File "../fonts/NikoshLightBan.ttf"
  File "../fonts/SutonnyMJ Regular.ttf"
  File "../fonts/AdorshoLipi_20-07-2007.ttf"
  File "../fonts/Bangla.ttf"

  ; Install each font with conflict detection & interactive user prompt
  Push "Nikosh.ttf"
  Push "Nikosh (TrueType)"
  Call InstallFontWithConflictCheck

  Push "NikoshBAN.ttf"
  Push "NikoshBAN (TrueType)"
  Call InstallFontWithConflictCheck

  Push "NikoshGrameen.ttf"
  Push "NikoshGrameen (TrueType)"
  Call InstallFontWithConflictCheck

  Push "NikoshLight.ttf"
  Push "NikoshLight (TrueType)"
  Call InstallFontWithConflictCheck

  Push "NikoshLightBan.ttf"
  Push "NikoshLightBan (TrueType)"
  Call InstallFontWithConflictCheck

  Push "SutonnyMJ Regular.ttf"
  Push "SutonnyMJ (TrueType)"
  Call InstallFontWithConflictCheck

  Push "AdorshoLipi_20-07-2007.ttf"
  Push "AdorshoLipi (TrueType)"
  Call InstallFontWithConflictCheck

  Push "Bangla.ttf"
  Push "Bangla (TrueType)"
  Call InstallFontWithConflictCheck

  ; Broadcast WM_FONTCHANGE so Word, Photoshop, Excel immediately pick up the fonts
  DetailPrint "উইন্ডোজ অ্যাপ্লিকেশনে ফন্ট আপডেট বার্তা সম্প্রচার করা হচ্ছে (WM_FONTCHANGE)..."
  SendMessage ${HWND_BROADCAST} ${WM_FONTCHANGE} 0 0
SectionEnd

Section "Windows Auto-Startup (স্বয়ংক্রিয় স্টার্টআপ)" SecAutostart
  DetailPrint "উইন্ডোজ স্টার্টআপে বর্ণমালা যুক্ত করা হচ্ছে..."
  WriteRegStr HKLM "${PRODUCT_AUTORUN_KEY}" "Bornomala" '"$INSTDIR\bornomala.exe" --startup'
SectionEnd

Section "Start Menu & Desktop Shortcuts (শর্টকাটসমূহ)" SecShortcuts
  SetShellVarContext all
  DetailPrint "ডেস্কটপ ও স্টার্ট মেনু শর্টকাট তৈরি হচ্ছে..."
  CreateDirectory "$SMPROGRAMS\Bornomala"
  CreateShortcut "$SMPROGRAMS\Bornomala\Bornomala (বর্ণমালা).lnk" "$INSTDIR\bornomala.exe" "" "$INSTDIR\bornomala.ico" 0
  CreateShortcut "$SMPROGRAMS\Bornomala\Bornomala Converter.lnk" "$INSTDIR\bornomala.exe" "--converter" "$INSTDIR\bornomala.ico" 0
  CreateShortcut "$SMPROGRAMS\Bornomala\Uninstall Bornomala.lnk" "$INSTDIR\Uninstall.exe" "" "$INSTDIR\Uninstall.exe" 0
  CreateShortcut "$DESKTOP\Bornomala (বর্ণমালা).lnk" "$INSTDIR\bornomala.exe" "" "$INSTDIR\bornomala.ico" 0
SectionEnd

; ----------------------------------------------------------------------------
; Component Descriptions
; ----------------------------------------------------------------------------
!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecCore} "মূল টাইপিং ইঞ্জিন ও TSF টেক্সট সার্ভিস ডায়নামিক লাইব্রেরি (অবশ্যই প্রয়োজন)।"
  !insertmacro MUI_DESCRIPTION_TEXT ${SecFonts} "সরকারি প্রমিত ৮টি বাংলা ফন্ট (নিকশ, সুতোন্নীএমজে, আদর্শলিপি ইত্যাদি)। ইতিমধ্যে থাকলে প্রতিস্থাপনের অপশন দেখানো হবে।"
  !insertmacro MUI_DESCRIPTION_TEXT ${SecAutostart} "কম্পিউটার চালু হওয়ার সাথে সাথে স্বয়ংক্রিয়ভাবে ব্যাকগ্রাউন্ডে বর্ণমালা চালু করবে।"
  !insertmacro MUI_DESCRIPTION_TEXT ${SecShortcuts} "স্টার্ট মেনু এবং ডেস্কটপে সহজে চালুর জন্য শর্টকাট তৈরি করবে।"
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; ----------------------------------------------------------------------------
; Uninstaller Section
; ----------------------------------------------------------------------------
Section "Uninstall"
  SetShellVarContext all
  SetRegView 64

  ; Guard: Enforce Bornomala must be closed before files are removed
  Call un.EnsureNotRunning

  DetailPrint "TSF টেক্সট সার্ভিস আনরেজিস্টার করা হচ্ছে..."
  ${If} ${RunningX64}
    ${DisableX64FSRedirection}
    ExecWait 'regsvr32.exe /s /u "$INSTDIR\bornomala_tsf.dll"'
    ${EnableX64FSRedirection}
  ${Else}
    ExecWait 'regsvr32.exe /s /u "$INSTDIR\bornomala_tsf.dll"'
  ${EndIf}

  ; Remove Auto-Startup registry
  DeleteRegValue HKLM "${PRODUCT_AUTORUN_KEY}" "Bornomala"
  DeleteRegValue HKCU "${PRODUCT_AUTORUN_KEY}" "Bornomala"

  ; Remove Application & Uninstall Registry Keys
  DeleteRegKey HKLM "${PRODUCT_UNINST_KEY}"
  DeleteRegKey HKLM "${PRODUCT_DIR_REGKEY}"

  ; Remove Shortcuts
  Delete "$DESKTOP\Bornomala (বর্ণমালা).lnk"
  Delete "$SMPROGRAMS\Bornomala\Bornomala (বর্ণমালা).lnk"
  Delete "$SMPROGRAMS\Bornomala\Bornomala Converter.lnk"
  Delete "$SMPROGRAMS\Bornomala\Uninstall Bornomala.lnk"
  RMDir "$SMPROGRAMS\Bornomala"

  ; Remove Installed Files
  Delete "$INSTDIR\bornomala.exe"
  Delete "$INSTDIR\bornomala_tsf.dll"
  Delete "$INSTDIR\bornomala-cli.exe"
  Delete "$INSTDIR\bornomala.ico"
  Delete "$INSTDIR\TERMS.txt"
  Delete "$INSTDIR\Uninstall.exe"

  ; Remove fonts in app folder
  Delete "$INSTDIR\fonts\*.ttf"
  RMDir "$INSTDIR\fonts"

  ; Remove assets
  Delete "$INSTDIR\assets\branding\*.bmp"
  Delete "$INSTDIR\assets\branding\*.png"
  RMDir "$INSTDIR\assets\branding"
  RMDir "$INSTDIR\assets"

  RMDir "$INSTDIR"
  
  DetailPrint "বর্ণমালা সফলভাবে আনইনস্টল হয়েছে।"
SectionEnd
