//! Bornomala Authentic Desi Settings & Configuration Window
//!
//! Provides full user customization for:
//! 1. General & Startup Behavior (Start in tray, Auto-launch on boot, Splash screen, Default mode)
//! 2. Custom Hotkeys & Commands (Phonetic toggle, Unicode, ANSI, English, Converter)
//! 3. Phonetic Rules & Text Expansion (Auto-Dari, Smart Juktoborno, Word Suggestions, Custom rules)
//! 4. Automatic Updates & Manual Checking
//!
//! Styled in authentic Desi Bengali theme: Forest Green (#066a4a), Crimson Red (#e02f2d),
//! and warm Parchment (#fef7de / #fbf3dc).

use super::api::*;
use super::assets::{self, IconId};
use bornomala_engine::{AppConfig, CustomRule};
use std::sync::Mutex;

static mut SETTINGS_HWND: HWND = 0;
static CURRENT_CONFIG: Mutex<Option<AppConfig>> = Mutex::new(None);

// Control IDs
const ID_CHK_START_TRAY: usize = 4001;
const ID_CHK_AUTO_START: usize = 4002;
const ID_CHK_SHOW_SPLASH: usize = 4003;
const ID_RADIO_MODE_EN: usize = 4004;
const ID_RADIO_MODE_UNI: usize = 4005;
const ID_RADIO_MODE_ANSI: usize = 4006;
const ID_RADIO_MODE_PHO: usize = 4007;

const ID_EDIT_HK_PHONETIC: usize = 4011;
const ID_EDIT_HK_UNICODE: usize = 4012;
const ID_EDIT_HK_ANSI: usize = 4013;
const ID_EDIT_HK_ENGLISH: usize = 4014;
const ID_EDIT_HK_CONVERTER: usize = 4015;

const ID_CHK_AUTO_DARI: usize = 4021;
const ID_CHK_SMART_JUKTO: usize = 4022;
const ID_CHK_SUGGESTIONS: usize = 4023;
const ID_EDIT_CUSTOM_RULES: usize = 4024;

const ID_CHK_AUTO_UPDATE: usize = 4031;
const ID_BTN_CHECK_UPDATE_NOW: usize = 4032;

const ID_BTN_SAVE: usize = 4041;
const ID_BTN_CANCEL: usize = 4042;
const ID_BTN_RESET_DEFAULTS: usize = 4043;

pub struct SettingsWindow;

impl SettingsWindow {
    pub fn open() {
        unsafe {
            if SETTINGS_HWND != 0 && IsWindowVisible(SETTINGS_HWND) != 0 {
                ShowWindow(SETTINGS_HWND, SW_RESTORE);
                SetForegroundWindow(SETTINGS_HWND);
                return;
            }

            let hinst = GetModuleHandleW(std::ptr::null());
            let class_name = to_wide("BornomalaSettingsClass");

            let mut wc: WNDCLASSEXW = std::mem::zeroed();
            wc.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
            wc.lpfnWndProc = Some(settings_wnd_proc);
            wc.hInstance = hinst;
            wc.hCursor = LoadCursorW(0, 32512 as *const u16);
            wc.hbrBackground = CreateSolidBrush(rgb(251, 243, 220)); // Parchment #fbf3dc
            wc.lpszClassName = class_name.as_ptr();

            RegisterClassExW(&wc);

            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            let width = 640;
            let height = 540;
            let x = (screen_w - width) / 2;
            let y = (screen_h - height) / 2;

            let title = to_wide("বর্ণমালা — কনফিগারেশন ও সেটিংস (Bornomala Settings)");
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE,
                x,
                y,
                width,
                height,
                0,
                0,
                hinst,
                std::ptr::null_mut(),
            );

            SETTINGS_HWND = hwnd;
            ShowWindow(hwnd, SW_SHOW);
            SetForegroundWindow(hwnd);
        }
    }
}

unsafe extern "system" fn settings_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let hinst = GetModuleHandleW(std::ptr::null());
            let config = AppConfig::load();
            if let Ok(mut lock) = CURRENT_CONFIG.lock() {
                *lock = Some(config.clone());
            }

            let font_label = CreateFontW(
                -13, 0, 0, 0, FW_BOLD, 0, 0, 0, DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                to_wide("Nirmala UI").as_ptr(),
            );
            let font_normal = CreateFontW(
                -12, 0, 0, 0, FW_NORMAL, 0, 0, 0, DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                to_wide("Nirmala UI").as_ptr(),
            );

            // ================================================================
            // SECTION 1: সাধারণ ও স্টার্টআপ (General & Startup)
            // ================================================================
            let chk_tray = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(),
                to_wide("টাস্কবার / সিস্টেম ট্রে-তে শুরু করুন (Start minimized in tray)").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                30, 95, 340, 22, hwnd, ID_CHK_START_TRAY, hinst, std::ptr::null_mut(),
            );
            SendMessageW(chk_tray, WM_SETFONT, font_normal, 1);
            if config.toolbar_minimized {
                SendMessageW(chk_tray, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
            }

            let chk_autostart = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(),
                to_wide("কম্পিউটার চালুর সাথে স্বয়ংক্রিয়ভাবে চালু করুন (Auto-launch on startup)").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                30, 120, 360, 22, hwnd, ID_CHK_AUTO_START, hinst, std::ptr::null_mut(),
            );
            SendMessageW(chk_autostart, WM_SETFONT, font_normal, 1);
            if config.launch_on_startup {
                SendMessageW(chk_autostart, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
            }

            let chk_splash = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(),
                to_wide("স্টার্টআপ স্প্ল্যাশ স্ক্রিন দেখান (Show authentic startup screen)").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                30, 145, 340, 22, hwnd, ID_CHK_SHOW_SPLASH, hinst, std::ptr::null_mut(),
            );
            SendMessageW(chk_splash, WM_SETFONT, font_normal, 1);
            if config.show_splash_screen {
                SendMessageW(chk_splash, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
            }

            // Default Mode Radios
            let r_en = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(), to_wide("ইংরেজি (EN)").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTORADIOBUTTON,
                160, 175, 95, 22, hwnd, ID_RADIO_MODE_EN, hinst, std::ptr::null_mut(),
            );
            let r_uni = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(), to_wide("ইউনিকোড").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTORADIOBUTTON,
                260, 175, 80, 22, hwnd, ID_RADIO_MODE_UNI, hinst, std::ptr::null_mut(),
            );
            let r_ansi = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(), to_wide("বিজয় এএনএসআই").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTORADIOBUTTON,
                345, 175, 115, 22, hwnd, ID_RADIO_MODE_ANSI, hinst, std::ptr::null_mut(),
            );
            let r_pho = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(), to_wide("ফোনেটিক").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTORADIOBUTTON,
                465, 175, 80, 22, hwnd, ID_RADIO_MODE_PHO, hinst, std::ptr::null_mut(),
            );
            SendMessageW(r_en, WM_SETFONT, font_normal, 1);
            SendMessageW(r_uni, WM_SETFONT, font_normal, 1);
            SendMessageW(r_ansi, WM_SETFONT, font_normal, 1);
            SendMessageW(r_pho, WM_SETFONT, font_normal, 1);

            match config.default_mode.as_str() {
                "unicode" => SendMessageW(r_uni, BM_SETCHECK, BST_CHECKED as WPARAM, 0),
                "ansi" => SendMessageW(r_ansi, BM_SETCHECK, BST_CHECKED as WPARAM, 0),
                "phonetic" => SendMessageW(r_pho, BM_SETCHECK, BST_CHECKED as WPARAM, 0),
                _ => SendMessageW(r_en, BM_SETCHECK, BST_CHECKED as WPARAM, 0),
            };

            // ================================================================
            // SECTION 2: কীবোর্ড শর্টকাট (Hotkeys & Commands)
            // ================================================================
            let make_edit = |id: usize, text: &str, x: i32, y: i32, w: i32| -> HWND {
                let h = CreateWindowExW(
                    0, to_wide("EDIT").as_ptr(), to_wide(text).as_ptr(),
                    WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL,
                    x, y, w, 24, hwnd, id, hinst, std::ptr::null_mut(),
                );
                SendMessageW(h, WM_SETFONT, font_normal, 1);
                h
            };

            make_edit(ID_EDIT_HK_PHONETIC, &config.hotkey_phonetic, 130, 230, 80);
            make_edit(ID_EDIT_HK_UNICODE, &config.hotkey_unicode, 310, 230, 95);
            make_edit(ID_EDIT_HK_ANSI, &config.hotkey_ansi, 500, 230, 95);
            make_edit(ID_EDIT_HK_ENGLISH, &config.hotkey_english, 130, 262, 95);
            make_edit(ID_EDIT_HK_CONVERTER, &config.hotkey_converter, 340, 262, 95);

            // ================================================================
            // SECTION 3: টাইপিং রুলস ও কাস্টম কমান্ড (Rules & Expansion)
            // ================================================================
            let chk_dari = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(),
                to_wide("স্বয়ংক্রিয় দাঁড়ি ('.' = '।')").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                30, 316, 170, 22, hwnd, ID_CHK_AUTO_DARI, hinst, std::ptr::null_mut(),
            );
            SendMessageW(chk_dari, WM_SETFONT, font_normal, 1);
            if config.auto_dari {
                SendMessageW(chk_dari, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
            }

            let chk_jukto = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(),
                to_wide("স্মার্ট যুক্তবর্ণ সংযোগ (Auto-join)").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                205, 316, 205, 22, hwnd, ID_CHK_SMART_JUKTO, hinst, std::ptr::null_mut(),
            );
            SendMessageW(chk_jukto, WM_SETFONT, font_normal, 1);
            if config.smart_juktoborno {
                SendMessageW(chk_jukto, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
            }

            let chk_sug = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(),
                to_wide("শব্দের পূর্বাভাস (Suggestions)").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                415, 316, 185, 22, hwnd, ID_CHK_SUGGESTIONS, hinst, std::ptr::null_mut(),
            );
            SendMessageW(chk_sug, WM_SETFONT, font_normal, 1);
            if config.enable_suggestions {
                SendMessageW(chk_sug, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
            }

            // Custom expansion rules (Multiline edit)
            let mut rules_text = String::new();
            for rule in &config.custom_rules {
                rules_text.push_str(&format!("{} = {}\r\n", rule.trigger, rule.replacement));
            }
            let edit_rules = CreateWindowExW(
                0, to_wide("EDIT").as_ptr(), to_wide(&rules_text).as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_BORDER | ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN,
                30, 370, 560, 48, hwnd, ID_EDIT_CUSTOM_RULES, hinst, std::ptr::null_mut(),
            );
            SendMessageW(edit_rules, WM_SETFONT, font_normal, 1);

            // ================================================================
            // SECTION 4: আপডেট ও চেক বাটন (Updates)
            // ================================================================
            let chk_up = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(),
                to_wide("চালু করার সময় স্বয়ংক্রিয়ভাবে আপডেট পরীক্ষা করুন").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                30, 442, 330, 22, hwnd, ID_CHK_AUTO_UPDATE, hinst, std::ptr::null_mut(),
            );
            SendMessageW(chk_up, WM_SETFONT, font_normal, 1);
            if config.auto_check_updates {
                SendMessageW(chk_up, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
            }

            let btn_chk_now = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(),
                to_wide("আপডেট চেক করুন (Check Now)").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                370, 440, 220, 26, hwnd, ID_BTN_CHECK_UPDATE_NOW, hinst, std::ptr::null_mut(),
            );
            SendMessageW(btn_chk_now, WM_SETFONT, font_normal, 1);

            // ================================================================
            // FOOTER BUTTONS
            // ================================================================
            let btn_save = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(),
                to_wide("সংরক্ষণ করুন (Save Settings)").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
                340, 485, 170, 38, hwnd, ID_BTN_SAVE, hinst, std::ptr::null_mut(),
            );
            SendMessageW(btn_save, WM_SETFONT, font_label, 1);

            let btn_cancel = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(),
                to_wide("বাতিল (Cancel)").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                520, 485, 80, 38, hwnd, ID_BTN_CANCEL, hinst, std::ptr::null_mut(),
            );
            SendMessageW(btn_cancel, WM_SETFONT, font_normal, 1);

            let btn_reset = CreateWindowExW(
                0, to_wide("BUTTON").as_ptr(),
                to_wide("ডিফল্ট সেটিংস (Reset)").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                30, 485, 150, 38, hwnd, ID_BTN_RESET_DEFAULTS, hinst, std::ptr::null_mut(),
            );
            SendMessageW(btn_reset, WM_SETFONT, font_normal, 1);

            0
        }

        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let color_green = rgb(6, 106, 74);
            let color_red = rgb(224, 47, 45);
            let color_parchment = rgb(251, 243, 220);
            let color_card_border = rgb(6, 106, 74);

            // Double Buffering
            let mut client_rc: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut client_rc);
            let w = client_rc.right - client_rc.left;
            let h = client_rc.bottom - client_rc.top;

            let mem_dc = CreateCompatibleDC(hdc);
            let mem_bmp = CreateCompatibleBitmap(hdc, w, h);
            let _old_bmp = SelectObject(mem_dc, mem_bmp);

            // Background Fill
            let bg_brush = CreateSolidBrush(color_parchment);
            FillRect(mem_dc, &client_rc, bg_brush);
            DeleteObject(bg_brush);

            // 1. Desi Header Banner (Forest Green #066a4a with Crimson Red Circle)
            let header_brush = CreateSolidBrush(color_green);
            let header_rc = RECT { left: 0, top: 0, right: w, bottom: 68 };
            FillRect(mem_dc, &header_rc, header_brush);
            DeleteObject(header_brush);

            // Draw Red Circle emblem behind logo on header
            let red_brush = CreateSolidBrush(color_red);
            let red_pen = CreatePen(PS_SOLID, 1, color_red);
            let old_brush = SelectObject(mem_dc, red_brush);
            let old_pen = SelectObject(mem_dc, red_pen);
            Ellipse(mem_dc, 16, 10, 66, 60);
            SelectObject(mem_dc, old_brush);
            SelectObject(mem_dc, old_pen);
            DeleteObject(red_brush);
            DeleteObject(red_pen);

            // Draw Bornomala Logo in header
            assets::draw_icon(mem_dc, IconId::Logo, 22, 16);

            // Header Titles
            SetBkMode(mem_dc, TRANSPARENT);
            SetTextColor(mem_dc, rgb(254, 247, 222)); // Cream font

            let title_font = CreateFontW(
                -17, 0, 0, 0, FW_BOLD, 0, 0, 0, DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                to_wide("Nirmala UI").as_ptr(),
            );
            let old_f = SelectObject(mem_dc, title_font);
            let mut t_rect = RECT { left: 78, top: 13, right: 600, bottom: 40 };
            DrawTextW(mem_dc, to_wide("বর্ণমালা সেটিংস ও কনফিগারেশন").as_ptr(), -1, &mut t_rect, DT_LEFT | DT_SINGLELINE);

            let sub_font = CreateFontW(
                -12, 0, 0, 0, FW_NORMAL, 0, 0, 0, DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                to_wide("Nirmala UI").as_ptr(),
            );
            SelectObject(mem_dc, sub_font);
            let mut st_rect = RECT { left: 78, top: 38, right: 600, bottom: 60 };
            DrawTextW(mem_dc, to_wide("কাস্টম শর্টকাট, কমান্ড ও টাইপিং রুলস কনফিগার করুন").as_ptr(), -1, &mut st_rect, DT_LEFT | DT_SINGLELINE);

            // 2. Desi Section Container Boxes
            let border_pen = CreatePen(PS_SOLID, 1, color_card_border);
            let old_p = SelectObject(mem_dc, border_pen);
            let null_brush = GetStockObject(5 /* NULL_BRUSH */);
            let old_b = SelectObject(mem_dc, null_brush);

            // Box 1: General (top: 80, bottom: 204)
            RoundRect(mem_dc, 18, 80, w - 18, 204, 8, 8);
            // Box 2: Shortcuts (top: 212, bottom: 295)
            RoundRect(mem_dc, 18, 212, w - 18, 295, 8, 8);
            // Box 3: Rules (top: 304, bottom: 428)
            RoundRect(mem_dc, 18, 304, w - 18, 428, 8, 8);

            SelectObject(mem_dc, old_b);
            SelectObject(mem_dc, old_p);
            DeleteObject(border_pen);

            // Section Titles
            SetTextColor(mem_dc, color_green);
            let sec_font = CreateFontW(
                -13, 0, 0, 0, FW_BOLD, 0, 0, 0, DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                to_wide("Nirmala UI").as_ptr(),
            );
            SelectObject(mem_dc, sec_font);

            // Mode Label
            let mut l_mode = RECT { left: 30, top: 177, right: 155, bottom: 198 };
            DrawTextW(mem_dc, to_wide("ডিফল্ট টাইপিং মোড:").as_ptr(), -1, &mut l_mode, DT_LEFT | DT_SINGLELINE);

            // Shortcut labels
            let mut l_pho = RECT { left: 30, top: 234, right: 125, bottom: 254 };
            DrawTextW(mem_dc, to_wide("ফোনেটিক টগল:").as_ptr(), -1, &mut l_pho, DT_LEFT | DT_SINGLELINE);

            let mut l_uni = RECT { left: 220, top: 234, right: 305, bottom: 254 };
            DrawTextW(mem_dc, to_wide("ইউনিকোড:").as_ptr(), -1, &mut l_uni, DT_LEFT | DT_SINGLELINE);

            let mut l_ansi = RECT { left: 415, top: 234, right: 495, bottom: 254 };
            DrawTextW(mem_dc, to_wide("বিজয় এএনএসআই:").as_ptr(), -1, &mut l_ansi, DT_LEFT | DT_SINGLELINE);

            let mut l_en = RECT { left: 30, top: 266, right: 125, bottom: 286 };
            DrawTextW(mem_dc, to_wide("ইংরেজি মোড:").as_ptr(), -1, &mut l_en, DT_LEFT | DT_SINGLELINE);

            let mut l_conv = RECT { left: 240, top: 266, right: 335, bottom: 286 };
            DrawTextW(mem_dc, to_wide("রূপান্তরকারী:").as_ptr(), -1, &mut l_conv, DT_LEFT | DT_SINGLELINE);

            // Rules subtitle
            let lbl_font = CreateFontW(
                -12, 0, 0, 0, FW_NORMAL, 0, 0, 0, DEFAULT_CHARSET, 0, 0, CLEARTYPE_QUALITY, 0,
                to_wide("Nirmala UI").as_ptr(),
            );
            SelectObject(mem_dc, lbl_font);
            SetTextColor(mem_dc, rgb(50, 60, 55));
            let mut l_rnote = RECT { left: 30, top: 348, right: 580, bottom: 368 };
            DrawTextW(mem_dc, to_wide("কাস্টম শর্টকাট / টেক্সট এক্সপেনশন রুলস (প্রতি লাইনে 'সংক্ষেপ = মূল শব্দ'):").as_ptr(), -1, &mut l_rnote, DT_LEFT | DT_SINGLELINE);

            // Blit to screen
            BitBlt(hdc, 0, 0, w, h, mem_dc, 0, 0, 0x00CC0020);

            SelectObject(mem_dc, old_f);
            DeleteObject(title_font);
            DeleteObject(sub_font);
            DeleteObject(sec_font);
            DeleteObject(lbl_font);
            DeleteObject(mem_bmp);
            DeleteDC(mem_dc);

            EndPaint(hwnd, &ps);
            0
        }

        WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => {
            let hdc_ctrl = wparam as HDC;
            SetBkMode(hdc_ctrl, TRANSPARENT);
            SetTextColor(hdc_ctrl, rgb(6, 106, 74));
            // Return parchment brush
            let parchment_brush = CreateSolidBrush(rgb(251, 243, 220));
            parchment_brush as LRESULT
        }

        WM_COMMAND => {
            let id = wparam & 0xFFFF;
            match id {
                ID_BTN_SAVE => {
                    let mut config = AppConfig::load();

                    // 1. General & Startup
                    config.toolbar_minimized = SendMessageW(GetDlgItem(hwnd, ID_CHK_START_TRAY as i32), BM_GETCHECK, 0, 0) == BST_CHECKED;
                    config.launch_on_startup = SendMessageW(GetDlgItem(hwnd, ID_CHK_AUTO_START as i32), BM_GETCHECK, 0, 0) == BST_CHECKED;
                    config.show_splash_screen = SendMessageW(GetDlgItem(hwnd, ID_CHK_SHOW_SPLASH as i32), BM_GETCHECK, 0, 0) == BST_CHECKED;

                    if SendMessageW(GetDlgItem(hwnd, ID_RADIO_MODE_UNI as i32), BM_GETCHECK, 0, 0) == BST_CHECKED {
                        config.default_mode = "unicode".to_string();
                    } else if SendMessageW(GetDlgItem(hwnd, ID_RADIO_MODE_ANSI as i32), BM_GETCHECK, 0, 0) == BST_CHECKED {
                        config.default_mode = "ansi".to_string();
                    } else if SendMessageW(GetDlgItem(hwnd, ID_RADIO_MODE_PHO as i32), BM_GETCHECK, 0, 0) == BST_CHECKED {
                        config.default_mode = "phonetic".to_string();
                    } else {
                        config.default_mode = "english".to_string();
                    }

                    // 2. Shortcuts
                    let read_edit = |edit_id: usize| -> String {
                        let h = GetDlgItem(hwnd, edit_id as i32);
                        let len = SendMessageW(h, WM_GETTEXTLENGTH, 0, 0) as usize;
                        let mut buf = vec![0u16; len + 1];
                        SendMessageW(h, WM_GETTEXT, (len + 1) as WPARAM, buf.as_mut_ptr() as LPARAM);
                        String::from_utf16_lossy(&buf[..len]).trim().to_string()
                    };

                    config.hotkey_phonetic = read_edit(ID_EDIT_HK_PHONETIC);
                    config.hotkey_unicode = read_edit(ID_EDIT_HK_UNICODE);
                    config.hotkey_ansi = read_edit(ID_EDIT_HK_ANSI);
                    config.hotkey_english = read_edit(ID_EDIT_HK_ENGLISH);
                    config.hotkey_converter = read_edit(ID_EDIT_HK_CONVERTER);

                    // 3. Rules
                    config.auto_dari = SendMessageW(GetDlgItem(hwnd, ID_CHK_AUTO_DARI as i32), BM_GETCHECK, 0, 0) == BST_CHECKED;
                    config.smart_juktoborno = SendMessageW(GetDlgItem(hwnd, ID_CHK_SMART_JUKTO as i32), BM_GETCHECK, 0, 0) == BST_CHECKED;
                    config.enable_suggestions = SendMessageW(GetDlgItem(hwnd, ID_CHK_SUGGESTIONS as i32), BM_GETCHECK, 0, 0) == BST_CHECKED;

                    // Parse multiline custom rules
                    let raw_rules = read_edit(ID_EDIT_CUSTOM_RULES);
                    let mut parsed_rules = Vec::new();
                    for line in raw_rules.lines() {
                        if let Some((trigger, repl)) = line.split_once('=') {
                            let t = trigger.trim();
                            let r = repl.trim();
                            if !t.is_empty() && !r.is_empty() {
                                parsed_rules.push(CustomRule {
                                    trigger: t.to_string(),
                                    replacement: r.to_string(),
                                    enabled: true,
                                });
                            }
                        }
                    }
                    config.custom_rules = parsed_rules;

                    // 4. Updates
                    config.auto_check_updates = SendMessageW(GetDlgItem(hwnd, ID_CHK_AUTO_UPDATE as i32), BM_GETCHECK, 0, 0) == BST_CHECKED;

                    // Save and apply
                    let _ = config.save();
                    config.apply_autostart_registry();

                    // Instantly apply updated hotkeys and rules into keyboard hook & engine
                    super::hook::KeyboardHook::reload_config(super::get_main_hwnd());

                    // If user checked "start in tray", minimize toolbar now; otherwise show it
                    if config.toolbar_minimized {
                        super::minimize_toolbar_to_tray();
                    } else {
                        super::toolbar::FloatingToolbar::show();
                    }

                    let msg = to_wide("বর্ণমালা সেটিংস সফলভাবে সংরক্ষিত হয়েছে।");
                    let title = to_wide("সেটিংস সংরক্ষিত");
                    MessageBoxW(hwnd, msg.as_ptr(), title.as_ptr(), 0x00000040);

                    ShowWindow(hwnd, SW_HIDE);
                    DestroyWindow(hwnd);
                    SETTINGS_HWND = 0;
                }

                ID_BTN_CANCEL => {
                    ShowWindow(hwnd, SW_HIDE);
                    DestroyWindow(hwnd);
                    SETTINGS_HWND = 0;
                }

                ID_BTN_RESET_DEFAULTS => {
                    let default_cfg = AppConfig::default();
                    let set_chk = |chk_id: usize, val: bool| {
                        let h = GetDlgItem(hwnd, chk_id as i32);
                        SendMessageW(h, BM_SETCHECK, if val { BST_CHECKED as WPARAM } else { BST_UNCHECKED as WPARAM }, 0);
                    };

                    set_chk(ID_CHK_START_TRAY, default_cfg.toolbar_minimized);
                    set_chk(ID_CHK_AUTO_START, default_cfg.launch_on_startup);
                    set_chk(ID_CHK_SHOW_SPLASH, default_cfg.show_splash_screen);
                    set_chk(ID_CHK_AUTO_DARI, default_cfg.auto_dari);
                    set_chk(ID_CHK_SMART_JUKTO, default_cfg.smart_juktoborno);
                    set_chk(ID_CHK_SUGGESTIONS, default_cfg.enable_suggestions);
                    set_chk(ID_CHK_AUTO_UPDATE, default_cfg.auto_check_updates);

                    let set_edit = |edit_id: usize, text: &str| {
                        let h = GetDlgItem(hwnd, edit_id as i32);
                        SendMessageW(h, WM_SETTEXT, 0, to_wide(text).as_ptr() as LPARAM);
                    };

                    set_edit(ID_EDIT_HK_PHONETIC, &default_cfg.hotkey_phonetic);
                    set_edit(ID_EDIT_HK_UNICODE, &default_cfg.hotkey_unicode);
                    set_edit(ID_EDIT_HK_ANSI, &default_cfg.hotkey_ansi);
                    set_edit(ID_EDIT_HK_ENGLISH, &default_cfg.hotkey_english);
                    set_edit(ID_EDIT_HK_CONVERTER, &default_cfg.hotkey_converter);
                }

                ID_BTN_CHECK_UPDATE_NOW => {
                    super::updater::trigger_manual_update_check("0.1.0");
                }

                _ => {}
            }
            0
        }

        WM_CLOSE => {
            ShowWindow(hwnd, SW_HIDE);
            DestroyWindow(hwnd);
            SETTINGS_HWND = 0;
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
